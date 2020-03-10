use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTuple<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    values: &'a Vec<NoProtoSchema>
}


impl<'a> NoProtoTuple<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, values: &'a Vec<NoProtoSchema>) -> Self {
        NoProtoTuple {
            address,
            head,
            memory,
            values
        }
    }
/*
    pub fn select(&self, index: u16) -> Option<NoProtoPointer> {

    }
*/
    pub fn delete(&self, index: u16) -> bool {
        false
    }

    pub fn clear(&self) {

    }

}