
use crate::{json_flex::{JSMAP}, schema::{NP_Parsed_Schema}};
use alloc::vec::Vec;
use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::{schema::{NP_Schema, NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

/// Any data type
#[derive(Debug)]
pub struct NP_Any { }

impl NP_Any { }

impl<'value> NP_Value<'value> for NP_Any {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Any as u8, "any".to_owned(), NP_TypeKeys::Any) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Any as u8, "any".to_owned(), NP_TypeKeys::Any) }

    fn schema_to_json(_schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String("any".to_owned()));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_ptr: &mut NP_Ptr<'value>, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
    fn into_value<'into>(_ptr: &'into NP_Ptr<'into>) -> Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
    fn to_json(_ptr: &'value NP_Ptr<'value>) -> NP_JSON {
        NP_JSON::Null
    }
    fn get_size(_ptr: &'value NP_Ptr<'value>) -> Result<usize, NP_Error> {
        Ok(0)
    }
    fn do_compact(_from_ptr: NP_Ptr<'value>, _to_ptr: &mut NP_Ptr<'value>) -> Result<(), NP_Error> where Self: NP_Value<'value> + Default {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
    fn from_json_to_schema(json_schema: &NP_JSON)-> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "any" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Any as u8);
            return Ok(Some((schema_data, NP_Parsed_Schema::Any {
                i: NP_TypeKeys::Any,
                sortable: false
            })));
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }

    fn from_bytes_to_schema(_address: usize, _bytes: &Vec<u8>) -> NP_Parsed_Schema {
        NP_Parsed_Schema::Any {
            i: NP_TypeKeys::Any,
            sortable: false
        }
    }
}

impl<'value> Default for NP_Any {
    fn default() -> Self { 
        NP_Any {}
    }
}


#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"any\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}
