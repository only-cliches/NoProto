use crate::pointer::NoProtoPointerKinds;
use crate::pointer::NoProtoValue;
use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoMap<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    value: Option<&'a NoProtoSchema>,
}

impl<'a> NoProtoMap<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, value: &'a NoProtoSchema) -> Self {
        NoProtoMap {
            address,
            head,
            memory,
            value: Some(value)
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


impl<'a> NoProtoValue<'a> for NoProtoMap<'a> {
    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        unreachable!()
    }
    fn is_type( type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "map".to_owned()) }
    /*fn buffer_get(&self, address: u32, kind: &NoProtoPointerKinds, schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        Err(NoProtoError::new("This type doesn't support .get()!"))
    }
    fn buffer_set(&mut self, address: u32, kind: &NoProtoPointerKinds, schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {
        Err(NoProtoError::new("This type doesn't support .set()!"))
    }
    fn buffer_into(&self, address: u32, kind: &NoProtoPointerKinds, schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        self.buffer_get(address, kind, schema, buffer)
    }*/
}

impl<'a> Default for NoProtoMap<'a> {

    fn default() -> Self {
        NoProtoMap { address: 0, head: 0, memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![]})), value: None }
    }
}