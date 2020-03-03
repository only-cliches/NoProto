use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTuple {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
}

/*
impl NoProtoTuple {

    pub fn new(address: u32, memory: Rc<RefCell<NoProtoMemory>>, model: Rc<RefCell<JsonValue>>) -> Self {
        NoProtoTuple {
            head: 0,
            address: address,
            memory: memory,
            model: model
        }
    }

    pub fn select(&self, index: u16) -> NoProtoPointer {

    }

    pub fn delete(&self, index: u16) -> bool {
        false
    }

    pub fn clear(&self) {

    }

}*/