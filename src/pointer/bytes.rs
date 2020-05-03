use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::JFObject};
use super::{NP_ValueInto, NP_PtrKinds};

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

pub struct NP_Bytes {
    pub bytes: Vec<u8>
}

impl NP_Bytes {
    pub fn new(bytes: Vec<u8>) -> Self {
        NP_Bytes { bytes: bytes }
    }
}

impl NP_Value for NP_Bytes {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Bytes { bytes: vec![] }
    }

    fn is_type( type_str: &str) -> bool {
        "bytes" == type_str || "u8[]" == type_str || "[u8]" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Bytes as i64, "bytes".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Bytes as i64, "bytes".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {

        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(None)
        }
        
        // get size of bytes
        let addr = value as usize;
        let size: [u8; 4] = *buffer.get_4_bytes(addr).unwrap_or(&[0; 4]);
        let memory = buffer;
        let bytes_size = u32::from_be_bytes(size) as usize;

        // get bytes
        let bytes = &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)];

        Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let size = value.bytes.len() as u64;

        if size >= core::u32::MAX as u64 { 
            return Err(NP_Error::new("Bytes too large!"));
        } else {

            let mut addr = kind.get_value() as usize;

            let write_bytes = memory.write_bytes();

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
    }

}

impl Default for NP_Bytes {
    fn default() -> Self { 
        NP_Bytes { bytes: vec![] }
     }
}

impl<'a> NP_ValueInto<'a> for NP_Bytes {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(None)
        }
        
        // get size of bytes
        let addr = value as usize;
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer;
        size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
        let bytes_size = u32::from_be_bytes(size) as usize;

        // get bytes
        let bytes = &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)];

        Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_bytes = Self::buffer_into(address, *kind, schema, buffer);

        match this_bytes {
            Ok(x) => {
                match x {
                    Some(y) => {

                        let bytes = y.bytes.into_iter().map(|x| JFObject::Integer(x as i64)).collect();

                        JFObject::Array(bytes)
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }
}