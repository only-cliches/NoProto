//! Allows buffers to be created and mutated

use crate::pointer::NP_ValueInto;
use crate::pointer::NP_Value;
use crate::error::NP_Error;
use crate::pointer::NP_Ptr;
use crate::memory::NP_Memory;
use crate::schema::{NP_TypeKeys, NP_Schema};

use alloc::borrow::ToOwned;


pub struct NP_Buffer<'a> {
    pub memory: &'a NP_Memory,
    root_model: &'a NP_Schema
}

impl<'a> NP_Buffer<'a> {
    pub fn new(model: &'a NP_Schema, memory: &'a NP_Memory) -> Self { // make new buffer

        NP_Buffer {
            memory: memory,
            root_model: model
        }
    }

    pub fn root<T: NP_Value + Default + NP_ValueInto<'a>>(&mut self) -> core::result::Result<NP_Ptr<'a, T>, NP_Error> {
        let buffer = NP_Ptr::new_standard_ptr(1, self.root_model, self.memory);

        // casting to ANY type -OR- schema is ANY type
        if T::type_idx().0 == NP_TypeKeys::Any as i64 || buffer.schema.type_data.0 == NP_TypeKeys::Any as i64  {
            return Ok(buffer);
        }

        // casting matches root schema
        if T::type_idx().0 == buffer.schema.type_data.0 {
            return Ok(buffer);
        }
        
        let mut err = "TypeError: Attempted to cast type (".to_owned();
        err.push_str(T::type_idx().1.as_str());
        err.push_str(") to schema of type (");
        err.push_str(buffer.schema.type_data.1.as_str());
        err.push_str(")");
        Err(NP_Error::new(err))
    }

    pub fn set_value<X: NP_Value + Default, S: AsRef<str>>(self, _path: S, _value: X) -> Result<bool, NP_Error> {
        // Ok(false) 
        todo!();
    }

    pub fn get_value<X: NP_Value + Default, S: AsRef<str>>(&self, _path: S) -> Result<Option<X>, NP_Error> {
        //Ok(Some(X::default()))
        todo!();
    }

    pub fn compact(self)  {
        todo!();
    }

    pub fn calc_wasted_bytes(&self) -> u32 {

        // let total_bytes = self.memory.borrow().bytes.len() as u32;
        todo!();
    }

    pub fn maybe_compact<F>(self, mut callback: F) -> bool 
        where F: FnMut(f32, f32) -> bool // wasted bytes, percent of waste
    {
        todo!();
        /*
        let wasted_bytes = self.calc_wasted_bytes() as f32;

        let total_bytes = self.memory.read_bytes().len() as f32;

        let size_without_waste = total_bytes - wasted_bytes;

        if callback(wasted_bytes, (total_bytes / size_without_waste) as f32) {
            self.compact();
            true
        } else {
            false
        }*/
    }
}