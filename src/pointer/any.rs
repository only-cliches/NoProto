use alloc::rc::Rc;
use crate::schema::NP_Schema;
use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_Lite_Ptr, NP_PtrKinds};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

/// Any data type
#[derive(Debug)]
pub struct NP_Any {

}

impl<'a> NP_Any {

    /// Casts a pointer from NP_Any to any other type.
    /// 
    /// 
    pub fn cast<T: NP_Value + Default>(pointer: NP_Ptr<NP_Any>) -> Result<NP_Ptr<T>, NP_Error> {

        // schema is "any" type, all casting permitted
        if pointer.schema.type_data.0 == NP_TypeKeys::Any as i64 {
            return Ok(NP_Ptr::_new_standard_ptr(pointer.location, pointer.schema, pointer.memory));
        }

        // schema matches type
        if T::type_idx().0 == pointer.schema.type_data.0 { 
            return Ok(NP_Ptr::_new_standard_ptr(pointer.location, pointer.schema, pointer.memory));
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

impl NP_Value for NP_Any {

    fn is_type(type_str: &str) -> bool {
        type_str == "*" || type_str == "any"
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Any as i64, "any".to_owned()) }

    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
    fn into_value(_pointer: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
    fn to_json(_pointer: NP_Lite_Ptr) -> NP_JSON {
        NP_JSON::Null
    }
    fn get_size(_pointer: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        Ok(0)
    }
    fn do_compact(_from_ptr: NP_Lite_Ptr, _to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value + Default {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
}

impl Default for NP_Any {
    fn default() -> Self { 
        NP_Any { }
    }
}