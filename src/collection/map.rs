use crate::pointer::NP_PtrKinds;
use crate::pointer::NP_Value;
use crate::{memory::NP_Memory, schema::NP_Schema, error::NP_Error};

use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::string::ToString;

pub struct NP_Map<'a> {
    pub address: u32, // pointer location
    pub head: u32,
    pub memory: Option<&'a NP_Memory>,
    pub value: Option<&'a NP_Schema>,
}

impl<'a> NP_Map<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: &'a NP_Memory, value: &'a NP_Schema) -> Self {
        NP_Map {
            address,
            head,
            memory: Some(memory),
            value: Some(value)
        }
    }
}

/*
impl NP_Map {

    pub fn new(address: u32, memory: &NP_Memory, model: Rc<RefCell<JsonValue>>) -> Self {
        NP_Map {
            head: 0,
            address: address,
            memory: memory,
            model: model
        }
    }

    pub fn select(&self, column: &str) -> NP_Ptr {

    }

    pub fn delete(&self, column: &str) -> bool {
        false
    }

    pub fn clear(&self) {

    }

    pub fn has(&self, column: &str) {

    }

}*/


impl<'a> NP_Value for NP_Map<'a> {
    fn new<T: NP_Value + Default>() -> Self {
        unreachable!()
    }
    fn is_type( _type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "map".to_owned()) }
    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .get()! Use .into() instead."))
    }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a> Default for NP_Map<'a> {

    fn default() -> Self {
        NP_Map { address: 0, head: 0, memory: None, value: None }
    }
}