use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoMap<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    value: &'a NoProtoSchema,
}

impl<'a> NoProtoMap<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, value: &'a NoProtoSchema) -> Self {
        NoProtoMap {
            address,
            head,
            memory,
            value
        }
    }
}

/*
impl NoProtoMap {

    pub fn new(address: u32, memory: Rc<RefCell<NoProtoMemory>>, model: Rc<RefCell<JsonValue>>) -> Self {
        NoProtoMap {
            head: 0,
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

}*/