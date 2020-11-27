//! Represents the string value of a choice in a schema
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::option::NP_Option;
//! use no_proto::here;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "option",
//!    "choices": ["red", "green", "blue"]
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set(here(), NP_Option::new("green"))?;
//! 
//! assert_eq!(Box::new(NP_Option::new("green")), new_buffer.get::<NP_Option>(here())?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::{memory::NP_Memory, schema::{NP_Parsed_Schema}};
use alloc::vec::Vec;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug}, hint::unreachable_unchecked};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{string::ToString};
use super::{NP_Cursor_Addr};

/// Holds Enum / Option type data.
/// 
/// Check out documentation [here](../option/index.html).
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NP_Option<'opt> {
    /// The value of this option type
    pub value: Option<&'opt str>
}

impl<'opt> NP_Option<'opt> {
    /// Create a new option type with the given string
    pub fn new(value: &'opt str) -> Self {
        NP_Option {
            value: Some(value)
        }
    }

    /// Create a new empty option type
    pub fn empty() -> Self {
        NP_Option {
            value: None
        }
    }
    
    /// Set the value of this option type
    pub fn set(&mut self, value: Option<&'opt str>) {
        self.value = value;
    }
}

impl<'opt> Default for NP_Option<'opt> {
    fn default() -> Self { 
        NP_Option { value: None }
     }
}

impl<'value> NP_Value<'value> for NP_Option<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("option", NP_TypeKeys::Enum) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("option", NP_TypeKeys::Enum) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        match &schema[address] {
            NP_Parsed_Schema::Enum { i: _, choices, default, sortable: _} => {

                let options: Vec<NP_JSON> = choices.into_iter().map(|value| {
                    NP_JSON::String(value.to_string())
                }).collect();
            
                if let Some(d) = default {
                    schema_json.insert("default".to_owned(), options[**d as usize].clone());
                }
        
                schema_json.insert("choices".to_owned(), NP_JSON::Array(options));
            },
            _ => { unsafe { unreachable_unchecked() } }
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Box<Self>> {

        match schema {
            NP_Parsed_Schema::Enum { i: _, choices, default, sortable: _} => {
                if let Some(d) = default {
                    Some(Box::new(NP_Option::new(choices[**d as usize])))
                } else {
                    None
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor_addr.is_virtual { panic!() }

        match &cursor.schema_data(&memory) {
            NP_Parsed_Schema::Enum { i: _, choices, default: _, sortable: _} => {
                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;
        
                    for opt in choices {
                        if value.value == Some(opt) {
                            value_num = ct as i32;
                        }
                        ct += 1;
                    };
        
                    if value_num == -1 {
                        return Err(NP_Error::new("Option not found, cannot set uknown option!"));
                    }
                }
        
                let bytes = value_num as u8;
        
                if cursor.address_value != 0 { // existing value, replace
        
                    let write_bytes = memory.write_bytes();
        
                    write_bytes[cursor.address_value] = bytes;
                    return Ok(cursor_addr);
        
                } else { // new value
        
                    cursor.address_value = memory.malloc_borrow(&[bytes])?;

                    memory.set_value_address(cursor.address, cursor.address_value);
                    return Ok(cursor_addr);
                }     
            },
            _ => { unsafe { unreachable_unchecked() } }
        }               
    }

    fn into_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> Result<Option<Box<Self>>, NP_Error> {

        let cursor = cursor_addr.get_data(&memory).unwrap();

        // empty value
        if cursor.address_value == 0 {
            return Ok(None);
        }
  
        match &cursor.schema_data(&memory) {
            NP_Parsed_Schema::Enum { i: _, choices, default: _, sortable: _} => {
                Ok(match memory.get_1_byte(cursor.address_value) {
                    Some(x) => {
                        let value_num = u8::from_be_bytes([x]) as usize;
        
                        if value_num > choices.len() {
                            None
                        } else {
                            Some(Box::new(NP_Option { value: Some(choices[value_num].to_string().as_str()) }))
                        }
                    },
                    None => None
                })
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn to_json(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> NP_JSON {

        match Self::into_value(cursor_addr, memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        match y.value {
                            Some(str_value) => {
                                NP_JSON::String(str_value.to_string())
                            },
                            None => {
                                let cursor = cursor_addr.get_data(&memory).unwrap();
                                match &cursor.schema_data(&memory) {
                                    NP_Parsed_Schema::Enum { i: _, choices, default, sortable: _} => {
                                        if let Some(d) = default {
                                            NP_JSON::String(choices[**d as usize].to_string())
                                        } else {
                                            NP_JSON::Null
                                        }
                                    },
                                    _ => { unsafe { unreachable_unchecked() } }
                                }
                            }
                        }
                    },
                    None => {
                        let cursor = cursor_addr.get_data(&memory).unwrap();
                        match &cursor.schema_data(&memory) {
                            NP_Parsed_Schema::Enum { i: _, choices, default, sortable: _} => {
                                if let Some(d) = default {
                                    NP_JSON::String(choices[**d as usize].to_string())
                                } else {
                                    NP_JSON::Null
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

    fn get_size(cursor_addr: NP_Cursor_Addr, memory: NP_Memory) -> Result<usize, NP_Error> {
        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>())
        }
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "option" == type_str || "enum" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Enum as u8);

            let mut choices: Vec<&str> = Vec::new();

            let mut default_stir: Option<String> = None;

            match &json_schema["default"] {
                NP_JSON::String(def) => {
                    default_stir = Some(def.clone());
                },
                _ => {}
            }

            let mut default_index: Option<Box<u8>> = None;

            match &json_schema["choices"] {
                NP_JSON::Array(x) => {
                    for opt in x {
                        match opt {
                            NP_JSON::String(stir) => {
                                if stir.len() > 255 {
                                    return Err(NP_Error::new("'option' choices cannot be longer than 255 characters each!"))
                                }

                                if let Some(def) = &default_stir {
                                    if def == stir {
                                        default_index = Some(Box::new(choices.len() as u8));
                                    }
                                }
                                choices.push(stir.as_str());
                            },
                            _ => {}
                        }
                    }
                },
                _ => {
                    return Err(NP_Error::new("'option' type requires a 'choices' key with an array of strings!"))
                }
            }

            if choices.len() > 254 {
                return Err(NP_Error::new("'option' type cannot have more than 254 choices!"))
            }

            // default value
            match &default_index {
                Some(x) => schema_data.push(**x + 1),
                None => schema_data.push(0)
            }

            // choices
            schema_data.push(choices.len() as u8);
            for choice in &choices {
                schema_data.push(choice.len() as u8);
                schema_data.extend(choice.as_bytes().to_vec())
            }

            schema.push(NP_Parsed_Schema::Enum { 
                i: NP_TypeKeys::Enum,
                default: default_index,
                choices: choices,
                sortable: true
            });

            return Ok(Some((schema_data, schema)));
        }
        
        Ok(None)
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        let mut default_index: Option<Box<u8>> = None;

        if bytes[address + 1] > 0 {
            default_index = Some(Box::new(bytes[address + 1] - 1));
        }

        let choices_len = bytes[address + 2];

        let mut choices: Vec<&str> = Vec::new();
        let mut offset: usize = address + 3;
        for _x in 0..choices_len {
            let choice_size = bytes[offset] as usize;
            let choice_bytes = &bytes[(offset + 1)..(offset + 1 + choice_size)];
            choices.push(unsafe { core::str::from_utf8_unchecked(choice_bytes) });
            offset += 1 + choice_size;
        }

        schema.push(NP_Parsed_Schema::Enum {
            i: NP_TypeKeys::Enum,
            sortable: true,
            default: default_index,
            choices: choices
        });

        schema
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"option\",\"default\":\"hello\",\"choices\":[\"hello\",\"world\"]}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"option\",\"choices\":[\"hello\",\"world\"]}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"option\",\"default\":\"hello\",\"choices\":[\"hello\",\"world\"]}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get(&[])?.unwrap(), Box::new(NP_Option::new("hello")));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"option\",\"choices\":[\"hello\",\"world\"]}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&[], NP_Option::new("hello"))?;
    assert_eq!(buffer.get::<NP_Option>(&[])?, Some(Box::new(NP_Option::new("hello"))));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<NP_Option>(&[])?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}