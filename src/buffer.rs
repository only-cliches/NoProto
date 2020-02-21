use std::cell::BorrowMutError;
use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::pointer::{NoProtoDataTypes, NoProtoGeneric};
use std::result;


pub struct NoProtoMemory {
    pub bytes: Vec<u8>
}

impl NoProtoMemory {
    pub fn malloc(&mut self, bytes: Vec<u8>) -> u32 {
        let location: u32 = self.bytes.len() as u32;
        &self.bytes.extend(bytes);
        location
    }
}

pub struct NoProtoBuffer {
    memory: Rc<RefCell<NoProtoMemory>>,
    rootModel: Rc<RefCell<JsonValue>>
}

impl NoProtoBuffer {

    /*pub fn new() -> Self { // make new buffer

    }
    
    pub fn load() -> Self { // load existing buffer

    }
    
    */

    pub fn get_root(&self) -> NoProtoGeneric {
        NoProtoGeneric::new(0, self.rootModel, self.memory)
    }

    pub fn set_root(&self, address: u32) {

    }

    pub fn new_string(&self, value: String) -> std::result::Result<u32, BorrowMutError> {

        let mut bytes = self.memory.try_borrow_mut()?;

        // first 4 bytes are string length
        let addr = bytes.malloc(value.len().to_le_bytes().to_vec());
        // then string content
        bytes.malloc(value.as_bytes().to_vec());

        Ok(addr)
    }

}