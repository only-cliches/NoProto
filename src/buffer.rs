use crate::pointer::NoProtoPointer;
use std::cell::BorrowMutError;
use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
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

    pub fn new(model: Rc<RefCell<JsonValue>>, size: Option<usize>) -> Self { // make new buffer

        let capacity = match size {
            Some(x) => x,
            None => 1024
        };

        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: Vec::with_capacity(capacity) })),
            rootModel: model
        }
    }

    pub fn load(model: Rc<RefCell<JsonValue>>, bytes: Vec<u8>) -> Self { // load existing buffer
        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: bytes})),
            rootModel: model
        }
    }

    pub fn get_root(&self) -> NoProtoPointer {

        {
            let mut memory = self.memory.borrow_mut();
            if memory.bytes.len() == 0 {
                memory.malloc(vec![0,0,0,0]);
            }
        }
        
        NoProtoPointer::new_standard(0, Rc::clone(&self.rootModel), Rc::clone(&self.memory))
    }

}