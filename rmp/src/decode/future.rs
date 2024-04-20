use std::io::{self, Read};

use alloc::collections::VecDeque;
use bytes::{Buf, Bytes};
use super::{LenError, MessageLen};

/// Collect chunks of bytes until a full message is found
pub struct MessageBuffer {
    /// Once parsing fails, it's impossible to recover
    fatal_error: bool,
    /// A rope for all incoming data
    chunks: VecDeque<Bytes>,
    /// Chunks are parsed lazily
    chunks_parsed_num: usize,
    /// Cumulative byte size of `chunks[..chunks_parsed_num]`
    chunks_parsed_byte_len: usize,
    /// msgpack parser
    msg_len: MessageLen,
}

/// Result after buffering chunk of data
pub enum MaybeMessage {
    /// Found a complete message
    ///
    /// The message is split into pieces
    Message(MessageChunks),
    /// Message not complete yet. Read this many bytes.
    MoreBytes(usize),
}

/// This keeps individual `Bytes` pieces to avoid reallocating memory
///
/// Use `into_inner` to process them manually, or use `MessageChunks` as `io::Read`
pub struct MessageChunks(VecDeque<Bytes>);

impl MessageChunks {
    /// Get the underlying `Bytes`
    pub fn into_inner(self) -> VecDeque<Bytes> {
        self.0
    }
}

/// The exact `IntoIter` type may change in the future
impl IntoIterator for MessageChunks {
    type IntoIter = <VecDeque<Bytes> as IntoIterator>::IntoIter;
    type Item = Bytes;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Read for MessageChunks {
    fn read(&mut self, out_buf: &mut [u8]) -> io::Result<usize> {
        while let Some(bytes) = self.0.get_mut(0) {
            let mut ch = bytes.chunk();
            if ch.is_empty() {
                self.0.pop_front();
                continue;
            }
            let read_len = out_buf.len().min(ch.len());
            out_buf[..read_len].copy_from_slice(&ch[..read_len]);
            if read_len == ch.len() {
                self.0.pop_front();
            } else {
                ch.advance(read_len);
            }
            return Ok(read_len);
        }
        Ok(0)
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> io::Result<usize> {
        let len = self.0.iter().map(|ch| ch.remaining()).sum();
        buf.try_reserve_exact(len).map_err(|_| io::ErrorKind::OutOfMemory)?;
        for c in self.0.drain(..) {
            buf.extend_from_slice(c.chunk());
        }
        Ok(len)
    }
}

impl MessageBuffer {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            fatal_error: false,
            chunks_parsed_num: 0,
            chunks_parsed_byte_len: 0,
            chunks: VecDeque::new(),
            msg_len: MessageLen::new(), // TODO: limits
        }
    }

    /// Parse chunks added with `push_bytes` etc., and dequeue chunks of complete msgpack messages
    pub fn poll_messages(&mut self) -> impl Iterator<Item = Result<MaybeMessage, ()>> + '_ {
        std::iter::from_fn(move || {
            while self.chunks_parsed_num < self.chunks.len() && !self.fatal_error {
                let bytes = &mut self.chunks[self.chunks_parsed_num];
                self.chunks_parsed_num += 1;
                self.chunks_parsed_byte_len += bytes.len();

                match self.msg_len.incremental_len(bytes.as_ref()) {
                    Ok(message_len) => {
                        self.msg_len.reset();

                        let unused_bytes = self.chunks_parsed_byte_len.saturating_sub(message_len);
                        let remainder = bytes.split_off(bytes.len() - unused_bytes);

                        // includes the `bytes` cut
                        let message_data = self.chunks.drain(..self.chunks_parsed_num).collect::<VecDeque<_>>();

                        self.chunks_parsed_byte_len = 0;
                        self.chunks_parsed_num = 0;
                        self.chunks.push_front(remainder);

                        debug_assert!(message_data.iter().all(|b| b.remaining() == b.len()));
                        Some(Ok::<MaybeMessage, ()>(MaybeMessage::Message(MessageChunks(message_data))));
                    },
                    Err(LenError::Truncated(new_len)) => {
                        if self.chunks_parsed_num >= self.chunks.len() {
                            let wants_more = new_len.get().saturating_sub(self.chunks_parsed_byte_len);
                            return Some(Ok(MaybeMessage::MoreBytes(wants_more)));
                        }
                    },
                    Err(LenError::ParseError) => {
                        self.fatal_error = true;
                        return Some(Err(()));
                    },
                }
            }
            None
        })
    }

    /// Buffer more data
    pub fn push_bytes(&mut self, mut bytes: Bytes) {
        // bytes are stateful, and later `io::Read` will use that
        if bytes.remaining() != bytes.len() {
            bytes = bytes.slice(..);
        }
        self.chunks.push_back(bytes);
    }

    /// Buffer more data
    #[inline]
    pub fn push_vec(&mut self, bytes: Vec<u8>) {
        self.push_bytes(bytes.into());
    }

    /// Buffer more data
    #[inline]
    pub fn copy_from_slice(&mut self, bytes: &[u8]) {
        self.push_bytes(Bytes::copy_from_slice(bytes));
    }

    /// Recover buffered data
    pub fn into_bytes(self) -> Vec<Bytes> {
        self.chunks.into()
    }
}

