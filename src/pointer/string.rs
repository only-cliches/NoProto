use alloc::vec::Vec;
use crate::{json_flex::JSMAP, schema::NP_Schema};
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys, NP_Schema_Ptr}, pointer::NP_Value, utils::from_utf8_lossy, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr, bytes::NP_Bytes};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use alloc::string::ToString;

/// Schema state for String
#[derive(Debug)]
pub struct NP_String_Schema_State {
    /// 0 for dynamic size, anything greater than 0 is for fixed size
    pub size: u16,
    /// The default bytes
    pub default: Option<String>
}

/// Get schema state for string type
pub fn str_get_schema_state(schema_ptr: &NP_Schema_Ptr) -> NP_String_Schema_State {

    // fixed size
    let fixed_size = u16::from_be_bytes([
        schema_ptr.schema.bytes[schema_ptr.address + 1],
        schema_ptr.schema.bytes[schema_ptr.address + 2]
    ]);

    // default value size
    let default_size = u16::from_be_bytes([
        schema_ptr.schema.bytes[schema_ptr.address + 3],
        schema_ptr.schema.bytes[schema_ptr.address + 4]
    ]) as usize;

    if default_size == 0 {
        return NP_String_Schema_State {
            size: fixed_size,
            default: None
        }
    }

    let default_bytes = {
        let bytes = &schema_ptr.schema.bytes[(schema_ptr.address + 5)..(schema_ptr.address + 5 + (default_size - 1))];
        from_utf8_lossy(bytes).to_string()
    };

    return NP_String_Schema_State { size: fixed_size, default: Some(default_bytes) }
}


impl<'str> NP_Value<'str> for String {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::UTF8String as u8, "string".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::UTF8String as u8, "string".to_owned()) }

    fn schema_to_json(schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = str_get_schema_state(&schema_ptr);
        if schema_state.size > 0 {
            schema_json.insert("size".to_owned(), NP_JSON::Integer(schema_state.size.into()));
        }

        if let Some(default) = schema_state.default {
            schema_json.insert("default".to_owned(), NP_JSON::String(default.clone()));
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(pointer: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        let bytes = value.as_bytes().to_vec();
        NP_Bytes::set_value(pointer, Box::new(&NP_Bytes::new(bytes)))
    }

    fn into_value(pointer: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        match NP_Bytes::into_value(pointer)? {
            Some(x) => {
                let bytes = &*x.bytes.to_vec();
                Ok(Some(Box::new(from_utf8_lossy(bytes))))
            },
            None => Ok(None)
        }
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {
        let state = str_get_schema_state(schema);
        match state.default {
            Some(x) => {
                Some(Box::new(String::from(x)))
            },
            None => None
        }
    }

    fn to_json(pointer: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(pointer.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(*y)
                    },
                    None => {
                        let schema_state = str_get_schema_state(&pointer.schema);
                        match schema_state.default {
                            Some(x) => {
                                NP_JSON::String(String::from(x))
                            },
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(pointer: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        NP_Bytes::get_size(pointer)
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if type_str == "string" || type_str == "str" || type_str == "utf8" || type_str == "utf-8" {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::UTF8String as u8);

            match json_schema["size"] {
                NP_JSON::Integer(x) => {
                    if x < 1 {
                        return Err(NP_Error::new("Fixed size for string must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for string cannot be larger than 2^16!"));
                    }
                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                NP_JSON::Float(x) => {
                    if x < 1.0 {
                        return Err(NP_Error::new("Fixed size for string must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for string cannot be larger than 2^16!"));
                    }

                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                }
            }

            match &json_schema["default"] {
                NP_JSON::String(bytes) => {
                    let str_bytes = bytes.clone().into_bytes();
                    if str_bytes.len() > u16::max as usize - 1 {
                        return Err(NP_Error::new("Default string value cannot be larger than 2^16 bytes!"));
                    }
                    schema_data.extend(((str_bytes.len() + 1) as u16).to_be_bytes().to_vec());
                    schema_data.extend(str_bytes);
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                }
            }

            return Ok(Some(schema_data));
        }
        
        Ok(None)
    }
}