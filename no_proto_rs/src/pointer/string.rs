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
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], "I want to play a game")?;
//!
//! assert_eq!("I want to play a game", new_buffer.get::<&str>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(())
//! ```

use alloc::string::String;
use alloc::prelude::v1::Box;
use crate::{error::NP_Error, schema::String_Case};
use crate::{
    json_flex::JSMAP,
    memory::NP_Memory,
    schema::{NP_Parsed_Schema},
};
use crate::{json_flex::NP_JSON, pointer::NP_Value, schema::NP_TypeKeys};
use alloc::vec::Vec;

use super::{NP_Cursor, NP_Scalar};
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
            NP_Parsed_Schema::UTF8String { size, default, case, ..} => {
                let mut schema_json = JSMAP::new();
                schema_json.insert(
                    "type".to_owned(),
                    NP_JSON::String(Self::type_idx().0.to_string()),
                );

                match case {
                    String_Case::Uppercase => {
                        schema_json.insert("uppercase".to_owned(), NP_JSON::True);
                    },
                    String_Case::Lowercase => {
                        schema_json.insert("lowercase".to_owned(), NP_JSON::True);
                    },
                    _ => {}
                }

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
            },
            _ => Ok(NP_JSON::Null)
        }
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {

        // case byte
        let case_byte = String_Case::from(bytes[address + 1]);

        // fixed size
        let fixed_size = u16::from_be_bytes([bytes[address + 2], bytes[address + 3]]);

        // default value size
        let default_size = u16::from_be_bytes([bytes[address + 4], bytes[address + 5]]) as usize;

        if default_size == 0 {
            schema.push(NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                default: None,
                case: case_byte,
                sortable: fixed_size > 0,
                size: fixed_size,
            })
        } else {
            let default_bytes = str::from_utf8(&bytes[(address + 6)..(address + 6 + (default_size - 1))]).unwrap_or_default();

            schema.push(NP_Parsed_Schema::UTF8String {
                i: NP_TypeKeys::UTF8String,
                default: Some(default_bytes.to_string()),
                size: fixed_size,
                case: case_byte,
                sortable: fixed_size > 0,
            })
        }

        (fixed_size > 0, schema)
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c_value = cursor.get_value(memory);

        let value_addr = c_value.get_addr_value() as usize;
        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::UTF8String { size, .. } => {
                if *size > 0 {
                    // fixed size

                    // get bytes
                    let bytes = &memory.read_bytes()[(value_addr)..(value_addr + (*size as usize))];

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
            _ => Err(NP_Error::new("unreachable")),
        }
    }

    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Self> {
        match schema {
            NP_Parsed_Schema::UTF8String { default, .. } => match default {
                Some(x) => Some(x),
                None => None,
            },
            _ => None
        }
    }
    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);
        let value_addr = c_value.get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(0);
        }

        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::UTF8String { size, .. } => {
                // fixed size
                if *size > 0 {
                    return Ok(*size as usize);
                }

                // dynamic size
                let bytes_size: usize = u16::from_be_bytes(*memory.get_2_bytes(value_addr).unwrap_or(&[0; 2])) as usize;

                // return total size of this string plus length bytes
                return Ok(bytes_size + 2);
            }
            _ => Err(NP_Error::new("unreachable")),
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::UTF8String as u8);

        let mut case_byte = String_Case::None;
        let mut set = 0;

        match json_schema["lowercase"] {
            NP_JSON::True => { case_byte = String_Case::Lowercase; set += 1; },
            _ => {}
        }

        match json_schema["uppercase"] {
            NP_JSON::True => { case_byte = String_Case::Uppercase; set += 1; },
            _ => {}
        }

        if set == 2 {
            return Err(NP_Error::new("Only one of uppercase and lowercase can be set!"));
        }

        schema_data.push(case_byte as u8);

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
                if str_bytes.len() > u16::MAX as usize - 1 {
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
            case: case_byte,
            sortable: has_fixed_size,
        });

        return Ok((has_fixed_size, schema_data, schema));
    }

    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        match Self::into_value(cursor, memory) {
            Ok(x) => match x {
                Some(y) => NP_JSON::String(y.to_string()),
                None => {
                    match &memory.get_schema(cursor.schema_addr) {
                        NP_Parsed_Schema::UTF8String { default, .. } => match default {
                            Some(x) => NP_JSON::String(x.to_string()),
                            None => NP_JSON::Null,
                        },
                        _ => NP_JSON::Null,
                    }
                }
            },
            Err(_e) => NP_JSON::Null,
        }
    }

    fn set_value<'set, M: NP_Memory>(cursor: NP_Cursor, memory: &'set M, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {

        let c_value = cursor.get_value(memory);

        let (size, case) = match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::UTF8String { size, case, .. } => (*size, *case),
            _ => (0, String_Case::None)
        };

        let mut bytes = value.as_bytes();

        let mut owned: String;
        match case {
            String_Case::Uppercase => {
                owned = String::from(value);
                owned.make_ascii_uppercase();
                bytes = owned.as_bytes();
            },
            String_Case::Lowercase => {
                owned = String::from(value);
                owned.make_ascii_lowercase();
                bytes = owned.as_bytes();
            },
            _ => {}
        }
    
        let str_size = bytes.len() as usize;
    
        let mut write_bytes = memory.write_bytes();    

        if size > 0 {
            // fixed size bytes
    
            if c_value.get_addr_value() == 0 {
                // malloc new bytes
    
                let mut empty_bytes: Vec<u8> = Vec::with_capacity(size as usize);
                for _x in 0..size {
                    empty_bytes.push(32); // white space
                }
    
                let new_addr = memory.malloc(empty_bytes)? as usize;
                c_value.set_addr_value(new_addr as u16);
            }

            let addr = c_value.get_addr_value() as usize;
            write_bytes = memory.write_bytes();
    
            for x in 0..(size as usize) {
                if x < bytes.len() {
                    // assign values of bytes
                    write_bytes[(addr + x)] = bytes[x];
                } else {
                    // rest is white space
                    write_bytes[(addr + x)] = 32;
                }
            }
    
            return Ok(cursor);
        }
    
        // flexible size
        let addr_value = c_value.get_addr_value() as usize;
    
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
    
            c_value.set_addr_value(new_addr as u16);
    
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

    let schema = "{\"type\":\"string\",\"lowercase\":true}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\",\"uppercase\":true}";
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
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello");

    Ok(())
}

#[test]
fn fixed_size_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"size\": 20}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello there this sen");

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(
        buffer.get::<&str>(&[])?.unwrap(),
        "hello there this sentence is long"
    );
    buffer.del(&[])?;
    assert_eq!(buffer.get::<&str>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}


#[test]
fn uppercase_lowercase_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"lowercase\": true}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], "HELLO")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(),"hello");

    let schema = "{\"type\":\"string\",\"uppercase\": true}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], "hello")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(),"HELLO");


    Ok(())
}
