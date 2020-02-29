use crate::pointer::NoProtoPointer;
use std::cell::BorrowMutError;
use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::result;


mod pointer;
mod collection;

pub struct NoProtoMemory {
    pub bytes: Vec<u8>
}

impl NoProtoMemory {
    pub fn malloc(&mut self, bytes: Vec<u8>) -> Option<u32> {
        let location: u32 = self.bytes.len() as u32;

        // not enough space left?
        if location + bytes.len() as u32 > 2u32.pow(32) {
            return None;
        }

        &self.bytes.extend(bytes);
        Some(location)
    }
}

pub struct NoProtoBuffer {
    memory: Rc<RefCell<NoProtoMemory>>,
    rootModel: Rc<RefCell<JsonValue>>
}

impl NoProtoBuffer {

    pub fn new(model: JsonValue, size: Option<usize>) -> Self { // make new buffer

        let capacity = match size {
            Some(x) => x,
            None => 1024
        };

        let new_bytes: Vec<u8> = Vec::with_capacity(capacity);

        new_bytes.extend(vec![0, 0, 0, 0]); // HEAD for root value (starts empty)

        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: new_bytes })),
            rootModel: Rc::new(RefCell::new(model))
        }
    }

    pub fn load(model: JsonValue, bytes: Vec<u8>) -> Self { // load existing buffer
        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: bytes})),
            rootModel: Rc::new(RefCell::new(model))
        }
    }

    pub fn get_root(&self) -> NoProtoPointer {        
        NoProtoPointer::new_standard(0, Rc::clone(&self.rootModel), Rc::clone(&self.memory))
    }

    pub fn compact(&self)  {
        
    }

    pub fn calc_wasted_bytes(&self) -> u32 {

        let total_bytes = self.memory.borrow().bytes.len() as u32;

        return 0;
    }

    pub fn maybe_compact<F>(&self, mut callback: F) -> bool 
        where F: FnMut(f32, f32) -> bool // wasted bytes, percent of waste
    {
        let wasted_bytes = self.calc_wasted_bytes() as f32;

        let total_bytes = self.memory.borrow().bytes.len() as f32;

        let size_without_waste = total_bytes - wasted_bytes;

        if callback(wasted_bytes, (total_bytes / size_without_waste) as f32) {
            self.compact();
            true
        } else {
            false
        }
    }

}