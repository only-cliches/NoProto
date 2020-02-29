use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTuple {
    address: u32, // pointer location
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
}
