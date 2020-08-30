use alloc::vec::Vec;
use crate::schema::NP_Schema_Parser;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys, NP_Schema_Ptr}, pointer::NP_Value, utils::from_utf8_lossy, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr, bytes::NP_Bytes};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};

/// Schema state for String
#[derive(Debug)]
pub struct NP_String_Schema_State<'state> {
    /// 0 for dynamic size, anything greater than 0 is for fixed size
    pub size: u16,
    /// The default bytes
    pub default: &'state str
}

impl NP_Schema_Parser for String {

    fn type_key(&self) -> u8 { NP_TypeKeys::UTF8String as u8 }


    fn from_json_to_state(&self, json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if type_str == "string" || type_str == "str" || type_str == "utf8" || type_str == "utf-8" {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Bytes as u8);

            match json_schema["size"] {
                NP_JSON::Integer(x) => {
                    if x < 1 {
                        return Err(NP_Error::new("Fixed size for bytes must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for bytes cannot be larger than 2^16!"));
                    }
                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                NP_JSON::Float(x) => {
                    if x < 1.0 {
                        return Err(NP_Error::new("Fixed size for bytes must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for bytes cannot be larger than 2^16!"));
                    }

                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                }
            }

            match json_schema["default"] {
                NP_JSON::String(bytes) => {
                    let length = bytes.len() as u16;
                    schema_data.extend(length.to_be_bytes().to_vec());
                    schema_data.extend(bytes.into_bytes());
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

pub fn str_get_schema_state<'state>(schema_ptr: &'state NP_Schema_Ptr) -> NP_String_Schema_State<'state> {

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

    let default_bytes: &str = if default_size > 0 {
        let bytes = &schema_ptr.schema.bytes[(schema_ptr.address + 5)..(schema_ptr.address + 5 + default_size)];
        from_utf8_lossy(bytes).as_str()
    } else {
        ""
    };

    return NP_String_Schema_State { size: fixed_size, default: default_bytes }
}


impl NP_Value for String {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::UTF8String as u8, "string".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::UTF8String as u8, "string".to_owned()) }

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
        if state.default.len() > 0 {
            Some(Box::new(String::from(state.default)))
        } else {
            None
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
                        if schema_state.default.len() > 0 {
                            NP_JSON::String(String::from(schema_state.default))
                        } else {
                            NP_JSON::Null
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
}