use crate::pointer::NP_Value;
use crate::{memory::NP_Memory, schema::NP_Schema};

pub struct NP_Tuple<'a> {
    pub address: u32, // pointer location
    pub head: u32,
    pub memory: Option<&'a NP_Memory>,
    pub values: Option<&'a Vec<NP_Schema>>
}


impl<'a> NP_Tuple<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: &'a NP_Memory, values: &'a Vec<NP_Schema>) -> Self {
        NP_Tuple {
            address,
            head,
            memory: Some(memory),
            values: Some(values)
        }
    }
/*
    pub fn select(&self, index: u16) -> Option<NP_Ptr> {

    }
*/
    pub fn delete(&self, _index: u16) -> bool {
        false
    }

    pub fn clear(&self) {

    }

}

impl<'a> NP_Value for NP_Tuple<'a> {
    fn new<T: NP_Value + Default>() -> Self {
        unreachable!()
    }
    fn is_type(_type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "tuple".to_owned()) }
    /*fn buffer_get(&self, _address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("This type doesn't support .get()!"))
    }
    fn buffer_set(&mut self, _address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("This type doesn't support .set()!"))
    }
    fn buffer_into(&self, address: u32, kind: &NP_PtrKinds, schema: NP_Schema, buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        self.buffer_get(address, kind, schema, buffer)
    }*/
}

impl<'a> Default for NP_Tuple<'a> {

    fn default() -> Self {
        NP_Tuple { address: 0, head: 0, memory: None, values: None}
    }
}