use crate::pointer::NP_ValueInto;
use crate::schema::NP_Schema;
use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::JFObject};
use super::NP_PtrKinds;

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

#[derive(Debug)]
pub struct NP_Any {

}

impl<'a> NP_Any {

    /// Casts a pointer from NP_Any to any other type.
    /// 
    /// 
    pub fn cast<T: NP_Value + Default + NP_ValueInto<'a>>(pointer: NP_Ptr<'a, NP_Any>) -> core::result::Result<NP_Ptr<'a, T>, NP_Error> {

        // schema is "any" type, all casting permitted
        if pointer.schema.type_data.0 == NP_TypeKeys::Any as i64 {
            return Ok(NP_Ptr::new_standard_ptr(pointer.location, &pointer.schema, pointer.memory));
        }

        // schema matches type
        if T::type_idx().0 == pointer.schema.type_data.0 { 
            return Ok(NP_Ptr::new_standard_ptr(pointer.location, &pointer.schema, pointer.memory));
        }

        // schema does not match type
        let mut err = "TypeError: Attempted to cast type (".to_owned();
        err.push_str(T::type_idx().1.as_str());
        err.push_str(") to schema of type (");
        err.push_str(pointer.schema.type_data.1.as_str());
        err.push_str(")");
        Err(NP_Error::new(err))
    }
}

impl<'a> NP_Value for NP_Any {

    fn is_type(type_str: &str) -> bool {
        type_str == "*" || type_str == "any"
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }

    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
}

impl<'a> NP_ValueInto<'a> for NP_Any {
    fn buffer_into(_address: u32, _kind: NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
    fn buffer_to_json(_address: u32, _kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> JFObject {
        JFObject::Null
    }
    fn buffer_get_size(_address: u32, _kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        Ok(0)
    }
    fn buffer_do_compact<X: NP_Value + Default + NP_ValueInto<'a>>(_from_ptr: &NP_Ptr<'a, X>, _to_ptr: NP_Ptr<'a, NP_Any>) -> Result<(u32, NP_PtrKinds, &'a NP_Schema), NP_Error> where Self: NP_Value + Default {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
}

impl Default for NP_Any {
    fn default() -> Self { 
        NP_Any { }
    }
}