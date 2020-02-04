use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::pointer::{NoProtoPointer, NoProtoValue};
use std::result;

pub struct NoProtoBuffer {
    bytes: Rc<RefCell<Vec<u8>>>,
    rootModel: Rc<RefCell<JsonValue>>
}

impl NoProtoBuffer {


    pub fn root(&self) -> NoProtoPointer {
        NoProtoPointer::new(0, &self.rootModel, &self.bytes)
    }
}