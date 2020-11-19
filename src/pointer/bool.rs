//! NoProto supports Rust's native [`bool`](https://doc.rust-lang.org/std/primitive.bool.html) type.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::bytes::NP_Bytes;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "bool"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set("", true)?;
//! 
//! assert_eq!(Box::new(true), new_buffer.get::<bool>("")?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```

use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP, schema::{NP_Parsed_Schema}};
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_Ptr};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};


impl<'value> NP_Value<'value> for bool {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Boolean as u8, "bool".to_owned(), NP_TypeKeys::Boolean) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Boolean as u8, "bool".to_owned(), NP_TypeKeys::Boolean) }

    fn schema_to_json(schema: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        match schema {
            NP_Parsed_Schema::Boolean { i: _, sortable: _, default} => {
                if let Some(d) = default {
                    schema_json.insert("default".to_owned(), match **d {
                        true => NP_JSON::True,
                        false => NP_JSON::False
                    });
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Box<Self>> {

        match schema {
            NP_Parsed_Schema::Boolean { i: _, sortable: _, default} => {
                default.clone()
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn set_value(ptr: &mut NP_Ptr<'value>, value: Box<&Self>) -> Result<(), NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            // overwrite existing values in buffer
            ptr.memory.write_bytes()[addr as usize] = bytes[0];

            return Ok(());

        } else { // new value

            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            addr = ptr.memory.malloc(bytes.to_vec())?;
            ptr.kind = ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);

            return Ok(());
        }
        
    }

    fn into_value(ptr: NP_Ptr<'value>) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = ptr.memory;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(if x == 1 { true } else { false }))
            },
            None => None
        })
    }

    fn to_json(ptr: &'value NP_Ptr<'value>) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        if *y == true {
                            NP_JSON::True
                        } else {
                            NP_JSON::False
                        }
                    },
                    None => {
                        match &**ptr.schema {
                            NP_Parsed_Schema::Boolean { i: _, sortable: _, default} => {
                                if let Some(d) = default {
                                    if **d == true {
                                        NP_JSON::True
                                    } else {
                                        NP_JSON::False
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

    fn get_size(ptr: &'value NP_Ptr<'value>) -> Result<usize, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>())
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if type_str == "bool" || type_str == "boolean" {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Boolean as u8);

            let default = match json_schema["default"] {
                NP_JSON::False => {
                    schema_data.push(2);
                    Some(Box::new(false))
                },
                NP_JSON::True => {
                    schema_data.push(1);
                    Some(Box::new(true))
                },
                _ => {
                    schema_data.push(0);
                    None
                }
            };

            return Ok(Some((schema_data, NP_Parsed_Schema::Boolean {
                i: NP_TypeKeys::Boolean,
                default: default,
                sortable: true
            })));
        }

        Ok(None)
    }
    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema { 
        NP_Parsed_Schema::Boolean {
            i: NP_TypeKeys::Boolean,
            sortable: true,
            default: match bytes[address] {
                0 => None,
                1 => Some(Box::new(true)),
                2 => Some(Box::new(false)),
                _ => unreachable!()
            }
        }
     }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"bool\",\"default\":false}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"bool\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"bool\",\"default\":false}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get("")?.unwrap(), Box::new(false));

    Ok(())
}


#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"bool\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set("", false)?;
    assert_eq!(buffer.get::<bool>("")?.unwrap(), Box::new(false));
    buffer.del("")?;
    assert_eq!(buffer.get::<bool>("")?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}