const FIXSTR_SIZE   : u8 = 0x1f;
const FIXARRAY_SIZE : u8 = 0x0f;
const FIXMAP_SIZE   : u8 = 0x0f;

/// Format markers.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Marker {
    FixPos(u8) = 0x00,
    FixMap(u8) = 0x80,
    FixArray(u8) = 0x90,
    FixStr(u8) = 0xa0,
    Null = 0xc0,
    /// Marked in MessagePack spec as never used.
    Reserved,
    False,
    True,
    Bin8,
    Bin16,
    Bin32,
    Ext8,
    Ext16,
    Ext32,
    F32,
    F64,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    FixExt1,
    FixExt2,
    FixExt4,
    FixExt8,
    FixExt16,
    Str8,
    Str16,
    Str32,
    Array16,
    Array32,
    Map16,
    Map32,
    FixNeg(i8) = 0xe0,
}

impl Marker {
    /// Construct a msgpack marker from a single byte.
    #[must_use]
    #[inline]
    pub const fn from_u8(n: u8) -> Self {
        match n {
            0x00..=0x7f => Self::FixPos(n),
            0x80..=0x8f => Self::FixMap(n & FIXMAP_SIZE),
            0x90..=0x9f => Self::FixArray(n & FIXARRAY_SIZE),
            0xa0..=0xbf => Self::FixStr(n & FIXSTR_SIZE),
            0xc0 => Self::Null,
            // Marked in MessagePack spec as never used.
            0xc1 => Self::Reserved,
            0xc2 => Self::False,
            0xc3 => Self::True,
            0xc4 => Self::Bin8,
            0xc5 => Self::Bin16,
            0xc6 => Self::Bin32,
            0xc7 => Self::Ext8,
            0xc8 => Self::Ext16,
            0xc9 => Self::Ext32,
            0xca => Self::F32,
            0xcb => Self::F64,
            0xcc => Self::U8,
            0xcd => Self::U16,
            0xce => Self::U32,
            0xcf => Self::U64,
            0xd0 => Self::I8,
            0xd1 => Self::I16,
            0xd2 => Self::I32,
            0xd3 => Self::I64,
            0xd4 => Self::FixExt1,
            0xd5 => Self::FixExt2,
            0xd6 => Self::FixExt4,
            0xd7 => Self::FixExt8,
            0xd8 => Self::FixExt16,
            0xd9 => Self::Str8,
            0xda => Self::Str16,
            0xdb => Self::Str32,
            0xdc => Self::Array16,
            0xdd => Self::Array32,
            0xde => Self::Map16,
            0xdf => Self::Map32,
            0xe0..=0xff => Self::FixNeg(n as i8),
        }
    }

    /// Converts a marker object into a single-byte representation.
    #[must_use]
    #[inline]
    pub const fn to_u8(&self) -> u8 {
        match *self {
            Self::FixPos(val)   => val,
            Self::FixNeg(val)   => val as u8,

            Self::Null          => 0xc0,

            Self::True          => 0xc3,
            Self::False         => 0xc2,

            Self::U8            => 0xcc,
            Self::U16           => 0xcd,
            Self::U32           => 0xce,
            Self::U64           => 0xcf,

            Self::I8            => 0xd0,
            Self::I16           => 0xd1,
            Self::I32           => 0xd2,
            Self::I64           => 0xd3,

            Self::F32           => 0xca,
            Self::F64           => 0xcb,

            Self::FixStr(len)   => 0xa0 | (len & FIXSTR_SIZE),
            Self::Str8          => 0xd9,
            Self::Str16         => 0xda,
            Self::Str32         => 0xdb,

            Self::Bin8          => 0xc4,
            Self::Bin16         => 0xc5,
            Self::Bin32         => 0xc6,

            Self::FixArray(len) => 0x90 | (len & FIXARRAY_SIZE),
            Self::Array16       => 0xdc,
            Self::Array32       => 0xdd,

            Self::FixMap(len)   => 0x80 | (len & FIXMAP_SIZE),
            Self::Map16         => 0xde,
            Self::Map32         => 0xdf,

            Self::FixExt1       => 0xd4,
            Self::FixExt2       => 0xd5,
            Self::FixExt4       => 0xd6,
            Self::FixExt8       => 0xd7,
            Self::FixExt16      => 0xd8,
            Self::Ext8          => 0xc7,
            Self::Ext16         => 0xc8,
            Self::Ext32         => 0xc9,

            Self::Reserved      => 0xc1,
        }
    }
}

impl From<u8> for Marker {
    #[inline(always)]
    fn from(val: u8) -> Self {
        Self::from_u8(val)
    }
}

impl From<Marker> for u8 {
    #[inline(always)]
    fn from(val: Marker) -> Self {
        val.to_u8()
    }
}
