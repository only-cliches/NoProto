use crate::pointer::NoProtoPointer;
use crate::error::NoProtoError;
use crate::memory::NoProtoMemory;
use std::{cell::RefCell, rc::Rc};
use crate::pointer::NoProtoValue;
use super::NoProtoPointerKinds;

pub struct NoProtoAny {

}

impl NoProtoAny {

    pub fn cast<T: NoProtoValue + Default>(pointer: NoProtoPointer<NoProtoAny>) -> std::result::Result<NoProtoPointer<T>, NoProtoError> {

        if T::type_idx() == pointer.schema.value.self_type_idx() { // schema matches type
            Err(NoProtoError::new("WRONG TYPE!"))
        } else { // schema does not match type
            Err(NoProtoError::new("WRONG TYPE!"))
        }
    }
}

impl NoProtoValue for NoProtoAny {

    fn new() -> Self {
        NoProtoAny { }
    }

    fn is_type(&self, type_str: &str) -> bool {
        false
    }

    fn type_idx() -> (i64, &'static str) { (0, "any") }
    fn self_type_idx(&self) -> (i64, &'static str) { (0, "any") }

    fn buffer_read(&mut self, address: u32, kind: &NoProtoPointerKinds, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<NoProtoAny>, NoProtoError> {
        Err(NoProtoError::new("Can't read from ANY value, must cast first with NoProtoAny::cast<T>(pointer)!"))
    }

    fn buffer_write(&mut self, address: u32, kind: &NoProtoPointerKinds, buffer: Rc<RefCell<NoProtoMemory>>, value: Vec<u8>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {
        Err(NoProtoError::new("Can't write to ANY value, must cast first with NoProtoAny::cast<T>(pointer)!"))
    }
}

impl Default for NoProtoAny {
    fn default() -> Self { 
        NoProtoAny { }
    }
}