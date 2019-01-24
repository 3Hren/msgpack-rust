use core::fmt::{Debug, Display};
use core::str::Utf8Error;

pub trait Error: Debug + Display {
    fn description(&self) -> &str;
    fn cause(&self) -> Option<&Error> { None }
}

impl Error for Utf8Error {
    fn description(&self) -> &str {
        "Utf8Error"
    }

    fn cause(&self) -> Option<&Error> {
        None
    }
}
