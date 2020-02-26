

use json::JsonValue;
use crate::buffer::NoProtoMemory;
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

    pub fn select(&self, column: String) {

    }

    fn delete(&self, key: String) -> bool {
        false
    }

    fn clear(&self) {

    }

    fn has(&self, key: String) {

    }

}