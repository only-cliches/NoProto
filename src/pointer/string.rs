//! NoProto supports Rust's native UTF8 [`String`](https://doc.rust-lang.org/std/string/struct.String.html) type.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::here;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set(here(), String::from("I want to play a game"))?;
//! 
//! assert_eq!(Box::new(String::from("I want to play a game")), new_buffer.get::<String>(here())?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```


use core::hint::unreachable_unchecked;

use alloc::vec::Vec;
use crate::{json_flex::JSMAP, memory::NP_Size, memory::NP_Memory, schema::{NP_Parsed_Schema, NP_Schema}};
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use core::str;
use super::{NP_Cursor_Addr};

impl<'value> NP_Value<'value> for String {
    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }
    fn schema_to_json(schema: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> { str_schema_to_json(schema) }
    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema { str_from_bytes_to_schema(address, bytes) }
    fn into_value(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<Option<Box<Self>>, NP_Error> { 
        Ok(match str_into_value(cursor_addr, memory)? {
            Some(x) => { Some(Box::new(String::from(*x))) },
            None => None
        })
    }
    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Box<Self>> { 
        match str_schema_default(schema) {
            Some(x) => { Some(Box::new(String::from(*x)))},
            None => None
        }
    }
    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> Result<usize, NP_Error> { str_get_size(cursor_addr, memory)}
    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> { str_from_json_to_schema(json_schema) }
    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON { str_to_json::<&str>(cursor_addr, memory) }
    fn set_value(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory, value: Box<&Self>) -> Result<NP_Cursor_Addr, NP_Error> { str_set_value(cursor_addr, memory, (*value).as_str()) }
}

impl<'value> NP_Value<'value> for &'value str {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::UTF8String as u8, "string".to_owned(), NP_TypeKeys::UTF8String) }
    fn schema_to_json(schema: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> { str_schema_to_json(schema) }
    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema { str_from_bytes_to_schema(address, bytes) }
    fn into_value(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<Option<Box<Self>>, NP_Error> { str_into_value(cursor_addr, memory) }
    fn schema_default(schema: &'value NP_Parsed_Schema) -> Option<Box<Self>> { str_schema_default(schema) }
    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> Result<usize, NP_Error> { str_get_size(cursor_addr, memory)}
    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> { str_from_json_to_schema(json_schema) }
    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON { str_to_json::<&str>(cursor_addr, memory) }
    fn set_value(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory, value: Box<&Self>) -> Result<NP_Cursor_Addr, NP_Error> { str_set_value(cursor_addr, memory, *value) }
}

fn str_schema_to_json(schema: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
    match schema {
        NP_Parsed_Schema::UTF8String { i: _, size, default, sortable: _ } => {
            let mut schema_json = JSMAP::new();
            schema_json.insert("type".to_owned(), NP_JSON::String(String::type_idx().1));
    
            if *size > 0 {
                schema_json.insert("size".to_owned(), NP_JSON::Integer(size.clone().into()));
            }
    
            if let Some(default_value) = default {
                schema_json.insert("default".to_owned(), NP_JSON::String(String::from(**default_value)));
            }
    
            Ok(NP_JSON::Dictionary(schema_json))
        },
        _ => {
            unsafe { unreachable_unchecked() }
        }
    }    
}

fn str_from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
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

    let default_bytes = str::from_utf8(&bytes[(address + 5)..(address + 5 + (default_size - 1))]).unwrap();

    return NP_Parsed_Schema::UTF8String {
        i: NP_TypeKeys::UTF8String,
        default: Some(Box::new(default_bytes)),
        size: fixed_size,
        sortable: fixed_size > 0
    }
}

fn str_into_value<'val>(cursor_addr: NP_Cursor_Addr, memory: &'val NP_Memory) -> Result<Option<Box<&'val str>>, NP_Error> {
    let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

    // empty value
    if cursor.address_value == 0 {
        return Ok(None);
    }

    match &**cursor.schema {
        NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
            if *size > 0 { // fixed size
        
                let size = *size as usize;
                
                // get bytes
                let bytes = &memory.read_bytes()[(cursor.address_value)..(cursor.address_value+size)];
    
                return Ok(Some(Box::new(str::from_utf8(bytes).unwrap())))
    
            } else { // dynamic size
                // get size of bytes
    
                let bytes_size: usize = memory.read_address(cursor.address_value);
    
                // get bytes
                let bytes = match memory.size {
                    NP_Size::U8 => { &memory.read_bytes()[(cursor.address_value+1)..(cursor.address_value+1+bytes_size)] },
                    NP_Size::U16 => { &memory.read_bytes()[(cursor.address_value+2)..(cursor.address_value+2+bytes_size)] },
                    NP_Size::U32 => { &memory.read_bytes()[(cursor.address_value+4)..(cursor.address_value+4+bytes_size)] }
                };
    
                return Ok(Some(Box::new(str::from_utf8(bytes).unwrap())))
            }
        },
        _ => { unsafe { unreachable_unchecked() } }
    }
}

fn str_schema_default<'def>(schema: &'def NP_Parsed_Schema) -> Option<Box<&'def str>> {
    match schema {
        NP_Parsed_Schema::UTF8String { i: _, size: _, default, sortable: _ } => {
            match default {
                Some(x) => Some(Box::new(&*x)),
                None => None
            }
        },
        _ => { panic!() }
    }
}

fn str_get_size(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory) -> Result<usize, NP_Error> {
    let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

    // empty value
    if cursor.address_value == 0 {
        return Ok(0)
    }
    

    match &**cursor.schema {
        NP_Parsed_Schema::UTF8String { i: _, size, default: _, sortable: _ } => {
            // fixed size
            if *size > 0 { 
                return Ok(*size as usize)
            }

            // dynamic size
            let bytes_size = memory.read_address(cursor.address_value) + memory.addr_size_bytes();
            
            // return total size of this string
            return Ok(bytes_size);
        },
        _ => { unsafe { unreachable_unchecked() } }
    }
}

fn str_from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

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
                Some(Box::new(bytes.as_str()))
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

fn str_to_json<'json, X>(cursor_addr: NP_Cursor_Addr, memory: &'json NP_Memory) -> NP_JSON where X: NP_Value<'json> + Into<String> {
    

    match X::into_value(cursor_addr, memory) {
        Ok(x) => {
            match x {
                Some(y) => {
                    NP_JSON::String((*y).into())
                },
                None => {
                    let cursor = memory.get_cursor_data(&cursor_addr).unwrap();
                    match &**cursor.schema {
                        NP_Parsed_Schema::UTF8String { i: _, size: _, default, sortable: _ } => {
                            match default {
                                Some(x) => NP_JSON::String(String::from(**x)),
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


fn str_set_value<'set>(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory, value: &str) -> Result<NP_Cursor_Addr, NP_Error> {

    if cursor_addr.is_virtual { panic!() }

    let bytes = value.as_bytes();

    let str_size = bytes.len() as usize;

    let cursor = memory.get_cursor_data(&cursor_addr).unwrap();

    let write_bytes = memory.write_bytes();

    let size = match &**cursor.schema {
        NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
            size
        },
        _ => { panic!() }
    };


    if *size > 0 { // fixed size bytes

        if cursor.address_value == 0 { // malloc new bytes

            let mut empty_bytes: Vec<u8> = Vec::with_capacity(*size as usize);
            for _x in 0..(*size as usize) {
                empty_bytes.push(0);
            }
            
            cursor.address_value = memory.malloc(empty_bytes)? as usize;
            memory.set_value_address(cursor.address, cursor.address_value);
        }

        for x in 0..(*size as usize) {
            if x < bytes.len() { // assign values of bytes
                write_bytes[(cursor.address_value + x)] = bytes[x];
            } else { // rest is zeros
                write_bytes[(cursor.address_value + x)] = 0;
            }
        }

        return Ok(cursor_addr)
    }

    // flexible size

    let prev_size: usize = if cursor.address_value != 0 {
        match memory.size {
            NP_Size::U8 => {
                let size_bytes: u8 = memory.get_1_byte(cursor.address_value).unwrap_or(0);
                u8::from_be_bytes([size_bytes]) as usize
            },
            NP_Size::U16 => {
                let size_bytes: &[u8; 2] = memory.get_2_bytes(cursor.address_value).unwrap_or(&[0; 2]);
                u16::from_be_bytes(*size_bytes) as usize
            },
            NP_Size::U32 => { 
                let size_bytes: &[u8; 4] = memory.get_4_bytes(cursor.address_value).unwrap_or(&[0; 4]);
                u32::from_be_bytes(*size_bytes) as usize
            }
        }
    } else {
        0 as usize
    };

    if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
        
        // update string length in buffer
        match memory.size {
            NP_Size::U8 => { 
                if str_size > core::u8::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u8).to_be_bytes();
                // set string size
                write_bytes[cursor.address_value] = size_bytes[0];
            }
            NP_Size::U16 => { 
                if str_size > core::u16::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u16).to_be_bytes();
                // set string size
                for x in 0..size_bytes.len() {
                    write_bytes[(cursor.address_value + x)] = size_bytes[x];
                }
            },
            NP_Size::U32 => { 
                if str_size > core::u32::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u32).to_be_bytes();
                // set string size
                for x in 0..size_bytes.len() {
                    write_bytes[(cursor.address_value + x)] = size_bytes[x];
                }
            }
        };


        let offset = memory.addr_size_bytes();

        // set bytes
        for x in 0..bytes.len() {
            write_bytes[(cursor.address_value + x + offset) as usize] = bytes[x as usize];
        }

        return Ok(cursor_addr)
    } else { // not enough space or space has not been allocted yet
        
        // first bytes are string length
        cursor.address_value = match memory.size {
            NP_Size::U8 => { 
                if str_size > core::u8::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u8).to_be_bytes();
                memory.malloc_borrow(&size_bytes)?
            },
            NP_Size::U16 => { 
                if str_size > core::u16::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u16).to_be_bytes();
                memory.malloc_borrow(&size_bytes)?
            },
            NP_Size::U32 => { 
                if str_size > core::u32::MAX as usize {
                    return Err(NP_Error::new("String too large!"))
                }
                let size_bytes = (str_size as u32).to_be_bytes();
                memory.malloc_borrow(&size_bytes)?
            }
        };

        memory.set_value_address(cursor.address, cursor.address_value);

        memory.malloc_borrow(bytes)?;

        return Ok(cursor_addr);
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
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!(buffer.get(&[])?.unwrap(), Box::new("hello"));

    Ok(())
}

#[test]
fn fixed_size_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"size\": 20}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(buffer.get(&[])?.unwrap(), Box::new("hello there this sen"));

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&[], "hello there this sentence is long")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), Box::new("hello there this sentence is long"));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<&str>(&[])?, None);

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}