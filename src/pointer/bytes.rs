use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use std::{cell::RefCell, rc::Rc};
use crate::{schema::NP_TypeKeys, pointer::NP_Value};
use super::{NP_ValueInto, NP_PtrKinds};

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

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(None)
        }
        
        // get size of bytes
        let addr = value as usize;
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer.try_borrow()?;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let bytes_size = u32::from_le_bytes(size) as usize;

        // get bytes
        let bytes = &memory.bytes[(addr+4)..(addr+4+bytes_size)];

        Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let size = value.bytes.len() as u64;

        if size >= std::u32::MAX as u64 { 
            return Err(NP_Error::new("Bytes too large!"));
        } else {

            let mut addr = kind.get_value() as usize;

            {
                let mut memory = buffer.try_borrow_mut()?;

                let prev_size: usize = if addr != 0 {
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                    u32::from_le_bytes(size_bytes) as usize
                } else {
                    0 as usize
                };

                if prev_size >= size as usize { // previous bytes is larger than this one, use existing memory
            
                    let size_bytes = size.to_le_bytes();
                    // set string size
                    for x in 0..size_bytes.len() {
                        memory.bytes[(addr + x) as usize] = size_bytes[x as usize];
                    }

                    // set bytes
                    for x in 0..value.bytes.len() {
                        memory.bytes[(addr + x + 4) as usize] = value.bytes[x as usize];
                    }
                    return Ok(*kind);
                } else { // not enough space or space has not been allocted yet
                    

                    // first 4 bytes are length
                    addr = memory.malloc((size as u32).to_le_bytes().to_vec())? as usize;

                    // then bytes content
                    memory.malloc(value.bytes.to_vec())?;

                    return Ok(memory.set_value_address(address, addr as u32, kind)?);
                }
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
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(None)
        }
        
        // get size of bytes
        let addr = value as usize;
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer.try_borrow()?;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let bytes_size = u32::from_le_bytes(size) as usize;

        // get bytes
        let bytes = &memory.bytes[(addr+4)..(addr+4+bytes_size)];

        Ok(Some(Box::new(NP_Bytes { bytes: bytes.to_vec() })))
    }
}