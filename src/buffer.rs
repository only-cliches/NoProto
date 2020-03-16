

use crate::pointer::NP_ValueInto;
use crate::pointer::NP_Value;
use crate::error::NP_Error;
use crate::pointer::NP_Ptr;
use crate::memory::NP_Memory;
use crate::schema::{NP_TypeKeys, NP_Schema};
use crate::PROTOCOL_VERSION;
use std::{rc::Rc, cell::RefCell};


pub struct NP_Buffer<'a> {
    pub memory: Rc<RefCell<NP_Memory>>,
    root_model: &'a NP_Schema
}

impl<'a> NP_Buffer<'a> {

    #[doc(hidden)]
    pub fn new(model: &'a NP_Schema, capcity: Option<u32>) -> Self { // make new buffer

        let capacity = match capcity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes: Vec<u8> = Vec::with_capacity(capacity as usize);

        new_bytes.extend(vec![
            PROTOCOL_VERSION, // Protocol version (for breaking changes if needed later)
            0, 0, 0, 0        // u32 HEAD for root value (starts at zero)
        ]); 

        NP_Buffer {
            memory: Rc::new(RefCell::new(NP_Memory { bytes: new_bytes })),
            root_model: model
        }
    }

    #[doc(hidden)]
    pub fn load(model: &'a NP_Schema, bytes: Vec<u8>) -> Self { // load existing buffer
        NP_Buffer {
            memory: Rc::new(RefCell::new(NP_Memory { bytes: bytes})),
            root_model: model
        }
    }
/*
    #[doc(hidden)]
    pub fn open_for_value<F, X: NP_Value<'a> + Default, R>(&mut self, mut callback: F) -> std::result::Result<R, NP_Error>
        where F: FnMut(NP_Ptr<X>) -> std::result::Result<R, NP_Error>
    {        
        let buffer = NP_Ptr::new_standard_ptr(1, self.root_model, Rc::clone(&self.memory))?;

        // casting to ANY type -OR- schema is ANY type
        if X::type_idx().0 == NP_TypeKeys::Any as i64 || buffer.schema.type_data.0 == NP_TypeKeys::Any as i64  {
            return callback(buffer);
        }

        // casting matches root schema
        if X::type_idx().0 == buffer.schema.type_data.0 {
            return callback(buffer);
        }
        
        Err(NP_Error::new(format!("TypeError: Attempted to cast type ({}) to schema of type ({})!", X::type_idx().1, buffer.schema.type_data.1).as_str()))
    }
*/
    pub fn open<F, X: NP_Value + Default + NP_ValueInto<'a>>(&mut self, mut callback: F) -> std::result::Result<(), NP_Error>
        where F: FnMut(NP_Ptr<'a, X>) -> std::result::Result<(), NP_Error>
    {        
        let buffer = NP_Ptr::new_standard_ptr(1, self.root_model, Rc::clone(&self.memory))?;

        // casting to ANY type -OR- schema is ANY type
        if X::type_idx().0 == NP_TypeKeys::Any as i64 || buffer.schema.type_data.0 == NP_TypeKeys::Any as i64  {
            return callback(buffer);
        }

        // casting matches root schema
        if X::type_idx().0 == buffer.schema.type_data.0 {
            return callback(buffer);
        }
        
        Err(NP_Error::new(format!("TypeError: Attempted to cast type ({}) to schema of type ({})!", X::type_idx().1, buffer.schema.type_data.1).as_str()))
    }

    pub fn deep_set<X: NP_Value + Default, S: AsRef<str>>(&self, _path: S, _value: X) -> std::result::Result<(), NP_Error> {
        Ok(())
    }

    pub fn deep_get<X: NP_Value + Default>(&self, _path: &str) -> std::result::Result<Option<X>, NP_Error> {
        Ok(Some(X::default()))
    }

    pub fn compact(&self)  {
        
    }

    pub fn close(self) -> std::result::Result<Vec<u8>, NP_Error> {
        Ok(Rc::try_unwrap(self.memory)?.into_inner().dump())
    }

    pub fn calc_wasted_bytes(&self) -> u32 {

        // let total_bytes = self.memory.borrow().bytes.len() as u32;

        return 0;
    }

    pub fn maybe_compact<F>(&self, mut callback: F) -> bool 
        where F: FnMut(f32, f32) -> bool // wasted bytes, percent of waste
    {
        let wasted_bytes = self.calc_wasted_bytes() as f32;

        let total_bytes = self.memory.borrow().bytes.len() as f32;

        let size_without_waste = total_bytes - wasted_bytes;

        if callback(wasted_bytes, (total_bytes / size_without_waste) as f32) {
            self.compact();
            true
        } else {
            false
        }
    }
}