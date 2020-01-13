use crate::pointer::{ NoProtoPointer }
use crate::buffer::{ NoProtoBuffer }

pub struct NoProtoTable {
    ptr: NoProtoPointer,
    buffer: NoProtoBuffer
}

impl NoProtoTable {

    pub fn new() -> Self {

    }

    pub fn set(&self, column: &str, data: NoProtoPointer) {

    }

    pub fn get(&self, column: &str) -> Option<NoProtoPointer> {

    }

    pub fn collection(&self, column: &str) -> NoProtoPointer {

    }

    fn delete(&self, key: String) -> bool {
        false
    }

    fn clear(&self) {

    }

    fn has(&self, key: String) {

    }

}