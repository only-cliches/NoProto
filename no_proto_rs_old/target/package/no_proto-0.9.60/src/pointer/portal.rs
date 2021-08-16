//! Clone type for recursion or duplicating data types.
//! 
//! Portal types allow you to teleport a type from one part of a schema to another.
//! 
//! This is useful for recursive types or if you'd like one type to be duplicated into multiple fields.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! // use portal to duplicate type
//! let factory: NP_Factory = NP_Factory::new(r#"struct({fields: {
//!     point1: struct({fields: { x: i32(), y: i32() }}),
//!     point2: portal({ to: "point1" }) // point2 is a "clone" of point1
//! }})"#)?;
//!
//! let mut new_buffer = factory.new_buffer(None);
//! new_buffer.set(&["point1", "x"], 200i32)?;
//! new_buffer.set(&["point1", "y"], 100i32)?;
//! 
//! // point2 can be treated just like point1
//! new_buffer.set(&["point2", "x"], 250i32)?;
//! new_buffer.set(&["point2", "y"], 150i32)?;
//! 
//! assert_eq!(new_buffer.get::<i32>(&["point1", "x"])?, Some(200i32));
//! assert_eq!(new_buffer.get::<i32>(&["point1", "y"])?, Some(100i32));
//! 
//! assert_eq!(new_buffer.get::<i32>(&["point2", "x"])?, Some(250i32));
//! assert_eq!(new_buffer.get::<i32>(&["point2", "y"])?, Some(150i32));
//! 
//! // use portal for recursive type
//! let factory: NP_Factory = NP_Factory::new(r#"struct({fields: {
//!     node: u32(),
//!     child: portal({ to: "" }) // child is a clone of the root object
//! }})"#)?;
//! 
//! let mut new_buffer = factory.new_buffer(None);
//! new_buffer.set(&["node"], 1u32)?;
//! new_buffer.set(&["child", "node"], 2u32)?;
//! new_buffer.set(&["child", "child", "node"], 3u32)?;
//! new_buffer.set(&["child", "child", "child", "node"], 4u32)?;
//! 
//! assert_eq!(Some(1u32), new_buffer.get(&["node"])?);
//! assert_eq!(Some(2u32), new_buffer.get(&["child", "node"])?);
//! assert_eq!(Some(3u32), new_buffer.get(&["child", "child", "node"])?);
//! assert_eq!(Some(4u32), new_buffer.get(&["child", "child", "child", "node"])?);
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! 

use crate::{idl::{JS_AST, JS_Schema}, memory::NP_Memory, schema::{NP_Parsed_Schema, NP_Portal_Data, NP_Value_Kind}};
use alloc::{sync::Arc, vec::Vec};

use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};


use alloc::string::String;
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::borrow::ToOwned;

use super::{NP_Cursor};

/// Defines the behavior of the portal data type
pub struct NP_Portal();


impl<'value> NP_Value<'value> for NP_Portal {
    fn type_idx() -> (&'value str, NP_TypeKeys) {
        ("portal", NP_TypeKeys::Portal)
    }

    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) {
        ("portal", NP_TypeKeys::Portal)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let schema = &schema[address];

        let data = unsafe { &*(*schema.data as *const NP_Portal_Data) };

        let mut schema_json = JSMAP::new();
        schema_json.insert(
            "type".to_owned(),
            NP_JSON::String(Self::type_idx().0.to_string()),
        );

        schema_json.insert(
            "to".to_owned(),
            NP_JSON::String(data.path.clone())
        );

        Ok(NP_JSON::Dictionary(schema_json))      
       
    }

    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error> {

        let data = unsafe { &*(*schema[address].data as *const NP_Portal_Data) };

        let mut result = String::from("portal({to: \"");
        result.push_str(data.path.as_str());
        result.push_str("\"});");
        Ok(result)
       
    }

    fn from_idl_to_schema(mut schema: Vec<NP_Parsed_Schema>, _name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut to: Option<String> = None;
        if args.len() > 0 {
            match &args[0] {
                JS_AST::object { properties } => {
                    for (key, value) in properties {
                        match idl.get_str(key).trim() {
                            "to" => {
                                match value {
                                    JS_AST::string { addr } => {
                                        to = Some(String::from(idl.get_str(addr).trim()));
                                    },
                                    _ => { }
                                }
                            },
                            _ => { }
                        }
                    }
                },
                _ => { }
            }
        }

        if let Some(path) = to {
            let mut schema_vec: Vec<u8> = Vec::new();
            schema_vec.push(NP_TypeKeys::Portal as u8);
            schema.push(NP_Parsed_Schema {
                val: NP_Value_Kind::Pointer,
                i: NP_TypeKeys::Portal,
                sortable: false,
                data: Arc::new(Box::into_raw(Box::new(NP_Portal_Data { path: path.clone(), schema: 0, parent_schema: 0 })) as *const u8)
            });
            let path_bytes = path.as_bytes();
            schema_vec.extend(&(path_bytes.len() as u16).to_be_bytes()[..]);
            schema_vec.extend(path_bytes);

            Ok((false, schema_vec, schema))             
        } else {
            Err(NP_Error::new("Portal types require a 'to' parameter!"))
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        let mut schema_vec: Vec<u8> = Vec::new();
        schema_vec.push(NP_TypeKeys::Portal as u8);
        match &json_schema["to"] {
            NP_JSON::String(path) => {
                schema.push(NP_Parsed_Schema {
                    val: NP_Value_Kind::Pointer,
                    i: NP_TypeKeys::Portal,
                    sortable: false,
                    data: Arc::new(Box::into_raw(Box::new(NP_Portal_Data { path: path.clone(), schema: 0, parent_schema: 0 })) as *const u8)
                });
                let path_bytes = path.as_bytes();
                schema_vec.extend(&(path_bytes.len() as u16).to_be_bytes()[..]);
                schema_vec.extend(path_bytes);
            },
            _ => return Err(NP_Error::new("Portal types require a 'to' parameter!"))
        }

        Ok((false, schema_vec, schema)) 
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let path_size: [u8; 2] = unsafe { *((&bytes[(address+1)..(address+3)]) as *const [u8] as *const [u8; 2]) };

        let path_size = u16::from_be_bytes(path_size) as usize;

        let path = &bytes[(address+3)..(address+3+path_size)];

        let path_str = unsafe { core::str::from_utf8_unchecked(path) };

        schema.push(NP_Parsed_Schema {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Portal,
            sortable: false,
            data: Arc::new(Box::into_raw(Box::new(NP_Portal_Data { path: String::from(path_str), schema: 0, parent_schema: 0 })) as *const u8)
        });

        (false, schema)
    }

    fn default_value(_depth: usize, _schema_addr: usize, _schemas: &Vec<NP_Parsed_Schema>) -> Option<Self> {
        None
    }

    fn to_json(depth:usize, cursor: &NP_Cursor, memory: &'value NP_Memory) -> NP_JSON {
        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_Portal_Data) };

        let mut next = cursor.clone();
        next.schema_addr = data.schema;
        next.parent_schema_addr = data.parent_schema;
        NP_Cursor::json_encode(depth + 1, &next, memory)
    }

    fn set_from_json<'set>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &'set NP_Memory, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {
        
        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_Portal_Data) };

        let mut next = cursor.clone();
        next.schema_addr = data.schema;
        next.parent_schema_addr = data.parent_schema;
        NP_Cursor::set_from_json(depth + 1, apply_null, next, memory, value)
       
    }

    fn get_size(depth:usize, cursor: &'value NP_Cursor, memory: &'value NP_Memory) -> Result<usize, NP_Error> {
        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_Portal_Data) };
        let mut next = cursor.clone();
        next.schema_addr = data.schema;
        next.parent_schema_addr = data.parent_schema;
        NP_Cursor::calc_size(depth + 1, &next, memory)
         
    }

    fn do_compact(depth:usize, mut from_cursor: NP_Cursor, from_memory: &'value NP_Memory, mut to_cursor: NP_Cursor, to_memory: &'value NP_Memory) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {
        
        let data = unsafe { &*(*from_memory.get_schema(from_cursor.schema_addr).data as *const NP_Portal_Data) };

        from_cursor.schema_addr = data.schema;
        from_cursor.parent_schema_addr = data.parent_schema;
        to_cursor.schema_addr = data.schema;
        to_cursor.parent_schema_addr = data.parent_schema;
        NP_Cursor::compact(depth + 1, from_cursor, from_memory, to_cursor, to_memory)
        
    }
}



#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {

    let schema = "{\"type\":\"portal\",\"to\":\"\"}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    Ok(())
}


#[test]
fn infinite_recursion() -> Result<(), NP_Error> {
    let schema = r#"{
        "type": "struct",
        "fields": [
            ["street", {"type": "string"}],
            ["city"  , {"type": "string"}],
            ["nested", {"type": "portal", "to": "nested"}]
        ]
    }"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);

    match buffer.set(&["nested","nested", "nested"], "hello infinite") {
        Ok(_done) => {
            panic!()
        },
        Err(_e) => {
            // should hit select overflow, if it doesn't we have a problem
        }
    }

    match buffer.get::<&str>(&["nested","nested", "nested"]) {
        Ok(_done) => {
            panic!()
        },
        Err(_e) => {
            // should hit select overflow, if it doesn't we have a problem
        }
    }

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = r#"{
        "type": "struct",
        "fields": [
            ["street", {"type": "string"}],
            ["city"  , {"type": "string"}],
            ["nested", {"type": "portal", "to": ""}]
        ]
    }"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);

    buffer.set(&["nested", "street"], "hello street")?;
    buffer.set(&["nested", "nested", "nested", "nested", "street"], "hello street 2")?;

    assert_eq!("hello street", buffer.get::<&str>(&["nested", "street"])?.unwrap());
    assert_eq!("hello street 2", buffer.get::<&str>(&["nested", "nested", "nested", "nested", "street"])?.unwrap());
    assert_eq!(buffer.calc_bytes()?.current_buffer, buffer.calc_bytes()?.after_compaction);
    buffer.del(&["nested", "street"])?;
    buffer.compact(None)?;
    assert_eq!("hello street 2", buffer.get::<&str>(&["nested", "nested", "nested", "nested", "street"])?.unwrap());
    assert_eq!(None, buffer.get::<&str>(&["nested", "street"])?);

    // testing set with JSON
    buffer.set_with_json(&[], r#"{"value":{"street": "foo", "nested": {"street": "foo2"}}}"#)?;

    assert_eq!(Some("foo"), buffer.get::<&str>(&["street"])?);
    assert_eq!(Some("foo2"), buffer.get::<&str>(&["nested", "street"])?);


    let schema = r#"{
        "type": "struct",
        "fields": [
            ["username", {"type": "string"}],
            ["email"  , {"type": "string"}],
            ["address", {"type": "struct", "fields": [
                ["street", {"type": "string"}],
                ["city", {"type": "string"}],
                ["more", {"type": "portal", "to": "address"}]
            ]}]
        ]
    }"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);

    buffer.set(&["address", "more", "more","more", "more","more", "more","more", "more", "street"], "hello")?;

    assert_eq!("hello", buffer.get::<&str>(&["address", "more", "more","more", "more","more", "more","more", "more", "street"])?.unwrap());

    Ok(())
}