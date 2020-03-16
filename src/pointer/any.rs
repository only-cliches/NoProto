use crate::schema::NoProtoSchema;
use crate::pointer::NoProtoPointer;
use crate::error::NoProtoError;
use crate::memory::NoProtoMemory;
use std::{cell::RefCell, rc::Rc};
use crate::{schema::NoProtoTypeKeys, pointer::NoProtoValue};
use super::NoProtoPointerKinds;

#[derive(Debug)]
pub struct NoProtoAny {

}
/*
impl<'a> NoProtoAny {

    pub fn cast<T: NoProtoValue<'a> + Default>(pointer: NoProtoPointer<NoProtoAny>) -> std::result::Result<NoProtoPointer<'a, T>, NoProtoError> {

        // schema is "any" type, all casting permitted
        if pointer.schema.type_data.0 == NoProtoTypeKeys::Any as i64 {
            return Ok(NoProtoPointer::new_standard_ptr(pointer.address, &pointer.schema, pointer.memory)?);
        }

        // schema matches type
        if T::type_idx().0 == pointer.schema.type_data.0 { 
            return Ok(NoProtoPointer::new_standard_ptr(pointer.address, &pointer.schema, pointer.memory)?);
        }

        // schema does not match type
        Err(NoProtoError::new(format!("TypeError: Attempted to cast type ({}) to schema of type ({})!", T::type_idx().1, pointer.schema.type_data.1).as_str()))
    }
}*/

impl<'a> NoProtoValue<'a> for NoProtoAny {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        NoProtoAny { }
    }

    fn is_type(type_str: &str) -> bool {
        type_str == "*" || type_str == "any"
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Any as i64, "any".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Any as i64, "any".to_owned()) }

    fn buffer_get(_address: u32, _kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        Err(NoProtoError::new("Can't .get() from ANY value, must cast first with NoProtoAny::cast<T>(pointer). dd"))
    }

    fn buffer_set(_address: u32, _kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>, _value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {
        Err(NoProtoError::new("Can't .set() to ANY value, must cast first with NoProtoAny::cast<T>(pointer)."))
    }
}

impl Default for NoProtoAny {
    fn default() -> Self { 
        NoProtoAny { }
    }
}