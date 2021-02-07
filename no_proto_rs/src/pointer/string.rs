//! NoProto supports Rust's native UTF8 [`String`](https://doc.rust-lang.org/std/string/struct.String.html) type.
//!
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//!
//! let factory: NP_Factory = NP_Factory::new("string()")?;
//!
//! let mut new_buffer = factory.new_buffer(None);
//! new_buffer.set(&[], "I want to play a game")?;
//!
//! assert_eq!("I want to play a game", new_buffer.get::<&str>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(())
//! ```

use alloc::sync::Arc;
use alloc::string::String;
use alloc::prelude::v1::Box;
use crate::{error::NP_Error, idl::{JS_AST, JS_Schema}, schema::{NP_String_Data, NP_Value_Kind, String_Case}};
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

// impl<'value> NP_Scalar<'value> for &'value str {
//     fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> where Self: Sized {
//         None
//     }
// }

impl<'value> NP_Scalar<'value> for String {
    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Self> where Self: Sized {
        let data = unsafe { &*(*schema.data as *const NP_String_Data) };

        let size = data.size;

        Some(if size > 0 {
            let mut v: String = String::with_capacity(size as usize);
            for _x in 0..size {
                v.push(' ');
            }
            v
        } else {
            String::from("")
        })
    }

    fn np_max_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Option<Self> {

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };

        let size = data.size;


        if size == 0 {
            None
        } else {
            let mut value: String = String::with_capacity(size as usize);

            for _x in 0..size {
                value.push_str(unsafe { str::from_utf8_unchecked(&[128])});
            }

            Some(value)
        }
    }

    fn np_min_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Option<Self> {

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };

        let size = data.size;


        if size == 0 {
            None
        } else {
            let mut value: String = String::with_capacity(size as usize);

            for _x in 0..size {
                value.push_str(unsafe { str::from_utf8_unchecked(&[32])});
            }

            Some(value)
        }
    }
}


impl<'value> NP_Value<'value> for String {



    fn type_idx() -> (&'value str, NP_TypeKeys) {
        ("string", NP_TypeKeys::UTF8String)
    }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) {
        ("string", NP_TypeKeys::UTF8String)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize) -> Result<NP_JSON, NP_Error> {
        let schema = &schema[address];

        let data = unsafe { &*(*schema.data as *const NP_String_Data) };

        let mut schema_json = JSMAP::new();
        schema_json.insert(
            "type".to_owned(),
            NP_JSON::String(Self::type_idx().0.to_string()),
        );

        match data.case {
            String_Case::Uppercase => {
                schema_json.insert("uppercase".to_owned(), NP_JSON::True);
            },
            String_Case::Lowercase => {
                schema_json.insert("lowercase".to_owned(), NP_JSON::True);
            },
            _ => {}
        }

        if data.size > 0 {
            schema_json.insert("size".to_owned(), NP_JSON::Integer(data.size.clone().into()));
        }

        if let Some(default_value) = &data.default {
            schema_json.insert(
                "default".to_owned(),
                NP_JSON::String(default_value.to_string()),
            );
        }

        Ok(NP_JSON::Dictionary(schema_json))
      
    }

    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error> {
        let schema = &schema[address];

        let data = unsafe { &*(*schema.data as *const NP_String_Data) };

        let mut properties: Vec<String> = Vec::new();

        if let Some(x) = &data.default {
            let mut def = String::from("default: ");
            def.push_str("\"");
            def.push_str(x.as_str());
            def.push_str("\"");
            properties.push(def);
        }

        if data.size > 0 {
            let mut def = String::from("size: ");
            def.push_str(data.size.to_string().as_str());
            properties.push(def);
        }

        match data.case {
            String_Case::Uppercase => {
                properties.push(String::from("uppercase: true"));
            },
            String_Case::Lowercase => {
                properties.push(String::from("lowercase: true"));
            },
            _ => {}
        }

        if properties.len() == 0 {
            Ok(String::from("string()"))
        } else {
            let mut final_str = String::from("string({");
            final_str.push_str(properties.join(", ").as_str());
            final_str.push_str("})");
            Ok(final_str)
        }
      
    }

    fn from_idl_to_schema(mut schema: Vec<NP_Parsed_Schema>, _name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::UTF8String as u8);

        let mut case_byte = String_Case::None;
        let mut set = 0;

        let mut has_fixed_size = false;
        let mut size = 0u16;

        let mut default: Option<String> = Option::None;

        if args.len() > 0 {

            match &args[0] {
                JS_AST::object { properties } => {
                    for (key, value) in properties.iter() {
                        match idl.get_str(key).trim() {
                            "lowercase" => {
                                case_byte = String_Case::Lowercase; 
                                set += 1;
                            },
                            "uppercase" => {
                                case_byte = String_Case::Uppercase; 
                                set += 1;
                            },
                            "size" => {
                                match value {
                                    JS_AST::number { addr } => {
                                        match idl.get_str(addr).trim().parse::<u16>() {
                                            Ok(x) => {
                                                size = x;
                                                has_fixed_size = true;
                                            },
                                            Err(_e) => { return Err(NP_Error::new("size property must be an integer!")) }
                                        }
                                    },
                                    _ => { }
                                }
                            },
                            "default" => {
                                match value {
                                    JS_AST::string { addr } => {
                                        default = Some(String::from(idl.get_str(addr)))
                                    },
                                    _ => { }
                                }
                            }
                            _ => { }
                        }
                    }
                }
                _ => { }
            }
        }
        

        if set == 2 {
            return Err(NP_Error::new("Only one of uppercase or lowercase can be set!"));
        }

        schema_data.push(case_byte as u8);

        if has_fixed_size {
            schema_data.extend_from_slice(&size.to_be_bytes());
        } else {
            schema_data.extend_from_slice(&0u16.to_be_bytes());
        }

        if let Some(x) = &default {
            let str_bytes = x.as_bytes();
            schema_data.extend_from_slice(&((str_bytes.len() + 1) as u16).to_be_bytes());
            schema_data.extend_from_slice(str_bytes);
        } else {
            schema_data.extend_from_slice(&0u16.to_be_bytes());
        }


        schema.push(NP_Parsed_Schema {
            val: if size > 0 {
                NP_Value_Kind::Fixed(size as u32)
            } else {
                NP_Value_Kind::Pointer
            },
            i: NP_TypeKeys::UTF8String,
            sortable: has_fixed_size,
            data:  Arc::new(Box::into_raw(Box::new(NP_String_Data { size: size, default, case: case_byte })) as *const u8)
        });

        return Ok((has_fixed_size, schema_data, schema));
    }

    fn set_from_json<'set, M: NP_Memory>(_depth: usize, _apply_null: bool, cursor: NP_Cursor, memory: &'set M, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {
        match &**value {
            NP_JSON::String(value) => {
                Self::set_value(cursor, memory, value.clone())?;
            },
            _ => {}
        }

        Ok(())
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {

        // case byte
        let case_byte = String_Case::from(bytes[address + 1]);

        // fixed size
        let fixed_size = u16::from_be_bytes([bytes[address + 2], bytes[address + 3]]);

        // default value size
        let default_size = u16::from_be_bytes([bytes[address + 4], bytes[address + 5]]) as usize;

        if default_size == 0 {
            schema.push(NP_Parsed_Schema {
                val: if fixed_size > 0 {
                    NP_Value_Kind::Fixed(fixed_size as u32)
                } else {
                    NP_Value_Kind::Pointer
                },
                i: NP_TypeKeys::UTF8String,
                sortable: fixed_size > 0,
                data:  Arc::new(Box::into_raw(Box::new(NP_String_Data { size: fixed_size, default: None, case: case_byte })) as *const u8)
            })
        } else {
            let default_bytes = str::from_utf8(&bytes[(address + 6)..(address + 6 + (default_size - 1))]).unwrap_or_default();

            schema.push(NP_Parsed_Schema {
                val: if fixed_size > 0 {
                    NP_Value_Kind::Fixed(fixed_size as u32)
                } else {
                    NP_Value_Kind::Pointer
                },
                i: NP_TypeKeys::UTF8String,
                sortable: fixed_size > 0,
                data:  Arc::new(Box::into_raw(Box::new(NP_String_Data { size: fixed_size, default: Some(default_bytes.to_string()), case: case_byte })) as *const u8)
            })
        }

        (fixed_size > 0, schema)
    }

    fn set_value<'set, M: NP_Memory>(cursor: NP_Cursor, memory: &'set M, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {
        NP_String::set_value(cursor, memory, &value)
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {
        match NP_String::into_value(cursor, memory)? {
            Some(x) => Ok(Some(String::from(x))),
            None => Ok(None)
        }
    }


    fn get_size<M: NP_Memory>(_depth:usize, cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = || { cursor.get_value(memory) };
        let value_addr = c_value().get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(0);
        }

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };

        // fixed size
        if data.size > 0 {
            return Ok(data.size as usize);
        }

        // dynamic size
        let bytes_size: usize = u16::from_be_bytes(*memory.get_2_bytes(value_addr).unwrap_or(&[0; 2])) as usize;

        // return total size of this string plus length bytes
        return Ok(bytes_size + 2);
       
        
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

        schema.push(NP_Parsed_Schema {
            val: if size > 0 {
                NP_Value_Kind::Fixed(size as u32)
            } else {
                NP_Value_Kind::Pointer
            },
            i: NP_TypeKeys::UTF8String,
            sortable: has_fixed_size,
            data:  Arc::new(Box::into_raw(Box::new(NP_String_Data { size, default, case: case_byte })) as *const u8)
        });

        return Ok((has_fixed_size, schema_data, schema));
    }

    fn to_json<M: NP_Memory>(_depth:usize, cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        match Self::into_value(cursor, memory) {
            Ok(x) => match x {
                Some(y) => NP_JSON::String(y.to_string()),
                None => {
                    let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };
                    
                    match &data.default {
                        Some(x) => NP_JSON::String(x.to_string()),
                        None => NP_JSON::Null,
                    }
                       
                }
            },
            Err(_e) => NP_JSON::Null,
        }
    }
    
    fn default_value(_depth: usize, schema_addr: usize,schema: &Vec<NP_Parsed_Schema>) -> Option<Self> {
        match NP_String::default_value(0, schema_addr, schema) {
            Some(x) => Some(String::from(x)),
            None => None
        }
    }

}


impl<'value> NP_Scalar<'value> for NP_String<'value> {
    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> where Self: Sized {
        None
    }
    fn np_max_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &M) -> Option<Self> {
        None
    }

    fn np_min_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &M) -> Option<Self> {
        None
    }
}

impl<'value> NP_Value<'value> for NP_String<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { String::type_idx() }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { String::default().self_type_idx() }

    fn schema_to_json(_schema: &Vec<NP_Parsed_Schema>, _address: usize)-> Result<NP_JSON, NP_Error> {
        String::schema_to_json(_schema, _address)
    }

    fn set_from_json<'set, M: NP_Memory>(_depth: usize, _apply_null: bool, _cursor: NP_Cursor, _memory: &'set M, _value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {

        Ok(())
    }

    fn set_value<'set, M: NP_Memory>(cursor: NP_Cursor, memory: &'set M, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {

        let c_value = || { cursor.get_value(memory) };

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };

        let (size, case) = (data.size, data.case);

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

        if size > 0 {
            // fixed size bytes
    
            if c_value().get_addr_value() == 0 {
                // malloc new bytes
    
                let mut empty_bytes: Vec<u8> = Vec::with_capacity(size as usize);
                for _x in 0..size {
                    empty_bytes.push(32); // white space
                }
    
                let new_addr = memory.malloc(empty_bytes)? as usize;
                cursor.get_value_mut(memory).set_addr_value(new_addr as u16);
            }

            let addr = c_value().get_addr_value() as usize;
            let write_bytes = memory.write_bytes();
    
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
        let addr_value = c_value().get_addr_value() as usize;
    
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

            let write_bytes = memory.write_bytes();

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
    
            cursor.get_value_mut(memory).set_addr_value(new_addr as u16);
    
            memory.malloc_borrow(bytes)?;
    
            return Ok(cursor);
        }
    }

    fn default_value(_depth: usize, schema_addr: usize,schema: &'value Vec<NP_Parsed_Schema>) -> Option<Self> {
        let data = unsafe { &*(*schema[schema_addr].data as *const NP_String_Data) };

        match &data.default {
            Some(x) => Some(x),
            None => None,
        }
    }

    /// This is never called
    fn schema_to_idl(_schema: &Vec<NP_Parsed_Schema>, _address: usize)-> Result<String, NP_Error> {
        Ok(String::from("string()"))
    }

    /// This is never called
    fn from_idl_to_schema(schema: Vec<NP_Parsed_Schema>, _name: &str, _idl: &JS_Schema, _args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        Self::from_json_to_schema(schema, &Box::new(NP_JSON::Null))
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c_value = || { cursor.get_value(memory) };

        let value_addr = c_value().get_addr_value() as usize;
        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_String_Data) };

        if data.size > 0 {
            // fixed size

            // get bytes
            let bytes = &memory.read_bytes()[(value_addr)..(value_addr + (data.size as usize))];

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

    fn to_json<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {
        String::to_json(depth, cursor, memory)
    }

    fn get_size<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {
        String::get_size(depth, cursor, memory)
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema>, _json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        String::from_json_to_schema(schema, _json_schema)
    }


    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, _address: usize, _bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        String::from_bytes_to_schema(schema, _address, _bytes)
    }
}


#[test]
fn schema_parsing_works_idl() -> Result<(), NP_Error> {
    let schema = r#"string({default: "hello"})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl().unwrap());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl().unwrap());

    let schema = r#"string({size: 10})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl().unwrap());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl().unwrap());

    let schema = r#"string({lowercase: true})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl().unwrap());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl().unwrap());

    let schema = r#"string({uppercase: true})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl().unwrap());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl().unwrap());

    let schema = r#"string()"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl().unwrap());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl().unwrap());

    Ok(())
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"default\":\"hello\"}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\",\"size\":10}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\",\"lowercase\":true}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\",\"uppercase\":true}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    Ok(())
}

#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"default\":\"hello\"}";
    let factory = crate::NP_Factory::new_json(schema)?;
    let buffer = factory.new_buffer(None);
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello");

    Ok(())
}

#[test]
fn fixed_size_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"size\": 20}";
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
    buffer.set(&[] as &[&str], "hello there this sentence is long")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(), "hello there this sen");

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\"}";
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
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


#[test]
fn uppercase_lowercase_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"string\",\"lowercase\": true}";
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
    buffer.set(&[], "HELLO")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(),"hello");

    let schema = "{\"type\":\"string\",\"uppercase\": true}";
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
    buffer.set(&[], "hello")?;
    assert_eq!(buffer.get::<&str>(&[])?.unwrap(),"HELLO");


    Ok(())
}
