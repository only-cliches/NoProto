use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
use crate::pointer::{NoProtoScalar, NoProtoValue};
use std::result;


pub struct NoProtoBytes {
    mem: Vec<u8>
}

impl NoProtoBytes {
    pub fn malloc(&mut self, bytes: Vec<u8>) -> u32 {

        let location: u32 = self.mem.len() as u32;
        &self.mem.extend(bytes);

        location
    }
}

pub struct NoProtoBuffer {
    bytes: Rc<RefCell<NoProtoBytes>>,
    rootModel: Rc<RefCell<JsonValue>>
}

impl NoProtoBuffer {


}