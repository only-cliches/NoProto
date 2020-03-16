use crate::pointer::NP_ValueInto;
use crate::schema::NP_Schema;
use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use std::{cell::RefCell, rc::Rc};
use crate::{schema::NP_TypeKeys, pointer::NP_Value};
use super::NP_PtrKinds;

#[derive(Debug)]
pub struct NP_Any {

}

impl<'a> NP_Any {

    pub fn cast<T: NP_Value + Default + NP_ValueInto<'a>>(pointer: NP_Ptr<'a, NP_Any>) -> std::result::Result<NP_Ptr<'a, T>, NP_Error> {

        // schema is "any" type, all casting permitted
        if pointer.schema.type_data.0 == NP_TypeKeys::Any as i64 {
            return Ok(NP_Ptr::new_standard_ptr(pointer.address, &pointer.schema, pointer.memory)?);
        }

        // schema matches type
        if T::type_idx().0 == pointer.schema.type_data.0 { 
            return Ok(NP_Ptr::new_standard_ptr(pointer.address, &pointer.schema, pointer.memory)?);
        }

        // schema does not match type
        Err(NP_Error::new(format!("TypeError: Attempted to cast type ({}) to schema of type ({})!", T::type_idx().1, pointer.schema.type_data.1).as_str()))
    }
}

impl<'a> NP_Value for NP_Any {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Any { }
    }

    fn is_type(type_str: &str) -> bool {
        type_str == "*" || type_str == "any"
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }

    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Can't use .get() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }

    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: Rc<RefCell<NP_Memory>>, _value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
}

impl<'a> NP_ValueInto<'a> for NP_Any {
    fn buffer_into(_address: u32, _kind: NP_PtrKinds, _schema: &'a NP_Schema, _buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
}

impl Default for NP_Any {
    fn default() -> Self { 
        NP_Any { }
    }
}