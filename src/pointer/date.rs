//! Stores the current unix epoch in u64.
//! 
//! Epoch should be stored in milliseconds.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::date::NP_Date;
//! use no_proto::here;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "date"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set(here(), NP_Date::new(1604965249484))?;
//! 
//! assert_eq!(Box::new(NP_Date::new(1604965249484)), new_buffer.get::<NP_Date>(here())?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug, Formatter}, hint::unreachable_unchecked};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::{NP_Cursor_Addr};
use crate::NP_Memory;

/// Holds Date data.
/// 
/// Check out documentation [here](../date/index.html).
/// 
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct NP_Date {
    /// The value of the date
    pub value: u64
}

impl NP_Date {
    /// Create a new date type with the given time
    pub fn new(time_ms: u64) -> Self {
        NP_Date { value: time_ms }
    }
}

impl Default for NP_Date {
    fn default() -> Self { 
        NP_Date { value: 0 }
     }
}

impl Debug for NP_Date {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl<'value> NP_Value<'value> for NP_Date {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Date as u8, "date".to_owned(), NP_TypeKeys::Date) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Date as u8, "date".to_owned(), NP_TypeKeys::Date) }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        match schema_ptr {
            NP_Parsed_Schema::Date { i: _, default, sortable: _} => {
                if let Some(d) = default {
                    schema_json.insert("default".to_owned(), NP_JSON::Integer(d.value as i64));
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    
        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        match schema {
            NP_Parsed_Schema::Date { i: _, default, sortable: _ } => {
                if let Some(d) = default {
                    Some(d.clone())
                } else {
                    None
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory, value: Box<&Self>) -> Result<NP_Cursor_Addr, NP_Error> {

        let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

        if cursor_addr.is_virtual { panic!() }

        if cursor.address_value != 0 { // existing value, replace
            let bytes = value.value.to_be_bytes();

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[cursor.address_value + x] = bytes[x];
            }

        } else { // new value

            let bytes = value.value.to_be_bytes();
            cursor.address_value = memory.malloc_borrow(&bytes)?;
            memory.set_value_address(cursor.address, cursor.address_value);
        }                    

        Ok(cursor_addr)
    }

    fn into_value<'into>(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> Result<Option<Box<Self>>, NP_Error> {
        let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

        // empty value
        if cursor.address_value == 0 {
            return Ok(None);
        }

        Ok(match memory.get_8_bytes(cursor.address_value) {
            Some(x) => {
                Some(Box::new(NP_Date { value: u64::from_be_bytes(*x) }))
            },
            None => None
        })
    }

    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> NP_JSON {

        match Self::into_value(cursor_addr, memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(y.value as i64)
                    },
                    None => {
                        let cursor = memory.get_cursor_data(&cursor_addr).unwrap();
                        match &**cursor.schema {
                            NP_Parsed_Schema::Date { i: _, default, sortable: _} => {
                                if let Some(d) = default {
                                    NP_JSON::Integer(d.value.clone() as i64)
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

    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> Result<usize, NP_Error> {
        let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

        if cursor.address_value == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u64>())
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "date" == type_str {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Date as u8);

            let default = match json_schema["default"] {
                NP_JSON::Integer(x) => {
                    schema_data.push(1);
                    schema_data.extend((x as u64).to_be_bytes().to_vec());
                    Some(Box::new(NP_Date { value: x as u64}))
                },
                _ => {
                    schema_data.push(0);
                    None
                }
            };

            return Ok(Some((schema_data, NP_Parsed_Schema::Date {
                i: NP_TypeKeys::Date,
                default: default,
                sortable: true
            })));
        }

        Ok(None)
    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
        let has_default = bytes[address + 1];

        let default = if has_default == 0 {
            None
        } else {
            let bytes_slice = &bytes[(address + 2)..(address + 10)];

            let mut u64_bytes = 0u64.to_be_bytes();
            u64_bytes.copy_from_slice(bytes_slice);
            Some(Box::new(NP_Date { value: u64::from_be_bytes(u64_bytes)}))
        };

        NP_Parsed_Schema::Date {
            i: NP_TypeKeys::Date,
            sortable: true,
            default: default
        }
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"date\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"date\",\"default\":1605138980392}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get(&[])?.unwrap(), Box::new(NP_Date::new(1605138980392)));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"date\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&[], NP_Date::new(1605138980392))?;
    assert_eq!(buffer.get::<NP_Date>(&[])?, Some(Box::new(NP_Date::new(1605138980392))));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<NP_Date>(&[])?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}