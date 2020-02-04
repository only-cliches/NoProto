

use std::rc::Rc;
use std::cell::RefCell;
use crate::pointer::NoProtoPointer;

pub struct NoProtoTable<'a> {
    pointer: Rc<RefCell<&'a NoProtoPointer<'a>>>
}


impl<'a> NoProtoTable<'a> {

    pub fn new(pointer: Rc<RefCell<&'a NoProtoPointer<'a>>>) -> Self {
        NoProtoTable {
            pointer: pointer
        }
    }

    pub fn set(&self, column: &str, data: NoProtoPointer) {
        let mut ptr = self.pointer.borrow_mut();
        let bytes: Vec<u8> = Vec::new();
        ptr.malloc(bytes);
    }

    //pub fn get(&self, column: &str) -> Option<NoProtoPointer> {

    //}

    fn delete(&self, key: String) -> bool {
        false
    }

    fn clear(&self) {

    }

    fn has(&self, key: String) {

    }

}