
use crate::{schema::NP_Schema_Ptr, json_flex::{JSMAP}};
use alloc::vec::Vec;
use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::{schema::{NP_Schema, NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_Lite_Ptr, NP_PtrKinds};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

/// Any data type
#[derive(Debug)]
pub struct NP_Any {

}

impl NP_Any {

    /// Casts a pointer from NP_Any to any other type.
    /// 
    /// 
    pub fn cast<'any, T: NP_Value<'any> + Default>(pointer: NP_Ptr<'any, NP_Any>) -> Result<NP_Ptr<'any, T>, NP_Error> {

        let this_type = pointer.schema.schema.bytes[pointer.schema.address];

        // schema is "any" type, all casting permitted
        if this_type == NP_TypeKeys::Any as u8 {
            return Ok(NP_Ptr::_new_standard_ptr(pointer.location, pointer.schema, pointer.memory));
        }

        // schema matches type
        if T::type_idx().0 == this_type { 
            return Ok(NP_Ptr::_new_standard_ptr(pointer.location, pointer.schema, pointer.memory));
        }

        // schema does not match type
        let mut err = "TypeError: Attempted to cast type (".to_owned();
        err.push_str(T::type_idx().1.as_str());
        err.push_str(") to schema of type (");
        err.push_str(NP_TypeKeys::from(this_type).into_type_idx().1.as_str());
        err.push_str(")");
        Err(NP_Error::new(err))
    }
}

impl<'any> NP_Value<'any> for NP_Any {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Any as u8, "any".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Any as u8, "any".to_owned()) }

    fn schema_to_json(_schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String("any".to_owned()));

        Ok(NP_JSON::Dictionary(schema_json))
    }

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
    fn do_compact(_from_ptr: NP_Lite_Ptr, _to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value<'any> + Default {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<NP_Schema>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "any" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Any as u8);
            return Ok(Some(NP_Schema { is_sortable: false, bytes: schema_data}))
        }

        Ok(None)
    }
}

impl<'any> Default for NP_Any {
    fn default() -> Self { 
        NP_Any {}
    }
}