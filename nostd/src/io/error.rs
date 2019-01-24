use core::{result, fmt};
use core::fmt::{Display, Formatter};
use error;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    reason: &'static str,
}

impl Error {
    pub fn new(reason: &'static str) -> Self {
        Error {
            reason: reason,
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.reason
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
        error::Error::description(self).fmt(f)
    }
}

#[cfg(feature = "std")]
impl From<::std::io::Error> for Error {
    fn from(_err: ::std::io::Error) -> Self {
        return ::io::Error { reason: "IO Error" };
    }
}

// Note(chbeck): Added this because rmp-serde relies on io::error::ErrorKind
// This is a redux of what existed in rust-sgx-sdk io::error::ErrorKind

/// A list specifying general categories of I/O error.
///
/// This list is intended to grow over time and it is not recommended to
/// exhaustively match against it.
///
/// It is used with the [`io::Error`] type.
///
/// [`io::Error`]: struct.Error.html
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(deprecated)]
pub enum ErrorKind {
    /// An error returned when an operation could not be completed because an
    /// "end of file" was reached prematurely.
    ///
    /// This typically means that an operation could only succeed if it read a
    /// particular number of bytes but only a smaller number of bytes could be
    /// read.
    UnexpectedEof,

    /// A marker variant that tells the compiler that users of this enum cannot
    /// match it exhaustively.
    #[doc(hidden)]
    __Nonexhaustive,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::UnexpectedEof => "unexpected end of file",
            ErrorKind::__Nonexhaustive => unreachable!()
        }
    }
}

/// Intended for use for errors not exposed to the user, where allocating onto
/// the heap (for normal construction via Error::new) is too costly.
impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error {
            reason: kind.as_str()
        }
    }
}
