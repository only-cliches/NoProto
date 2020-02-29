use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTable {
    address: u32, // pointer location
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
}

impl NoProtoTable {

    pub fn new(address: u32, memory: Rc<RefCell<NoProtoMemory>>, model: Rc<RefCell<JsonValue>>) -> Self {
        NoProtoTable {
            address: address,
            memory: memory,
            model: model
        }
    }

    pub fn select(&self, column: &str) -> NoProtoPointer {

    }

    pub fn delete(&self, column: &str) -> bool {
        false
    }

    pub fn clear(&self) {

    }

    pub fn has(&self, column: &str) {

    }

}