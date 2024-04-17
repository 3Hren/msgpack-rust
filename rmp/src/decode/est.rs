use std::num::{NonZeroU32, NonZeroUsize};
use crate::Marker;

/// Incremental MessagePack parser that can parse incomplete messages,
/// and report their estimated total length.
pub struct MessageLen {
    /// The last operation interrupted
    wip: Option<WIP>,
    /// Max size estimate
    max_position: NonZeroUsize,
    /// Bytes read so far
    position: usize,
    /// Stack of open arrays and maps
    /// It is not a complete stack. Used only when resumption is needed.
    sequences_wip: Vec<Seq>,
    /// Nesting of arrays and maps
    current_depth: u16,
    /// Configured limit
    max_depth: u16,
    /// Configured limit
    max_len: u32,
}

/// [`MessageLen`] result
#[derive(Debug)]
pub enum LenError {
    /// The message is truncated, and needs at least this many bytes to parse
    Truncated(NonZeroUsize),
    /// The message is invalid or exceeded size limits
    ParseError,
}

impl LenError {
    /// Get expected min length or 0 on error
    pub fn len(&self) -> usize {
        match *self {
            Self::ParseError => 0,
            Self::Truncated(l) => l.get(),
        }
    }
}

impl MessageLen {
    /// New parser with default limits
    ///
    /// If you have all MessagePack data in memory already, you can use [`MessageLen::len_of`].
    /// If you're reading data in a streaming fashion, you can feed chunks of data
    /// to [`MessageLen::incremental_len`].
    pub fn new() -> Self {
        Self::with_limits(1024, (u32::MAX as usize).min(isize::MAX as usize / 2))
    }

    /// * `max_depth` limits nesting of arrays and maps
    ///
    /// * `max_len` is maximum size of any string, byte string, map, or array.
    ///    For maps and arrays this is the number of items, not bytes.
    ///
    /// Messages can be both deep and wide, being `max_depth` * `max_len` in size.
    /// You should also limit the maximum byte size of the message (outside of this parser).
    pub fn with_limits(max_depth: usize, max_len: usize) -> Self {
        Self {
            max_position: NonZeroUsize::new(1).unwrap(),
            position: 0,
            current_depth: 0,
            max_depth: max_depth.min(u16::MAX as _) as u16,
            max_len: max_len.min(u32::MAX as _) as u32,
            sequences_wip: Vec::new(),
            wip: Some(WIP::NextMarker),
        }
    }

    /// Parse the entire message to find if it's complete, and what is its serialized size in bytes.
    ///
    /// If it returns `Ok(len)`, then the first `len` bytes of the given slice
    /// parse as a single MessagePack object.
    /// The length may be shorter than the slice given (extra data is gracefully ignored).
    ///
    /// `Err(LenError::Truncated(len))` means that the the object is incomplete, the slice is truncated,
    /// and it would need *at least* this many bytes to parse.
    /// The `len` is always the lower bound, and never exceeds actual message length.
    ///
    /// `Err(LenError::ParseError)` — the end of the message is unknown.
    ///
    /// Don't call this function in a loop. Use [`MessageLen::incremental_len`] instead.
    pub fn len_of(complete_message: &[u8]) -> Result<usize, LenError> {
        Self::with_limits(1024, 1<<30).incremental_len(&mut complete_message.as_ref())
    }

    /// Parse more bytes, and re-evaluate required message length.
    ///
    /// This function is stateful and keeps "appending" the data to its evaluation.
    ///
    /// * `Ok(len)` — size of the whole MessagePack object, in bytes, starting at the beginning
    ///   of all data given to this function, including previous calls (not just this slice).
    ///   The object's data may end before the end of this slice. In such case the extra bytes
    ///   are gracefully ignored, and have not been parsed.
    ///
    /// * `Err(LenError::Truncated(len))` — all bytes of this slice have been consumed,
    ///   and that was still not enough. The object needs at least `len` bytes in total
    ///   (counting from the start of all data given to this function, not just this slice).
    ///   The `len` is always the lower bound, and never exceeds actual message length,
    ///   so it's safe to read the additional bytes without overshooting the end of the message.
    ///
    /// * `Err(LenError::ParseError)` — the end of the message cannot be determined, and this
    ///   is a non-recoverable error. Any further calls to this function may return nonsense.
    pub fn incremental_len(&mut self, mut next_message_fragment: &[u8]) -> Result<usize, LenError> {
        let data = &mut next_message_fragment;
        let wip = match self.wip.take() {
            Some(wip) => wip,
            None => return Ok(self.position), // must have succeded already
        };
        match wip {
            WIP::Data(Data { bytes_left }) => self.skip_data(data, bytes_left.get()),
            WIP::MarkerLen(wip) => self.read_marker_with_len(data, wip),
            WIP::NextMarker => self.read_one_item(data),
            WIP::LimitExceeded => {
                self.wip = Some(WIP::LimitExceeded); // put it back!
                return Err(LenError::ParseError);
            },
        }.ok_or(LenError::Truncated(self.max_position))?;

        while let Some(seq) = self.sequences_wip.pop() {
            self.current_depth = seq.depth;
            debug_assert!(self.wip.is_none());
            self.read_sequence(data, seq.items_left.get() - 1).ok_or(LenError::Truncated(self.max_position))?;
        }
        debug_assert!(self.wip.is_none());
        debug_assert!(self.max_position.get() <= self.position);
        Ok(self.position)
    }

    /// Forget all the state. The next call to `incremental_len` will assume it's the start of a new message.
    pub fn reset(&mut self) {
        self.max_position = NonZeroUsize::new(1).unwrap();
        self.position = 0;
        self.current_depth = 0;
        self.sequences_wip.clear();
        self.wip = Some(WIP::NextMarker);
    }

    fn read_one_item(&mut self, data: &mut &[u8]) -> Option<()> {
        debug_assert!(self.wip.is_none());
        let marker = self.read_marker(data)?;
        match marker {
            Marker::FixPos(_) => Some(()),
            Marker::FixMap(len) => self.read_sequence(data, u32::from(len) * 2),
            Marker::FixArray(len) => self.read_sequence(data, u32::from(len)),
            Marker::FixStr(len) => self.skip_data(data, len.into()),
            Marker::Null |
            Marker::Reserved |
            Marker::False |
            Marker::True => Some(()),
            Marker::Str8 |
            Marker::Str16 |
            Marker::Str32 |
            Marker::Bin8 |
            Marker::Bin16 |
            Marker::Bin32 |
            Marker::Array16 |
            Marker::Array32 |
            Marker::Map16 |
            Marker::Map32 => self.read_marker_with_len(data, MarkerLen { marker, buf: [0; 4], has: 0 }),
            Marker::Ext8 |
            Marker::Ext16 |
            Marker::Ext32 => todo!(),
            Marker::F32 => self.skip_data(data, 4),
            Marker::F64 => self.skip_data(data, 8),
            Marker::U8 => self.skip_data(data, 1),
            Marker::U16 => self.skip_data(data, 2),
            Marker::U32 => self.skip_data(data, 4),
            Marker::U64 => self.skip_data(data, 8),
            Marker::I8 => self.skip_data(data, 1),
            Marker::I16 => self.skip_data(data, 2),
            Marker::I32 => self.skip_data(data, 4),
            Marker::I64 => self.skip_data(data, 8),
            Marker::FixExt1 |
            Marker::FixExt2 |
            Marker::FixExt4 |
            Marker::FixExt8 |
            Marker::FixExt16 => todo!(),
            Marker::FixNeg(_) => Some(()),
        }
    }

    fn read_marker_with_len(&mut self, data: &mut &[u8], mut wip: MarkerLen) -> Option<()> {
        let size = wip.size();
        debug_assert!(wip.has < size && size > 0 && size <= 4);
        let dest = &mut wip.buf[0..size as usize];
        let wanted = dest.len().checked_sub(wip.has as _)?;

        let taken = self.take_bytes(data, wanted as u32);
        dest[wip.has as usize..][..taken.len()].copy_from_slice(taken);
        wip.has += taken.len() as u8;
        if wip.has < size {
            return self.fail(WIP::MarkerLen(wip));
        }
        let len = match dest.len() {
            1 => dest[0].into(),
            2 => u16::from_be_bytes(dest.try_into().unwrap()).into(),
            4 => u32::from_be_bytes(dest.try_into().unwrap()),
            _ => {
                debug_assert!(false);
                return None
            },
        };
        if len >= self.max_len {
            return self.fail(WIP::LimitExceeded);
        }
        match wip.marker {
            Marker::Bin8 |
            Marker::Bin16 |
            Marker::Bin32 |
            Marker::Str8 |
            Marker::Str16 |
            Marker::Str32 => self.skip_data(data, len),
            Marker::Ext8 |
            Marker::Ext16 |
            Marker::Ext32 => todo!(),
            Marker::Array16 |
            Marker::Array32 => self.read_sequence(data, len),
            Marker::Map16 |
            Marker::Map32 => {
                if let Some(len) = len.checked_mul(2).filter(|&l| l < self.max_len) {
                    self.read_sequence(data, len)
                } else {
                    self.fail(WIP::LimitExceeded)
                }
            },
            _ => {
                debug_assert!(false);
                None
            }
        }
    }

    fn read_sequence(&mut self, data: &mut &[u8], mut items_left: u32) -> Option<()> {
        self.current_depth += 1;
        if self.current_depth > self.max_depth {
            return self.fail(WIP::LimitExceeded);
        }
        while let Some(non_zero) = NonZeroU32::new(items_left) {
            let position_before_item = self.position;
            self.read_one_item(data).or_else(|| {
                self.set_max_position(position_before_item + items_left as usize);
                // -1, because it will increase depth again when resumed
                self.sequences_wip.push(Seq { items_left: non_zero, depth: self.current_depth-1 });
                None
            })?;
            items_left -= 1;
        }
        debug_assert!(self.current_depth > 0);
        self.current_depth -= 1;
        Some(())
    }

    fn skip_data(&mut self, data: &mut &[u8], wanted: u32) -> Option<()> {
        let taken = self.take_bytes(data, wanted);
        if let Some(bytes_left) = NonZeroU32::new(wanted - taken.len() as u32) {
            debug_assert!(data.is_empty());
            self.fail(WIP::Data(Data { bytes_left }))
        } else {
            Some(())
        }
    }

    fn read_marker(&mut self, data: &mut &[u8]) -> Option<Marker> {
        let Some((&b, rest)) = data.split_first() else {
            debug_assert!(data.is_empty());
            return self.fail(WIP::NextMarker);
        };
        self.position += 1;
        *data = rest;
        Some(Marker::from_u8(b))
    }

    fn set_max_position(&mut self, position: usize) {
        self.max_position = NonZeroUsize::new(self.max_position.get().max(position)).unwrap();
    }

    /// May return less than requested
    fn take_bytes<'data>(&mut self, data: &mut &'data [u8], wanted: u32) -> &'data [u8] {
        let (taken, rest) = data.split_at(data.len().min(wanted as usize));
        self.position += taken.len();
        *data = rest;
        taken
    }

    #[inline(always)]
    fn fail<T>(&mut self, wip: WIP) -> Option<T> {
        debug_assert!(self.wip.is_none());
        let pos = match self.wip.insert(wip) {
            WIP::NextMarker => self.position + 1,
            WIP::Data(Data { bytes_left }) => self.position + bytes_left.get() as usize,
            WIP::MarkerLen(m) => self.position + (m.size() - m.has) as usize,
            WIP::LimitExceeded => 0,
        };
        self.set_max_position(pos);
        None
    }
}

enum WIP {
    NextMarker,
    Data(Data),
    MarkerLen(MarkerLen),
    LimitExceeded,
}

struct Seq { items_left: NonZeroU32, depth: u16 }
struct Data { bytes_left: NonZeroU32 }
struct MarkerLen { marker: Marker, buf: [u8; 4], has: u8 }

impl MarkerLen {
    fn size(&self) -> u8 {
        match self.marker {
            Marker::Bin8 => 1,
            Marker::Bin16 => 2,
            Marker::Bin32 => 4,
            Marker::Ext8 => 1,
            Marker::Ext16 => 2,
            Marker::Ext32 => 4,
            Marker::Str8 => 1,
            Marker::Str16 => 2,
            Marker::Str32 => 4,
            Marker::Array16 => 2,
            Marker::Array32 => 4,
            Marker::Map16 => 2,
            Marker::Map32 => 4,
            _ => unimplemented!(),
        }
    }
}
