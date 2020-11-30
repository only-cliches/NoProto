//! Represents the string value of a choice in a schema
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::option::NP_Enum;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "option",
//!    "choices": ["red", "green", "blue"]
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set(&[], NP_Enum::new("green"))?;
//! 
//! assert_eq!(NP_Enum::new("green"), new_buffer.get::<NP_Enum>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::{memory::NP_Memory, schema::{NP_Parsed_Schema}};
use alloc::vec::Vec;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug}, hint::unreachable_unchecked};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{string::ToString};
use super::{NP_Cursor};

/// Holds Enum / Option type data.
/// 
/// Check out documentation [here](../option/index.html).
/// 
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NP_Enum {
    /// No value
    None,
    /// Value
    Some(String)
}

impl super::NP_Scalar for NP_Enum {}

impl NP_Enum {
    /// Create a new option type with the given string
    pub fn new<S: Into<String>>(value: S) -> Self {
        NP_Enum::Some(value.into())
    }

    /// get length of value
    pub fn len(&self) -> usize {
        match self {
            NP_Enum::None => 0,
            NP_Enum::Some(x) => x.len()
        }
    }

    /// get value as bytes
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            NP_Enum::None => &[],
            NP_Enum::Some(x) => x.as_bytes()
        }
    }

    /// get string of value
    pub fn to_string(&self) -> String {
        match self {
            NP_Enum::None => String::from(""),
            NP_Enum::Some(x) => x.clone()
        }
    }
}

impl Default for NP_Enum {
    fn default() -> Self { 
        NP_Enum::None
     }
}

impl<'value> NP_Value<'value> for NP_Enum {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("option", NP_TypeKeys::Enum) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("option", NP_TypeKeys::Enum) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        match &schema[address] {
            NP_Parsed_Schema::Enum { i: _, choices, default, sortable: _} => {

                let options: Vec<NP_JSON> = choices.into_iter().map(|value| {
                    NP_JSON::String(value.to_string())
                }).collect();
            
                if let Some(d) = default {
                    if let NP_Enum::Some(x) = &d {
                        schema_json.insert("default".to_owned(), NP_JSON::String(x.to_string()));
                    }
                }
        
                schema_json.insert("choices".to_owned(), NP_JSON::Array(options));
            },
            _ => { unsafe { unreachable_unchecked() } }
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Self> {

        match schema {
            NP_Parsed_Schema::Enum { i: _, choices: _, default, sortable: _} => {
                if let Some(d) = default {
                    Some(d.clone())
                } else {
                    None
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn set_value(mut cursor: NP_Cursor, memory: &NP_Memory, value: Self) -> Result<NP_Cursor, NP_Error> {

        assert_ne!(cursor.buff_addr, 0);

        match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Enum { i: _, choices, default: _, sortable: _} => {

                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;
        
                    for opt in choices {
                        if *opt == value {
                            value_num = ct as i32;
                        }
                        ct += 1;
                    };
        
                    if value_num == -1 {
                        return Err(NP_Error::new("Option not found, cannot set uknown option!"));
                    }
                }
        
                let bytes = value_num as u8;

                let mut addr_value = cursor.value.get_value_address();
        
                if addr_value != 0 { // existing value, replace
        
                    let write_bytes = memory.write_bytes();
        
                    write_bytes[addr_value] = bytes;
                    return Ok(cursor);
        
                } else { // new value
        
                    addr_value = memory.malloc_borrow(&[bytes])?;
                    memory.write_address(cursor.buff_addr, addr_value);
                    cursor.value = cursor.value.update_value_address(addr_value);

                    return Ok(cursor);
                }     
            },
            _ => { unsafe { unreachable_unchecked() } }
        }               
    }

    fn into_value(cursor: NP_Cursor, memory: &NP_Memory) -> Result<Option<Self>, NP_Error> {

        let value_addr = cursor.value.get_value_address();

        // empty value
        if value_addr == 0 {
            return Ok(None);
        }
  
        match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Enum { i: _, choices, default: _, sortable: _} => {
                Ok(match memory.get_1_byte(value_addr) {
                    Some(x) => {
                        let value_num = x as usize;
        
                        if value_num > choices.len() {
                            None
                        } else {
                            Some(choices[value_num].clone())
                        }
                    },
                    None => None
                })
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn to_json(cursor: &NP_Cursor, memory: &NP_Memory<'value>) -> NP_JSON {

        match Self::into_value(cursor.clone(), memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        match y {
                            NP_Enum::Some(str_value) => {
                                NP_JSON::String(str_value.to_string())
                            },
                            NP_Enum::None => {
                                match &memory.schema[cursor.schema_addr] {
                                    NP_Parsed_Schema::Enum { i: _, choices: _, default, sortable: _} => {
                                        if let Some(d) = default {
                                            match d {
                                                NP_Enum::Some(val) => {
                                                    NP_JSON::String(val.clone())
                                                },
                                                NP_Enum::None => {
                                                    NP_JSON::Null
                                                }
                                            }
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

                        match &memory.schema[cursor.schema_addr] {
                            NP_Parsed_Schema::Enum { i: _, choices: _, default, sortable: _} => {
                                if let Some(d) = default {
                                    match d {
                                        NP_Enum::Some(x) => NP_JSON::String(x.clone()),
                                        NP_Enum::None => NP_JSON::Null
                                    }
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

    fn get_size(cursor: NP_Cursor, _memory: &NP_Memory) -> Result<usize, NP_Error> {

        if cursor.value.get_value_address() == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>())
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {


        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Enum as u8);

        let mut choices: Vec<NP_Enum> = Vec::new();

        let mut default_stir: Option<String> = None;

        match &json_schema["default"] {
            NP_JSON::String(def) => {
                default_stir = Some(def.clone());
            },
            _ => {}
        }

        let mut default_value: Option<NP_Enum> = None;
        let mut default_index: Option<u8> = None;

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
                                    default_value = Some(NP_Enum::new(def.clone()));
                                    default_index = Some(choices.len() as u8);
                                }
                            }
                            choices.push(NP_Enum::new(stir.clone()));
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
            Some(x) => schema_data.push(*x + 1),
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
            default: default_value,
            choices: choices,
            sortable: true
        });

        return Ok((true, schema_data, schema));
    
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let mut default_index: Option<u8> = None;
        let mut default_value: Option<NP_Enum> = None;

        if bytes[address + 1] > 0 {
            default_index = Some(bytes[address + 1] - 1);
        }

        let choices_len = bytes[address + 2];

        let mut choices: Vec<NP_Enum> = Vec::new();
        let mut offset: usize = address + 3;
        for x in 0..choices_len {
            let choice_size = bytes[offset] as usize;
            let choice_bytes = &bytes[(offset + 1)..(offset + 1 + choice_size)];
            let choice_string = unsafe { core::str::from_utf8_unchecked(choice_bytes) };
            choices.push(NP_Enum::new(choice_string.to_string()));
            offset += 1 + choice_size;

            if let Some(def) = default_index {
                if def == x {
                    default_value = Some(NP_Enum::new(choice_string.to_string()));
                }
            }
        }

        schema.push(NP_Parsed_Schema::Enum {
            i: NP_TypeKeys::Enum,
            sortable: true,
            default: default_value,
            choices: choices
        });

        (true, schema)
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
    let buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get::<NP_Enum>(&[])?.unwrap(), NP_Enum::new("hello"));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"option\",\"choices\":[\"hello\",\"world\"]}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&[], NP_Enum::new("hello"))?;
    assert_eq!(buffer.get::<NP_Enum>(&[])?, Some(NP_Enum::new("hello")));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<NP_Enum>(&[])?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}