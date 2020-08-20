use alloc::vec::Vec;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::{NP_Size, NP_Memory};
use crate::{schema::NP_TypeKeys, pointer::NP_Value, utils::from_utf8_lossy, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};


impl NP_Value for String {

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str || "utf-8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }

    fn schema_state(_type_string: &str, json_schema: &NP_JSON) -> core::result::Result<i64, NP_Error> {
        match json_schema["size"].into_i64() {
            Some(x) => {
                if *x > 0 && *x < (u32::MAX as i64) {
                    return Ok(*x);
                }
                return Ok(-1);
            },
            None => {
                return Ok(-1);
            }
        }
    }

    fn set_value(pointer: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
 
        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        let mut addr = pointer.kind.get_value_addr() as usize;

        let write_bytes = pointer.memory.write_bytes();

        if pointer.schema.type_state != -1 { // fixed size bytes
            let mut set_kind = pointer.kind.clone();

            if addr == 0 { // malloc new bytes

                let mut empty_bytes: Vec<u8> = Vec::with_capacity(pointer.schema.type_state as usize);
                for _x in 0..(pointer.schema.type_state as usize) {
                    empty_bytes.push(0);
                }
                
                addr = pointer.memory.malloc(empty_bytes)? as usize;

                // set location address
                set_kind = pointer.memory.set_value_address(pointer.location, addr as u32, &pointer.kind);
            }

            for x in 0..(pointer.schema.type_state as usize) {
                if x < bytes.len() { // assign values of bytes
                    write_bytes[(addr + x)] = bytes[x];
                } else { // rest is zeros
                    write_bytes[(addr + x)] = 0;
                }
            }

            return Ok(set_kind)
        }

        // flexible size

        let prev_size: usize = if addr != 0 {
            match pointer.memory.size {
                NP_Size::U16 => {
                    let mut size_bytes: [u8; 2] = [0; 2];
                    size_bytes.copy_from_slice(&pointer.memory.read_bytes()[addr..(addr+2)]);
                    u16::from_be_bytes(size_bytes) as usize
                },
                NP_Size::U32 => { 
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&pointer.memory.read_bytes()[addr..(addr+4)]);
                    u32::from_be_bytes(size_bytes) as usize
                }
            }
        } else {
            0 as usize
        };

        if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
    
            let size_bytes = match pointer.memory.size {
                NP_Size::U16 => { (str_size as u16).to_be_bytes().to_vec() },
                NP_Size::U32 => { (str_size as u32).to_be_bytes().to_vec() }
            };

            // set string size
            for x in 0..size_bytes.len() {
                write_bytes[(addr + x) as usize] = size_bytes[x as usize];
            }

            let offset = match pointer.memory.size {
                NP_Size::U16 => { 2usize },
                NP_Size::U32 => { 4usize }
            };

            // set bytes
            for x in 0..bytes.len() {
                write_bytes[(addr + x + offset) as usize] = bytes[x as usize];
            }

            return Ok(pointer.kind);
        } else { // not enough space or space has not been allocted yet
            
            // first 2 / 4 bytes are string length
            let str_bytes = match pointer.memory.size {
                NP_Size::U16 => { (str_size as u16).to_be_bytes().to_vec() },
                NP_Size::U32 => { (str_size as u32).to_be_bytes().to_vec() }
            };

            addr = pointer.memory.malloc(str_bytes)? as usize;

            // then string content
            pointer.memory.malloc(bytes.to_vec())?;

            return Ok(pointer.memory.set_value_address(pointer.location, addr as u32, &pointer.kind));
        }
            
    }

    fn into_value(pointer: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = pointer.kind.get_value_addr() as usize;
 
        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = pointer.memory;

        if pointer.schema.type_state != -1 { // fixed size
            
            let size = pointer.schema.type_state as usize;
            
            // get bytes
            let bytes = &memory.read_bytes()[(addr)..(addr+size)];

            return Ok(Some(Box::new(from_utf8_lossy(bytes))))

        } else { // dynamic size
            // get size of bytes

            let bytes_size: usize = match memory.size {
                NP_Size::U16 => {
                    let mut size_bytes: [u8; 2] = [0; 2];
                    size_bytes.copy_from_slice(&memory.read_bytes()[addr..(addr+2)]);
                    u16::from_be_bytes(size_bytes) as usize
                },
                NP_Size::U32 => { 
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
                    u32::from_be_bytes(size_bytes) as usize
                }
            };

            // get bytes
            let bytes = match memory.size {
                NP_Size::U16 => { &memory.read_bytes()[(addr+2)..(addr+2+bytes_size)]},
                NP_Size::U32 => { &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)] }
            };

            return Ok(Some(Box::new(from_utf8_lossy(bytes))))
        }
        
    }

    fn schema_default(schema: Rc<NP_Schema>) -> Option<Box<Self>> {
        match &schema.default {
            Some(x) => {
                match x {
                    NP_JSON::String(value) => {
                        Some(Box::new(value.clone()))
                    },
                    _ => {
                        None
                    }
                }
            },
            None => {
                None
            }
        }
    }

    fn to_json(pointer: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(pointer.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(*y)
                    },
                    None => {
                        match &pointer.schema.default {
                            Some(x) => x.clone(),
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

    fn get_size(pointer: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let value = pointer.kind.get_value_addr();

        // empty value
        if value == 0 {
            return Ok(0)
        }
        
        // get size of bytes
        let addr = value as usize;        
        let memory = pointer.memory;

        if pointer.schema.type_state != -1 { // fixed size
            return Ok(pointer.schema.type_state as u32);
        } else { // flexible size


            let bytes_size: u32 = match &memory.size {
                NP_Size::U16 => {
                    let mut size: [u8; 2] = [0; 2];
                    size.copy_from_slice(&memory.read_bytes()[addr..(addr+2)]);
                    (u16::from_be_bytes(size) as u32) + 2
                },
                NP_Size::U32 => {
                    let mut size: [u8; 4] = [0; 4];
                    size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
                    (u32::from_be_bytes(size) as u32) + 4
                }
            };
            
            // return total size of this string
            return Ok(bytes_size);
        }
    }
}