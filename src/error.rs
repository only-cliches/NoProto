//! Primary error type used by the library

use alloc::string::FromUtf8Error;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::string::ToString;

#[derive(Debug)]
pub struct NP_Error {
    pub message: String
}

impl NP_Error {
    pub fn new<S: AsRef<str>>(message: S) -> Self {
        NP_Error { message: message.as_ref().to_owned() }
    }
    pub fn unwrap<T>(value: Option<T>) -> Result<T, NP_Error> {
        match value {
            Some(x) => Ok(x),
            None => Err(NP_Error::new("Missing Value in option!"))
        }
    }
}

impl From<FromUtf8Error> for NP_Error {
    fn from(err: FromUtf8Error) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}

impl From<core::num::ParseFloatError> for NP_Error {
    fn from(err: core::num::ParseFloatError) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}

impl From<core::num::ParseIntError> for NP_Error {
    fn from(err: core::num::ParseIntError) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}