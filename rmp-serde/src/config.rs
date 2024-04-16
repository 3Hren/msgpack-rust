//! Change MessagePack behavior with configuration wrappers.

/// Represents configuration that dicatates what the serializer does.
///
/// Implemented as an empty trait depending on a hidden trait in order to allow changing the
/// methods of this trait without breaking backwards compatibility.
pub trait SerializerConfig: sealed::SerializerConfig {}

impl<T: sealed::SerializerConfig> SerializerConfig for T {}

pub(crate) mod sealed {
    /// This is the inner trait - the real `SerializerConfig`.
    ///
    /// This hack disallows external implementations and usage of `SerializerConfig` and thus
    /// allows us to change `SerializerConfig` methods freely without breaking backwards compatibility.
    pub trait SerializerConfig: Copy {
        /// Determines the value of `Serializer::is_human_readable` and
        /// `Deserializer::is_human_readable`.
        fn is_human_readable(&self) -> bool;

        /// String struct fields
        fn is_named(&self) -> bool;
    }
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct RuntimeConfig {
    pub(crate) is_human_readable: bool,
    pub(crate) is_named: bool,
}

impl RuntimeConfig {
    pub(crate) fn new(other: impl sealed::SerializerConfig) -> Self {
        Self {
            is_human_readable: other.is_human_readable(),
            is_named: other.is_named(),
        }
    }
}

impl sealed::SerializerConfig for RuntimeConfig {
    #[inline]
    fn is_human_readable(&self) -> bool {
        self.is_human_readable
    }

    #[inline]
    fn is_named(&self) -> bool {
        self.is_named
    }
}

/// The default serializer/deserializer configuration.
///
/// This configuration:
/// - Writes structs as a tuple, without field names
/// - Writes enum variants as integers
/// - Writes and reads types as binary, not human-readable
//
/// This is the most compact representation.
#[derive(Copy, Clone, Debug)]
pub struct DefaultConfig;

impl sealed::SerializerConfig for DefaultConfig {
    #[inline(always)]
    fn is_named(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        false
    }
}

/// Config wrapper, that overrides struct serialization by packing as a map with field names.
///
/// MessagePack specification does not tell how to serialize structs. This trait allows you to
/// extend serialization to match your app's requirements.
///
/// Default `Serializer` implementation writes structs as a tuple, i.e. only its length is encoded,
/// because it is the most compact representation.
#[derive(Copy, Clone, Debug)]
pub struct StructMapConfig<C>(C);

impl<C> StructMapConfig<C> {
    /// Creates a `StructMapConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        StructMapConfig(inner)
    }
}

impl<C> sealed::SerializerConfig for StructMapConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline(always)]
    fn is_named(&self) -> bool {
        true
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

/// Config wrapper that overrides struct serlization by packing as a tuple without field
/// names.
#[derive(Copy, Clone, Debug)]
pub struct StructTupleConfig<C>(C);

impl<C> StructTupleConfig<C> {
    /// Creates a `StructTupleConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        StructTupleConfig(inner)
    }
}

impl<C> sealed::SerializerConfig for StructTupleConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline(always)]
    fn is_named(&self) -> bool {
        false
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        self.0.is_human_readable()
    }
}

/// Config wrapper that overrides `Serializer::is_human_readable` and
/// `Deserializer::is_human_readable` to return `true`.
#[derive(Copy, Clone, Debug)]
pub struct HumanReadableConfig<C>(C);

impl<C> HumanReadableConfig<C> {
    /// Creates a `HumanReadableConfig` inheriting unchanged configuration options from the given configuration.
    #[inline]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for HumanReadableConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline(always)]
    fn is_named(&self) -> bool {
        self.0.is_named()
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        true
    }
}

/// Config wrapper that overrides `Serializer::is_human_readable` and
/// `Deserializer::is_human_readable` to return `false`.
#[derive(Copy, Clone, Debug)]
pub struct BinaryConfig<C>(C);

impl<C> BinaryConfig<C> {
    /// Creates a `BinaryConfig` inheriting unchanged configuration options from the given configuration.
    #[inline(always)]
    pub fn new(inner: C) -> Self {
        Self(inner)
    }
}

impl<C> sealed::SerializerConfig for BinaryConfig<C>
where
    C: sealed::SerializerConfig,
{
    #[inline(always)]
    fn is_named(&self) -> bool {
        self.0.is_named()
    }

    #[inline(always)]
    fn is_human_readable(&self) -> bool {
        false
    }
}
