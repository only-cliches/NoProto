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
//! let mut new_buffer = factory.empty_buffer(None)?;
//! new_buffer.set(&[], "I want to play a game")?;
//!
//! assert_eq!("I want to play a game", new_buffer.get::<&str>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(())
//! ```

use alloc::prelude::v1::Box;
use core::hint::unreachable_unchecked;

use crate::error::NP_Error;
use crate::{
    json_flex::JSMAP,
    memory::NP_Memory,
    schema::{NP_Parsed_Schema},
};
use crate::{json_flex::NP_JSON, pointer::NP_Value, schema::NP_TypeKeys};
use alloc::vec::Vec;

use super::{NP_Cursor, NP_Cursor_Addr, NP_Scalar};
use alloc::borrow::ToOwned;
use core::str;
use alloc::string::ToString;

/// &str type alias
pub type NP_String<'string> = &'string str;

impl NP_Scalar for &str {}

impl<'value> NP_Value<'value> for &'value str {
    fn type_idx() -> (&'value str, NP_TypeKeys) {
        ("string", NP_TypeKeys::UTF8String)
    }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) {
        ("string", NP_TypeKeys::UTF8String)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize) -> Result<NP_JSON, NP_Error> {
        match &schema[address] {
            NP_Parsed_Schema::UTF8String {
                i: _,
                size,
                default,
                sortable: _,
            } => {
                let mut schema_json = JSMAP::new();
                schema_json.insert(
                    "type".to_owned(),
                    NP_JSON::String(Self::type_idx().0.to_string()),
                );

                if *size > 0 {
                    schema_json.insert("size".to_owned(), NP_JSON::Integer(size.clone().into()));
                }

                if let Some(default_value) = default {
                    schema_json.insert(
                        "default".to_owned(),
                        NP_JSON::String(default_value.to_string()),
                    );
                }

                Ok(NP_JSON::Dictionary(schema_json))
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        // fixed size
        let fixed_size = u16::from_be_bytes([bytes[address + 1], bytes[address + 2]]);

        // default value size
        let default_size = u16::from_be_bytes([bytes[address + 3], bytes[address + 4]]) as usize;

        if default_size == 0 {
            schema.push(NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                default: None,
                sortable: fixed_size > 0,
                size: fixed_size,
            })
        } else {
            let default_bytes = str::from_utf8(&bytes[(address + 5)..(address + 5 + (default_size - 1))]).unwrap();

            schema.push(NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                default: Some(default_bytes.to_string()),
                size: fixed_size,
                sortable: fixed_size > 0,
            })
        }

        (fixed_size > 0, schema)
    }

    fn into_value(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c = memory.get_parsed(&cursor);

        let value_addr = c.value.get_addr_value() as usize;
        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::UTF8String { size, .. } => {
                if size > 0 {
                    // fixed size

                    // get bytes
                    let bytes = &memory.read_bytes()[(value_addr)..(value_addr + (size as usize))];

                    return Ok(Some(unsafe { str::from_utf8_unchecked(bytes) }));
                } else {
                    // dynamic size
                    // get size of bytes

                    let bytes_size: usize = u16::from_be_bytes(*memory.get_2_bytes(value_addr).unwrap_or(&[0u8; 2])) as usize;

                    // get bytes
                    let bytes = &memory.read_bytes()[(value_addr + 2)..(value_addr + 2 + bytes_size)];

                    return Ok(Some(unsafe { str::from_utf8_unchecked(bytes) }));
                }
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }

    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Self> {
        match schema {
            NP_Parsed_Schema::UTF8String { default, .. } => match default {
                Some(x) => Some(x),
                None => None,
            },
            _ => {
                panic!()
            }
        }
    }
    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

        let c = memory.get_parsed(&cursor);
        let value_addr = c.value.get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(0);
        }

        match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::UTF8String {
                i: _,
                size,
                default: _,
                sortable: _,
            } => {
                // fixed size
                if size > 0 {
                    return Ok(size as usize);
                }

                // dynamic size
                let bytes_size: usize = u16::from_be_bytes(*memory.get_2_bytes(value_addr).unwrap_or(&[0; 2])) as usize;

                // return total size of this string
                return Ok(bytes_size);
            }
            _ => unsafe { unreachable_unchecked() },
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::UTF8String as u8);

        let mut has_fixed_size = false;

        let size = match json_schema["size"] {
            NP_JSON::Integer(x) => {
                has_fixed_size = true;
                if x < 1 {
                    return Err(NP_Error::new(
                        "Fixed size for string must be larger than 1!",
                    ));
                }
                if x > u16::MAX.into() {
                    return Err(NP_Error::new(
                        "Fixed size for string cannot be larger than 2^16!",
                    ));
                }
                schema_data.extend((x as u16).to_be_bytes().to_vec());
                x as u16
            }
            NP_JSON::Float(x) => {
                has_fixed_size = true;
                if x < 1.0 {
                    return Err(NP_Error::new(
                        "Fixed size for string must be larger than 1!",
                    ));
                }
                if x > u16::MAX.into() {
                    return Err(NP_Error::new(
                        "Fixed size for string cannot be larger than 2^16!",
                    ));
                }

                schema_data.extend((x as u16).to_be_bytes().to_vec());
                x as u16
            }
            _ => {
                schema_data.extend(0u16.to_be_bytes().to_vec());
                0u16
            }
        };

        let default = match &json_schema["default"] {
            NP_JSON::String(bytes) => {
                let str_bytes = bytes.clone().into_bytes();
                if str_bytes.len() > u16::max as usize - 1 {
                    return Err(NP_Error::new(
                        "Default string value cannot be larger than 2^16 bytes!",
                    ));
                }
                schema_data.extend(((str_bytes.len() + 1) as u16).to_be_bytes().to_vec());
                schema_data.extend(str_bytes);
                Some(bytes.to_string())
            }
            _ => {
                schema_data.extend(0u16.to_be_bytes().to_vec());
                None
            }
        };

        schema.push(NP_Parsed_Schema::UTF8String {
            i: NP_TypeKeys::UTF8String,
            size: size,
            default: default,
            sortable: has_fixed_size,
        });

        return Ok((has_fixed_size, schema_data, schema));
    }

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        match Self::into_value(cursor.clone(), memory) {
            Ok(x) => match x {
                Some(y) => NP_JSON::String(y.to_string()),
                None => {
                    let c = memory.get_parsed(&cursor);
                    match &memory.schema[c.schema_addr] {
                        NP_Parsed_Schema::UTF8String { default, .. } => match default {
                            Some(x) => NP_JSON::String(x.to_string()),
                            None => NP_JSON::Null,
                        },
                        _ => unsafe { unreachable_unchecked() },
                    }
                }
            },
            Err(_e) => NP_JSON::Null,
        }
    }

    fn set_value<'set>(mut cursor: NP_Cursor_Addr, memory: &'set NP_Memory, value: Self) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'set + Sized {

        let c = memory.get_parsed(&cursor);

        assert_ne!(c.buff_addr, 0);
    
        let bytes = value.as_bytes();
    
        let str_size = bytes.len() as usize;
    
        let write_bytes = memory.write_bytes();
    
        let size = match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::UTF8String { size, .. } => size,
            _ => {
                unsafe { unreachable_unchecked() }
            }
        };
    
        if size > 0 {
            // fixed size bytes
    
            if c.value.get_addr_value() == 0 {
                // malloc new bytes
    
                let mut empty_bytes: Vec<u8> = Vec::with_capacity(size as usize);
                for _x in 0..size {
                    empty_bytes.push(0);
                }
    
                let new_addr = memory.malloc(empty_bytes)? as usize;
                c.value.set_addr_value(new_addr as u16);
            }

            let addr = c.value.get_addr_value() as usize;
    
            for x in 0..(size as usize) {
                if x < bytes.len() {
                    // assign values of bytes
                    write_bytes[(addr + x)] = bytes[x];
                } else {
                    // rest is zeros
                    write_bytes[(addr + x)] = 0;
                }
            }
    
            return Ok(cursor);
        }
    
        // flexible size
        let addr_value = c.value.get_addr_value() as usize;
    
        let prev_size: usize = if addr_value != 0 {
            let size_bytes: &[u8; 2] = memory.get_2_bytes(addr_value).unwrap_or(&[0; 2]);
            u16::from_be_bytes(*size_bytes) as usize
        } else {
            0 as usize
        };
    
        if prev_size >= str_size as usize {
            // previous string is larger than this one, use existing memory
    
            // update string length in buffer
            if str_size > core::u16::MAX as usize {
                return Err(NP_Error::new("String too large!"));
            }
            let size_bytes = (str_size as u16).to_be_bytes();
            // set string size
            for x in 0..size_bytes.len() {
                write_bytes[(addr_value + x)] = size_bytes[x];
            }

            let offset = 2;
    
            // set bytes
            for x in 0..bytes.len() {
                write_bytes[(addr_value + x + offset) as usize] = bytes[x];
            }
    
            return Ok(cursor);
        } else {
            // not enough space or space has not been allocted yet
    
            // first bytes are string length
            let new_addr = {
                if str_size > core::u16::MAX as usize {
                    return Err(NP_Error::new("String too large!"));
                }
                let size_bytes = (str_size as u16).to_be_bytes();
                memory.malloc_borrow(&size_bytes)?
            };
    
            c.value.set_addr_value(new_addr as u16);
    
            memory.malloc_borrow(bytes)?;
    
            return Ok(cursor);
        }
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
    let buffer = factory.empty_buffer(None)?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello");

    Ok(())
}

#[test]
fn fixed_size_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"size\": 20}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello there this sen");

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(
        buffer.get::<&str>(&[])?.unwrap(),
        "hello there this sentence is long"
    );
    buffer.del(&[])?;
    assert_eq!(buffer.get::<&str>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}
