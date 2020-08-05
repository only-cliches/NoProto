//! Top level abstraction for buffer objects

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::pointer::NP_Value;
use crate::error::NP_Error;
use crate::pointer::{any::NP_Any, NP_Ptr, DeepType};
use crate::memory::NP_Memory;
use crate::{NP_Factory, schema::{NP_TypeKeys, NP_Schema}};
use alloc::{borrow::ToOwned, rc::Rc};


pub struct NP_Buffer {
    pub memory: Rc<NP_Memory>,
    schema: Rc<NP_Schema>
}

pub struct NP_Compact_Data {
    pub old_buffer_size: u32,
    pub new_buffer_size: u32,
    pub wasted_bytes: u32
}

/// Buffers contain the memory of each object and allow you to access and mutate data.
impl NP_Buffer {
    /// Generate a complete new, empty buffer
    pub fn new(model: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self { // make new buffer

        NP_Buffer {
            memory: memory,
            schema: model
        }
    }

    /// Get the root pointer of the buffer.  You should make sure the type in the argument matches the schema.
    pub fn root<T: NP_Value + Default>(&mut self) -> Result<NP_Ptr<T>, NP_Error> {
        let buffer = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));

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

    /// Used to set scalar values inside the buffer, the path only works with dot notation.
    /// This does not work with collection types or `NP_JSON`.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    pub fn deep_set<X: NP_Value + Default>(&mut self, path: &str, value: X) -> Result<(), NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_set::<X>(DeepType::All, vec_path, 0, value)
    }

    /// Clear an inner value from the buffer.  The path only works with dot notation.
    /// This can also be used to clear deeply nested collection objects.
    /// 
    pub fn deep_clear(&self, path: &str) -> Result<(), NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_clear(vec_path, 0)
    }
  

    /// Retrieve an inner value from the buffer.  The path only works with dot notation.
    /// You can also use this to get JSON by casting the request type to `NP_JSON`.
    /// This can also be used to retrieve deeply nested collection objects.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    pub fn deep_get<X: NP_Value + Default>(&self, path: &str) -> Result<Option<Box<X>>, NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_get::<X>(DeepType::All, vec_path, 0)
    }

    #[doc(hidden)]
    pub fn _deep_set_scalar<X: NP_Value + Default>(&mut self, path: &str, value: X) -> Result<(), NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_set::<X>(DeepType::Scalar, vec_path, 0, value)
    }

    #[doc(hidden)]
    pub fn _deep_get_scalar<X: NP_Value + Default>(&self, path: &str) -> Result<Option<Box<X>>, NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_get::<X>(DeepType::Scalar, vec_path, 0)
    }

    /// This performs a compaction if the closure provided as the second argument returns `true`, otherwise it just returns the original buffer.
    /// The closure is provided an argument that contains the original size of the buffer, how many bytes could be saved by compaction, and how large the new buffer would be after compaction.
    pub fn maybe_compact<F>(self, new_capacity: Option<u32>, mut callback: F) -> Result<NP_Buffer, NP_Error> where F: FnMut(NP_Compact_Data) -> bool {

        let wasted_bytes = self.calc_wasted_bytes()?;
        let old_size = self.memory.read_bytes().len() as u32;

        let compact_data = NP_Compact_Data { 
            old_buffer_size: old_size,
            new_buffer_size: if old_size > wasted_bytes { old_size - wasted_bytes } else  { 0 },
            wasted_bytes: wasted_bytes
        };

        let do_compact = callback(compact_data);

        Ok(if do_compact {
            self.compact(new_capacity)?
        } else {
            self
        })

    }

    /// Compacts a buffer to remove an unused bytes or free space after a mutation.
    /// This is a pretty expensive operation so should be done sparingly.
    pub fn compact(self, new_capacity: Option<u32>) -> Result<NP_Buffer, NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let old_root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), self.memory);

        let new_bytes = Rc::new(NP_Memory::new(NP_Factory::new_buffer(Some(capacity))));
        let new_root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&new_bytes));

        old_root._compact(new_root)?;

        Ok(NP_Buffer {
            memory: new_bytes,
            schema: self.schema
        })
    }

    /// Recursively measures how many bytes each element in the buffer is using and subtracts that from the size of the buffer.
    /// This will let you know how many bytes can be saved from a compaction.
    pub fn calc_wasted_bytes(&self) -> Result<u32, NP_Error> {

        let root: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));

        let real_bytes = root.calc_size()? + 1u32;
        let total_bytes = self.memory.read_bytes().len() as u32;

        if total_bytes >= real_bytes {
            return Ok(total_bytes - real_bytes);
        } else {
            return Err(NP_Error::new("Error calclating bytes!"));
        }
    }
}