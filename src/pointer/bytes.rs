use crate::schema::NP_Schema_Ptr;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::{NP_Size};
use crate::{schema::{NP_Schema_Parser, NP_TypeKeys}, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};

/// Represents arbitrary bytes type
#[derive(Debug)]
pub struct NP_Bytes {
    /// The bytes of the vec in this type
    pub bytes: Vec<u8>
}

/// Schema state for NP_Bytes
#[derive(Debug)]
pub struct NP_Bytes_Schema_State<'state> {
    /// 0 for dynamic size, anything greater than 0 is for fixed size
    pub size: u16,
    /// The default bytes
    pub default: &'state [u8]
}

impl NP_Bytes {
    /// Create a new bytes type with the provided Vec
    pub fn new(bytes: Vec<u8>) -> Self {
        NP_Bytes { bytes: bytes }
    }

    /// Get the schema data for this type
    pub fn get_schema_state<'state>(schema_ptr: &'state NP_Schema_Ptr) -> NP_Bytes_Schema_State<'state> {

        // fixed size
        let fixed_size = u16::from_be_bytes([
            schema_ptr.schema.bytes[schema_ptr.address + 1],
            schema_ptr.schema.bytes[schema_ptr.address + 2]
        ]);

        // default value size
        let default_size = u16::from_be_bytes([
            schema_ptr.schema.bytes[schema_ptr.address + 3],
            schema_ptr.schema.bytes[schema_ptr.address + 4]
        ]) as usize;

        let default_bytes: &[u8] = if default_size > 0 {
            &schema_ptr.schema.bytes[(schema_ptr.address + 5)..(schema_ptr.address + 5 + default_size)]
        } else {
            &[]
        };

        return NP_Bytes_Schema_State { size: fixed_size, default: default_bytes }
    }
}

impl NP_Schema_Parser for NP_Bytes {

    fn type_key(&self) -> u8 { NP_TypeKeys::Bytes as u8 }

    fn from_json_to_state(&self, json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "bytes" == type_str || "u8[]" == type_str || "[u8]" == type_str {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Bytes as u8);

            match json_schema["size"] {
                NP_JSON::Integer(x) => {
                    if x < 1 {
                        return Err(NP_Error::new("Fixed size for bytes must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for bytes cannot be larger than 2^16!"));
                    }
                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                NP_JSON::Float(x) => {
                    if x < 1.0 {
                        return Err(NP_Error::new("Fixed size for bytes must be larger than 1!"));
                    }
                    if x > u16::MAX.into() {
                        return Err(NP_Error::new("Fixed size for bytes cannot be larger than 2^16!"));
                    }

                    schema_data.extend((x as u16).to_be_bytes().to_vec());
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                }
            }

            match json_schema["default"] {
                NP_JSON::Array(bytes) => {
                    let mut default_bytes: Vec<u8> = Vec::new();
                    for x in bytes {
                        match x {
                            NP_JSON::Integer(x) => {
                                default_bytes.push(x as u8);
                            },
                            _ => {}
                        }
                    }
                    let length = default_bytes.len() as u16;
                    schema_data.extend(length.to_be_bytes().to_vec());
                    schema_data.extend(default_bytes);
                },
                _ => {
                    schema_data.extend(0u16.to_be_bytes().to_vec());
                }
            }

            return Ok(Some(schema_data));
        }
        
        Ok(None)
    }
}


impl Default for NP_Bytes {
    fn default() -> Self { 
        NP_Bytes { bytes: vec![] }
     }
}

impl NP_Value for NP_Bytes {


    fn type_idx() -> (u8, String) { (NP_TypeKeys::Bytes as u8, "bytes".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Bytes as u8, "bytes".to_owned()) }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        let state = NP_Bytes::get_schema_state(schema);
        if state.default.len() > 0 {
            Some(Box::new(NP_Bytes { bytes: state.default.to_vec() }))
        } else {
            None
        }
    }

    fn set_value(pointer: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
 
        let bytes = &value.bytes;
        let str_size = bytes.len() as u64;

        let mut addr = pointer.kind.get_value_addr() as usize;

        let write_bytes = pointer.memory.write_bytes();

        let schema_state = NP_Bytes::get_schema_state(&pointer.schema);

        if schema_state.size > 0 { // fixed size bytes
            let mut set_kind = pointer.kind.clone();

            if addr == 0 { // malloc new bytes

                let mut empty_bytes: Vec<u8> = Vec::with_capacity(schema_state.size as usize);
                for _x in 0..(schema_state.size as usize) {
                    empty_bytes.push(0);
                }
                
                addr = pointer.memory.malloc(empty_bytes)? as usize;

                // set location address
                set_kind = pointer.memory.set_value_address(pointer.location, addr as u32, &pointer.kind);
            }

            for x in 0..(schema_state.size as usize) {
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
                NP_Size::U8 => {
                    let size_bytes: u8 = pointer.memory.get_1_byte(addr).unwrap_or(0);
                    u8::from_be_bytes([size_bytes]) as usize
                },
                NP_Size::U16 => {
                    let size_bytes: &[u8; 2] = pointer.memory.get_2_bytes(addr).unwrap_or(&[0; 2]);
                    u16::from_be_bytes(*size_bytes) as usize
                },
                NP_Size::U32 => { 
                    let size_bytes: &[u8; 4] = pointer.memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
                    u32::from_be_bytes(*size_bytes) as usize
                }
            }
        } else {
            0 as usize
        };

        if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
    
            let size_bytes = match pointer.memory.size {
                NP_Size::U8 => { (str_size as u8).to_be_bytes().to_vec() }
                NP_Size::U16 => { (str_size as u16).to_be_bytes().to_vec() },
                NP_Size::U32 => { (str_size as u32).to_be_bytes().to_vec() }
            };

            // set string size
            for x in 0..size_bytes.len() {
                write_bytes[(addr + x) as usize] = size_bytes[x as usize];
            }

            let offset = match pointer.memory.size {
                NP_Size::U8 =>  { 1usize },
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
                NP_Size::U8 => { (str_size as u8).to_be_bytes().to_vec() }
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

        let schema_state = NP_Bytes::get_schema_state(&pointer.schema);

        if schema_state.size > 0 { // fixed size
            
            let size = schema_state.size as usize;
            
            // get bytes
            let bytes = &memory.read_bytes()[(addr)..(addr+size)];

            return Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec()})))

        } else { // dynamic size
            // get size of bytes

            let bytes_size: usize = match memory.size {
                NP_Size::U8 => {
                    let mut size_bytes: [u8; 1] = [0; 1];
                    size_bytes.copy_from_slice(&memory.read_bytes()[addr..(addr+1)]);
                    u8::from_be_bytes(size_bytes) as usize
                },
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
                NP_Size::U8 => { &memory.read_bytes()[(addr+1)..(addr+1+bytes_size)] },
                NP_Size::U16 => { &memory.read_bytes()[(addr+2)..(addr+2+bytes_size)] },
                NP_Size::U32 => { &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)] }
            };

            return Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec()})))
        }
        
    }

    fn to_json(pointer: NP_Lite_Ptr) -> NP_JSON {
        let this_bytes = Self::into_value(pointer.clone());

        match this_bytes {
            Ok(x) => {
                match x {
                    Some(y) => {

                        let bytes = y.bytes.into_iter().map(|x| NP_JSON::Integer(x as i64)).collect();

                        NP_JSON::Array(bytes)
                    },
                    None => {
                        let schema_state = NP_Bytes::get_schema_state(&pointer.schema);
                        if schema_state.default.len() > 0 {
                            let mut copy_bytes: Vec<NP_JSON> = Vec::new();
                            for b in schema_state.default {
                                copy_bytes.push(NP_JSON::Integer(*b as i64));
                            }
                            NP_JSON::Array(copy_bytes)
                        } else {
                            NP_JSON::Null
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

        let schema_state = NP_Bytes::get_schema_state(&pointer.schema);

        if schema_state.size > 0 { // fixed size
            return Ok(schema_state.size as u32);
        } else { // flexible size


            let bytes_size: u32 = match &memory.size {
                NP_Size::U8 => {
                    let mut size: [u8; 1] = [0; 1];
                    size.copy_from_slice(&memory.read_bytes()[addr..(addr+1)]);
                    (u8::from_be_bytes(size) as u32) + 1
                },
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