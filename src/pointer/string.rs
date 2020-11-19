//! NoProto supports Rust's native UTF8 [`String`](https://doc.rust-lang.org/std/string/struct.String.html) type.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set("", String::from("I want to play a game"))?;
//! 
//! assert_eq!(Box::new(String::from("I want to play a game")), new_buffer.get::<String>("")?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```


use core::hint::unreachable_unchecked;

use alloc::vec::Vec;
use crate::{json_flex::JSMAP, memory::NP_Size, schema::{NP_Parsed_Schema, NP_Schema}};
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys}, pointer::NP_Value, utils::from_utf8_lossy, json_flex::NP_JSON};
use super::{NP_Ptr, bytes::NP_Bytes};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use alloc::string::ToString;

/// Schema state for String
#[derive(Debug)]
#[doc(hidden)]
pub struct NP_String_Schema_State {
    /// 0 for dynamic size, anything greater than 0 is for fixed size
    pub size: u16,
    /// The default bytes
    pub default: Option<String>
}


impl<'str> NP_Value<'str> for String {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }

    fn schema_to_json(schema: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {

        match schema {
            NP_Parsed_Schema::UTF8String { i: _, size, default, sortable: _ } => {
                let mut schema_json = JSMAP::new();
                schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));
        
                if *size > 0 {
                    schema_json.insert("size".to_owned(), NP_JSON::Integer(size.clone().into()));
                }
        
                if let Some(default_value) = default {
                    schema_json.insert("default".to_owned(), NP_JSON::String(*default_value.clone()));
                }
        
                Ok(NP_JSON::Dictionary(schema_json))
            },
            _ => {
                unsafe { unreachable_unchecked() }
            }
        }

    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
        // fixed size
        let fixed_size = u16::from_be_bytes([
            bytes[address + 1],
            bytes[address + 2]
        ]);

        // default value size
        let default_size = u16::from_be_bytes([
            bytes[address + 3],
            bytes[address + 4]
        ]) as usize;

        if default_size == 0 {
            return NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                default: None,
                sortable: fixed_size > 0,
                size: fixed_size
            }
        }

        let default_bytes = {
            let bytes = &bytes[(address + 5)..(address + 5 + (default_size - 1))];
            from_utf8_lossy(bytes).to_string()
        };

        return NP_Parsed_Schema::UTF8String {
            i: NP_TypeKeys::UTF8String,
            default: Some(Box::new(default_bytes)),
            size: fixed_size,
            sortable: fixed_size > 0
        }
    }

    fn set_value(pointer: &mut NP_Ptr<'str>, value: Box<&Self>) -> Result<(), NP_Error> {
        let bytes = value.as_bytes().to_vec();
        NP_Bytes::set_value(pointer, Box::new(&NP_Bytes::new(bytes)))
    }

    fn into_value(pointer: NP_Ptr<'str>) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = pointer.kind.get_value_addr() as usize;
 
        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = pointer.memory;

        match &**pointer.schema {
            NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
                if *size > 0 { // fixed size
            
                    let size = *size as usize;
                    
                    // get bytes
                    let bytes = &memory.read_bytes()[(addr)..(addr+size)];
        
                    return Ok(Some(Box::new(from_utf8_lossy(bytes))))
        
                } else { // dynamic size
                    // get size of bytes
        
                    let bytes_size: usize = memory.read_address(addr);
        
                    // get bytes
                    let bytes = match memory.size {
                        NP_Size::U8 => { &memory.read_bytes()[(addr+1)..(addr+1+bytes_size)] },
                        NP_Size::U16 => { &memory.read_bytes()[(addr+2)..(addr+2+bytes_size)] },
                        NP_Size::U32 => { &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)] }
                    };
        
                    return Ok(Some(Box::new(from_utf8_lossy(bytes))))
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        match schema {
            NP_Parsed_Schema::UTF8String { i: _, size: _, default, sortable: _ } => {
                match default {
                    Some(x) => Some(Box::new(*x.clone())),
                    None => None
                }
            },
            _ => { panic!() }
        }
    }

    fn to_json(pointer: &'str NP_Ptr<'str>) -> NP_JSON {
        let this_string = Self::into_value(pointer.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(*y)
                    },
                    None => {
                        match &**pointer.schema {
                            NP_Parsed_Schema::UTF8String { i: _, size: _, default, sortable: _ } => {
                                match default {
                                    Some(x) => NP_JSON::String(*x.clone()),
                                    None => NP_JSON::Null
                                }
                            },
                            _ => { unsafe { unreachable_unchecked() } }
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(pointer: &'str NP_Ptr<'str>) -> Result<usize, NP_Error> {
        let value = pointer.kind.get_value_addr();

        // empty value
        if value == 0 {
            return Ok(0)
        }
        
        // get size of bytes
        let addr = value as usize;        
        let memory = pointer.memory;

        match &**pointer.schema {
            NP_Parsed_Schema::UTF8String { i: _, size, default: _, sortable: _ } => {
                // fixed size
                if *size > 0 { 
                    return Ok(*size as usize)
                }

                // dynamic size
                let bytes_size = memory.read_address(addr) + memory.addr_size_bytes();
                
                // return total size of this string
                return Ok(bytes_size);
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if type_str == "string" || type_str == "str" || type_str == "utf8" || type_str == "utf-8" {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::UTF8String as u8);

            let mut has_fixed_size = false;

            let size = match json_schema["size"] {
                NP_JSON::Integer(x) => {
                    has_fixed_size = true;
                    if x < 1 {
                        return Err(NP_Error::new("Fixed size for string must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for string cannot be larger than 2^16!"));
                    }
                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                    x as u16
                },
                NP_JSON::Float(x) => {
                    has_fixed_size = true;
                    if x < 1.0 {
                        return Err(NP_Error::new("Fixed size for string must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for string cannot be larger than 2^16!"));
                    }

                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                    x as u16
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                    0u16
                }
            };

            let default = match &json_schema["default"] {
                NP_JSON::String(bytes) => {
                    let str_bytes = bytes.clone().into_bytes();
                    if str_bytes.len() > u16::max as usize - 1 {
                        return Err(NP_Error::new("Default string value cannot be larger than 2^16 bytes!"));
                    }
                    schema_data.extend(((str_bytes.len() + 1) as u16).to_be_bytes().to_vec());
                    schema_data.extend(str_bytes);
                    Some(Box::new(bytes.clone()))
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                    None
                }
            };

            return Ok(Some((schema_data, NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                size: size,
                default: default,
                sortable: has_fixed_size
            })));
        }
        
        Ok(None)
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"default\":\"hello\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\",\"size\":10}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"default\":\"hello\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get("")?.unwrap(), Box::new(String::from("hello")));

    Ok(())
}

#[test]
fn fixed_size_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"size\": 20}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set("", String::from("hello there this sentence is long"))?;
    assert_eq!(buffer.get("")?.unwrap(), Box::new(String::from("hello there this sen")));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set("", String::from("hello there this sentence is long"))?;
    assert_eq!(buffer.get::<String>("")?.unwrap(), Box::new(String::from("hello there this sentence is long")));
    buffer.del("")?;
    assert_eq!(buffer.get::<String>("")?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}