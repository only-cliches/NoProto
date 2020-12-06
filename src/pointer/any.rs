
use crate::pointer::NP_Cursor_Addr;
use crate::{json_flex::{JSMAP}, schema::{NP_Parsed_Schema}};
use alloc::vec::Vec;
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};


use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::{NP_Cursor};
use crate::NP_Memory;

/// Any data type
#[derive(Debug)]
pub struct NP_Any { }

impl NP_Any { }

impl<'value> NP_Value<'value> for NP_Any {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("any", NP_TypeKeys::Any) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("any", NP_TypeKeys::Any) }

    fn schema_to_json(_schema: &Vec<NP_Parsed_Schema>, _address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String("any".to_owned()));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value<'set>(mut cursor: NP_Cursor_Addr, memory: &'set NP_Memory, value: Self) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'set + Sized {
        Err(NP_Error::new("Can't use .set() with (Any), must cast first with NP_Any::cast<T>(pointer)."))
    }
    fn into_value(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<Option<Self>, NP_Error> {
        Err(NP_Error::new("Type (Any) doesn't support .into()!"))
    }
    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {
        NP_JSON::Null
    }
    fn get_size(cursor: NP_Cursor_Addr, _memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {
        Ok(0)
    }
    fn do_compact(from_cursor: &NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: Sized {
        Err(NP_Error::new("Cannot compact an ANY field!"))
    }
    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, _json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Any as u8);
        schema.push(NP_Parsed_Schema::Any {
            i: NP_TypeKeys::Any,
            sortable: false
        });
        return Ok((false, schema_data, schema));

    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, _address: usize, _bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        schema.push(NP_Parsed_Schema::Any {
            i: NP_TypeKeys::Any,
            sortable: false
        });
        (false, schema)
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
