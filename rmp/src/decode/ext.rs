
/// Extension type meta information.
///
/// Extension represents a tuple of type information and a byte array where type information is an
/// integer whose meaning is defined by applications.
///
/// Applications can assign 0 to 127 to store application-specific type information.
///
/// # Note
///
/// MessagePack reserves -1 to -128 for future extension to add predefined types which will be
/// described in separated documents.
#[derive(Debug, PartialEq)]
pub struct ExtMeta {
    /// Type information.
    pub typeid: i8,
    /// Byte array size.
    pub size: u32,
}
