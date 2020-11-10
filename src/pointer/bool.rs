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
//! new_buffer.deep_set("", true)?;
//! 
//! assert_eq!(Box::new(true), new_buffer.deep_get::<bool>("")?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```

use crate::{json_flex::JSMAP, schema::NP_Schema_Ptr};
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::{NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};


impl<'value> NP_Value<'value> for bool {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Boolean as u8, "bool".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Boolean as u8, "bool".to_owned()) }

    fn schema_to_json(schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = bool_get_schema_state(&schema_ptr);

        if let Some(default) = schema_state {
            schema_json.insert("default".to_owned(), match default {
                true => NP_JSON::True,
                false => NP_JSON::False
            });
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        let state = bool_get_schema_state(&schema);

        match state {
            Some(x) => {
                Some(Box::new(x))
            },
            None => None
        }
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            // overwrite existing values in buffer
            ptr.memory.write_bytes()[addr as usize] = bytes[0];

            return Ok(ptr.kind);

        } else { // new value

            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            addr = ptr.memory.malloc(bytes.to_vec())?;
            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
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

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
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
                        let state = bool_get_schema_state(&ptr.schema);
                        match state {
                            Some(x) => {
                                if x == true {
                                    NP_JSON::True
                                } else {
                                    NP_JSON::False
                                }
                            },
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>() as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<NP_Schema>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if type_str == "bool" || type_str == "boolean" {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Boolean as u8);

            match json_schema["default"] {
                NP_JSON::False => {
                    schema_data.push(2);
                },
                NP_JSON::True => {
                    schema_data.push(1);
                },
                _ => {
                    schema_data.push(0);
                }
            };

            return Ok(Some(NP_Schema { is_sortable: true, bytes: schema_data}));
        }

        Ok(None)
    }
}

fn bool_get_schema_state(schema_ptr: &NP_Schema_Ptr) -> Option<bool> {

    match schema_ptr.schema.bytes[schema_ptr.address + 1] {
        0 => None,
        1 => Some(true),
        2 => Some(false),
        _ => unreachable!()
    }
}
