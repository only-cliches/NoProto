//! Stores the current unix epoch in u64.
//! 
//! Epoch should be stored in milliseconds.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::date::NP_Date;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "date"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Date::new(1604965249484))?;
//! 
//! assert_eq!(NP_Date::new(1604965249484), new_buffer.get::<NP_Date>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug, Formatter}};

use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::{NP_Cursor};
use crate::NP_Memory;
use alloc::string::ToString;


/// Holds Date data.
/// 
/// Check out documentation [here](../date/index.html).
/// 
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct NP_Date {
    /// The value of the date
    pub value: u64
}

impl super::NP_Scalar for NP_Date {}

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

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("date", NP_TypeKeys::Date) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("date", NP_TypeKeys::Date) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        match &schema[address] {
            NP_Parsed_Schema::Date { i: _, default, sortable: _} => {
                if let Some(d) = default {
                    schema_json.insert("default".to_owned(), NP_JSON::Integer(d.value as i64));
                }
            },
            _ => { }
        }
    
        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Self> {
        match schema {
            NP_Parsed_Schema::Date { default, .. } => {
                if let Some(d) = default {
                    Some(d.clone())
                } else {
                    None
                }
            },
            _ => None
        }
    }

    fn set_value<'set, M: NP_Memory>(cursor: NP_Cursor, memory: &'set M, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {

        let c_value = cursor.get_value(memory);

        let mut value_address = c_value.get_addr_value() as usize;

        if value_address != 0 { // existing value, replace
            let bytes = value.value.to_be_bytes();

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[value_address + x] = bytes[x];
            }

        } else { // new value

            let bytes = value.value.to_be_bytes();
            value_address = memory.malloc_borrow(&bytes)?;
            c_value.set_addr_value(value_address as u16);
        }                    

        Ok(cursor)
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c_value = cursor.get_value(memory);

        let value_addr = c_value.get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        Ok(match memory.get_8_bytes(value_addr) {
            Some(x) => {
                Some(NP_Date { value: u64::from_be_bytes(*x) })
            },
            None => None
        })
    }

    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        match Self::into_value(cursor, memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(y.value as i64)
                    },
                    None => {
                        match memory.get_schema(cursor.schema_addr) {
                            NP_Parsed_Schema::Date { i: _, default, sortable: _} => {
                                if let Some(d) = default {
                                    NP_JSON::Integer(d.value.clone() as i64)
                                } else {
                                    NP_JSON::Null
                                }
                            },
                            _ => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            Ok(0) 
        } else {
            Ok(core::mem::size_of::<u64>())
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Date as u8);

        let default = match json_schema["default"] {
            NP_JSON::Integer(x) => {
                schema_data.push(1);
                schema_data.extend((x as u64).to_be_bytes().to_vec());
                Some(NP_Date { value: x as u64})
            },
            _ => {
                schema_data.push(0);
                None
            }
        };
        
        schema.push(NP_Parsed_Schema::Date {
            i: NP_TypeKeys::Date,
            default: default,
            sortable: true
        });

        return Ok((true, schema_data, schema));

    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let has_default = bytes[address + 1];

        let default = if has_default == 0 {
            None
        } else {
            let bytes_slice = &bytes[(address + 2)..(address + 10)];

            let mut u64_bytes = 0u64.to_be_bytes();
            u64_bytes.copy_from_slice(bytes_slice);
            Some(NP_Date { value: u64::from_be_bytes(u64_bytes)})
        };

        schema.push(NP_Parsed_Schema::Date {
            i: NP_TypeKeys::Date,
            sortable: true,
            default: default
        });
        (true, schema)
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
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<NP_Date>(&[])?.unwrap(), NP_Date::new(1605138980392));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"date\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], NP_Date::new(1605138980392))?;
    assert_eq!(buffer.get::<NP_Date>(&[])?, Some(NP_Date::new(1605138980392)));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<NP_Date>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}