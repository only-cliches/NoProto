
use crate::{json_flex::{JSMAP}, schema::{NP_Parsed_Schema}};
use alloc::vec::Vec;
use crate::error::NP_Error;
use crate::{schema::{NP_Schema, NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::{NP_Cursor_Addr};
use crate::NP_Memory;

/// Any data type
#[derive(Debug)]
pub struct NP_Any { }

impl NP_Any { }

impl<'value> NP_Value<'value> for NP_Any {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("any", NP_TypeKeys::Any) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("any", NP_TypeKeys::Any) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String("any".to_owned()));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
    fn into_value<'into>(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> Result<Option<&'value Self>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
    fn to_json(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> NP_JSON {
        NP_JSON::Null
    }
    fn get_size(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> Result<usize, NP_Error> {
        Ok(0)
    }
    fn do_compact(from_cursor: NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &'value NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: NP_Value<'value> {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "any" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Any as u8);
            schema.push(NP_Parsed_Schema::Any {
                i: NP_TypeKeys::Any,
                sortable: false
            });
            return Ok(Some((schema_data, schema)));
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<&'value Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        schema.push(NP_Parsed_Schema::Any {
            i: NP_TypeKeys::Any,
            sortable: false
        });
        schema
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
