use std::rc::Rc;
use std::{string::FromUtf8Error, cell::{BorrowError, BorrowMutError}};
use crate::memory::NP_Memory;

#[derive(Debug)]
pub struct NP_Error {
    message: String
}

impl NP_Error {
    pub fn new(message: &str) -> Self {
        NP_Error { message: message.to_owned() }
    }
}

impl From<BorrowMutError> for NP_Error {
    fn from(err: BorrowMutError) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}

impl From<BorrowError> for NP_Error {
    fn from(err: BorrowError) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}

impl From<FromUtf8Error> for NP_Error {
    fn from(err: FromUtf8Error) -> NP_Error {
        NP_Error::new(err.to_string().as_str())
    }
}

impl From<Rc<std::cell::RefCell<NP_Memory>>> for NP_Error {
    fn from(_err: Rc<std::cell::RefCell<NP_Memory>>) -> NP_Error {
        NP_Error::new("Reference Count Error, value still being borrowed!")
    }
}