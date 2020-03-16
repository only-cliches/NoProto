use crate::pointer::NoProtoPointerKinds;
use crate::pointer::NoProtoValue;
use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTuple<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    values: Option<&'a Vec<NoProtoSchema>>
}


impl<'a> NoProtoTuple<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, values: &'a Vec<NoProtoSchema>) -> Self {
        NoProtoTuple {
            address,
            head,
            memory,
            values: Some(values)
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

impl<'a> NoProtoValue<'a> for NoProtoTuple<'a> {
    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        unreachable!()
    }
    fn is_type(_type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "tuple".to_owned()) }
    /*fn buffer_get(&self, _address: u32, _kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        Err(NoProtoError::new("This type doesn't support .get()!"))
    }
    fn buffer_set(&mut self, _address: u32, _kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>, _value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {
        Err(NoProtoError::new("This type doesn't support .set()!"))
    }
    fn buffer_into(&self, address: u32, kind: &NoProtoPointerKinds, schema: NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        self.buffer_get(address, kind, schema, buffer)
    }*/
}

impl<'a> Default for NoProtoTuple<'a> {

    fn default() -> Self {
        NoProtoTuple { address: 0, head: 0, memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![]})), values: None}
    }
}