//! Top level abstraction for buffer objects

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::pointer::NP_ValueInto;
use crate::pointer::NP_Value;
use crate::error::NP_Error;
use crate::pointer::{any::NP_Any, NP_Ptr};
use crate::memory::NP_Memory;
use crate::schema::{NP_TypeKeys, NP_Schema};
use alloc::{borrow::ToOwned};


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

    pub fn deep_set<X: NP_Value + Default, S: AsRef<str>>(&mut self, path: S, value: X) -> Result<(), NP_Error> {
        let vec_path: Vec<&str> = path.as_ref().split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, self.root_model, self.memory);
        pointer._deep_set::<X>(vec_path, 0, value)
    }

    pub fn deep_get<X: NP_Value + Default, S: AsRef<str>>(&self, path: S) -> Result<Option<Box<X>>, NP_Error> {
        let vec_path: Vec<&str> = path.as_ref().split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, self.root_model, self.memory);
        pointer._deep_get::<X>(vec_path, 0)
    }

    pub fn calc_wasted_bytes(&self) -> Result<u32, NP_Error> {

        let root: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, &self.root_model, self.memory);

        let real_bytes = root.calc_size()? + 1u32;
        let total_bytes = self.memory.read_bytes().len() as u32;

        if total_bytes >= real_bytes {
            return Ok(total_bytes - real_bytes);
        } else {
            return Err(NP_Error::new("Error calclating bytes!"));
        }
    }
}