use crate::error::NoProtoError;
use crate::memory::NoProtoMemory;
use std::{cell::RefCell, rc::Rc};
use crate::pointer::NoProtoValue;
use super::NoProtoPointerKinds;

pub struct NoProtoBytes {
    bytes: Vec<u8>
}

impl NoProtoValue for NoProtoBytes {

    fn new() -> Self {
        NoProtoBytes { bytes: vec![] }
    }

    fn is_type(&self, type_str: &str) -> bool {
        "bytes" == type_str || "u8[]" == type_str
    }

    fn type_idx() -> (i64, &'static str) { (2, "bytes") }
    fn self_type_idx(&self) -> (i64, &'static str) { (2, "bytes") }

    fn buffer_read(&mut self, address: u32, kind: &NoProtoPointerKinds, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<NoProtoBytes>, NoProtoError> {

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

        Ok(Some(NoProtoBytes { bytes: bytes.to_vec()}))
    }

    fn buffer_write(&mut self, address: u32, kind: &NoProtoPointerKinds, buffer: Rc<RefCell<NoProtoMemory>>, value: Vec<u8>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let size = value.len() as u64;

        if size >= std::u32::MAX as u64 { 
            return Err(NoProtoError::new("Bytes too large!"));
        } else {

            let mut addr = kind.get_value() as usize;
            let mut set_addr = false;

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
                    for x in 0..value.len() {
                        memory.bytes[(addr + x + 4) as usize] = value[x as usize];
                    }

                } else { // not enough space or space has not been allocted yet
                    

                    // first 4 bytes are length
                    addr = memory.malloc((size as u32).to_le_bytes().to_vec())? as usize;

                    set_addr = true;
                    // then bytes content
                    memory.malloc(value.to_vec())?;
                }
            }
            if set_addr { 
                Ok(kind.set_value_address(address, addr as u32, buffer)?)
            } else {
                Ok(*kind)
            }
        }
    }
}

impl Default for NoProtoBytes {
    fn default() -> Self { 
        NoProtoBytes { bytes: vec![] }
     }
}