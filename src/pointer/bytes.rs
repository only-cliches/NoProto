use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_PtrKinds};

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};

pub struct NP_Bytes {
    pub bytes: Vec<u8>
}

impl NP_Bytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        NP_Bytes { bytes: bytes }
    }
}


impl Default for NP_Bytes {
    fn default() -> Self { 
        NP_Bytes { bytes: vec![] }
     }
}

impl NP_Value for NP_Bytes {

    fn is_type( type_str: &str) -> bool {
        "bytes" == type_str || "u8[]" == type_str || "[u8]" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Bytes as i64, "bytes".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Bytes as i64, "bytes".to_owned()) }

    fn schema_default(schema: Rc<NP_Schema>) -> Option<Box<Self>> {
        match &schema.default {
            Some(x) => {
                match x {
                    NP_JSON::Array(value) => {

                        let mut vector = Vec::new();

                        for x in value {
                            match x {
                                NP_JSON::Integer(y) => {
                                    vector.push(*y as u8);
                                },
                                _ => {
                                    vector.push(0);
                                }
                            }
                        };

                        Some(Box::new(NP_Bytes { bytes: vector }))
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

    fn schema_state(_type_string: &str, json_schema: &NP_JSON) -> Result<i64, NP_Error> {
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

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let size = value.bytes.len() as u64;

        let mut addr = kind.get_value() as usize;

        let write_bytes = memory.write_bytes();

        if schema.type_state != -1 { // fixed size bytes
            let mut set_kind = kind.clone();

            if addr == 0 { // malloc new bytes

                let mut empty_bytes: Vec<u8> = Vec::with_capacity(schema.type_state as usize);
                for _x in 0..(schema.type_state as usize) {
                    empty_bytes.push(0);
                }
                
                addr = memory.malloc(empty_bytes)? as usize;

                // set location address
                set_kind = memory.set_value_address(address, addr as u32, kind);
            }

            for x in 0..(schema.type_state as usize) {
                if x < value.bytes.len() { // assign values of bytes
                    write_bytes[(addr + x)] = value.bytes[x];
                } else { // rest is zeros
                    write_bytes[(addr + x)] = 0;
                }
            }

            return Ok(set_kind)
        }

        // flexible size

        let prev_size: usize = if addr != 0 {
            let size_bytes: [u8; 4] = *memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
            u32::from_be_bytes(size_bytes) as usize
        } else {
            0 as usize
        };

        if prev_size >= size as usize { // previous bytes is larger than this one, use existing memory
    
            let size_bytes = size.to_be_bytes();
            // set string size
            for x in 0..size_bytes.len() {
                write_bytes[(addr + x) as usize] = size_bytes[x as usize];
            }

            // set bytes
            for x in 0..value.bytes.len() {
                write_bytes[(addr + x + 4) as usize] = value.bytes[x as usize];
            }
            return Ok(*kind);
        } else { // not enough space or space has not been allocted yet
            

            // first 4 bytes are length
            addr = memory.malloc((size as u32).to_be_bytes().to_vec())? as usize;

            // then bytes content
            memory.malloc(value.bytes.to_vec())?;

            return Ok(memory.set_value_address(address, addr as u32, kind));
        }
        
    }
    
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> Result<Option<Box<Self>>, NP_Error> {
        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(None)
        }

        let addr = value as usize;
        let memory = buffer;

        if schema.type_state != -1 { // fixed size
            
            let size = schema.type_state as usize;
            
            // get bytes
            let bytes = &memory.read_bytes()[(addr)..(addr+size)];

            return Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))

        } else { // dynamic size
            // get size of bytes
            let mut size: [u8; 4] = [0; 4];
            size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
            let bytes_size = u32::from_be_bytes(size) as usize;

            // get bytes
            let bytes = &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)];

            return Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))
        }
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> NP_JSON {
        let this_bytes = Self::buffer_into(address, *kind, Rc::clone(&schema), buffer);

        match this_bytes {
            Ok(x) => {
                match x {
                    Some(y) => {

                        let bytes = y.bytes.into_iter().map(|x| NP_JSON::Integer(x as i64)).collect();

                        NP_JSON::Array(bytes)
                    },
                    None => {
                        match &schema.default {
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

    fn buffer_get_size(_address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> Result<u32, NP_Error> {
        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(0)
        }
        
        // get size of bytes
        let addr = value as usize;
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer;

        if schema.type_state != -1 { // fixed size
            return Ok(schema.type_state as u32);
        } else { // flexible size
            size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
            let bytes_size = u32::from_be_bytes(size) as u32;
            
            // return total size of this string
            return Ok(bytes_size + 4);
        }
    }
}