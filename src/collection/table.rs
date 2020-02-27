

use crate::pointer::NoProtoPointer;
use json::JsonValue;
use crate::buffer::NoProtoMemory;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTable {
    address: u32, // pointer location
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
}

pub struct NoProtoTableColumn {
    i: u8,
    column: String,
    ptr: NoProtoPointer
}

// iterator / looping feature for Table
impl Iterator for NoProtoTable {
    type Item = NoProtoTableColumn;
    
    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    fn next(&mut self) -> Option<NoProtoTableColumn> {

    }
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