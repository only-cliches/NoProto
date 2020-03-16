use std::rc::Rc;
use std::cell::RefCell;

pub enum NP_PtrItemKind {
    list, map, table
}

pub struct NP_IteratorItem {
    pub i: u16,
    pub column: String,
    pub empty: bool,
    kind: NP_PtrItemKind
}
/*
impl NP_IteratorItem {
    
    pub fn select(&self) -> NP_Ptr {

    }

    pub fn delete(&self) {

    }

    pub fn select_key(&self) -> NP_Ptr {

    }
}



// iterator / looping feature for Table
impl Iterator for NP_Table {
    type Item = NP_IteratorItem;
    
    // Here, we define the sequence using `.curr` and `.next`.
    // The return type is `Option<T>`:
    //     * When the `Iterator` is finished, `None` is returned.
    //     * Otherwise, the next value is wrapped in `Some` and returned.
    fn next(&mut self) -> Option<NP_IteratorItem> {

    }
}*/