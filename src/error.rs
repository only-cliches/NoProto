use std::rc::Rc;
use std::{string::FromUtf8Error, cell::{BorrowError, BorrowMutError}};
use crate::memory::NoProtoMemory;

#[derive(Debug)]
pub struct NoProtoError {
    message: String
}

impl NoProtoError {
    pub fn new(message: &str) -> Self {
        NoProtoError { message: message.to_owned() }
    }
}

impl From<BorrowMutError> for NoProtoError {
    fn from(err: BorrowMutError) -> NoProtoError {
        NoProtoError::new(err.to_string().as_str())
    }
}

impl From<BorrowError> for NoProtoError {
    fn from(err: BorrowError) -> NoProtoError {
        NoProtoError::new(err.to_string().as_str())
    }
}

impl From<FromUtf8Error> for NoProtoError {
    fn from(err: FromUtf8Error) -> NoProtoError {
        NoProtoError::new(err.to_string().as_str())
    }
}

impl From<Rc<std::cell::RefCell<NoProtoMemory>>> for NoProtoError {
    fn from(_err: Rc<std::cell::RefCell<NoProtoMemory>>) -> NoProtoError {
        NoProtoError::new("Reference Count Error, value still being borrowed!")
    }
}