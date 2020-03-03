use crate::collection::table::NoProtoTable;
use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub enum NoProtoPointerItemKind {
    list, map, table
}

pub struct NoProtoIteratorItem {
    pub i: u16,
    pub column: String,
    pub empty: bool,
    kind: NoProtoPointerItemKind
}
/*
impl NoProtoIteratorItem {
    
    pub fn select(&self) -> NoProtoPointer {

    }

    pub fn delete(&self) {

    }

    pub fn select_key(&self) -> NoProtoPointer {

    }
}



// iterator / looping feature for Table
impl Iterator for NoProtoTable {
    type Item = NoProtoIteratorItem;
    
    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    fn next(&mut self) -> Option<NoProtoIteratorItem> {

    }
}*/