//! Top level abstraction for buffer objects

use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::pointer::NP_Value;
use crate::error::NP_Error;
use crate::pointer::{any::NP_Any, NP_Ptr};
use crate::memory::NP_Memory;
use crate::{schema::{NP_TypeKeys, NP_Schema}, json_flex::NP_JSON};
use alloc::{borrow::ToOwned, rc::Rc};


pub struct NP_Buffer {
    memory: Rc<NP_Memory>,
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

    /// Open the buffer to do something fancy with the internals
    /// 
    pub fn open<X: NP_Value + Default>(&mut self, callback: &mut (dyn FnMut(NP_Ptr<X>) -> Result<(), NP_Error>)) -> Result<(), NP_Error>
    {   
        let pointer: NP_Ptr<X> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
    
        // casting to ANY type -OR- schema is ANY type
        if X::type_idx().0 == NP_TypeKeys::Any as i64 || self.schema.type_data.0 == NP_TypeKeys::Any as i64  {
            return callback(pointer);
        }

        // casting matches root schema
        if X::type_idx().0 == self.schema.type_data.0 {
            return callback(pointer);
        }

        let mut err = "TypeError: Attempted to cast type (".to_owned();
        err.push_str(X::type_idx().1.as_str());
        err.push_str(") to schema of type (");
        err.push_str(self.schema.type_data.1.as_str());
        err.push_str(")");
        Err(NP_Error::new(err))
    }

    /// Open the buffer to extract an internal value using custom logic
    /// 
    pub fn extract<X: NP_Value + Default, T: NP_Value + Default, F>(&mut self, callback: &mut (dyn FnMut(NP_Ptr<X>) -> Result<T, NP_Error>)) -> Result<T, NP_Error>
    {

        match NP_TypeKeys::from(T::type_idx().0) {
            NP_TypeKeys::Table => { Err(NP_Error::new("Can't extract table type!")) },
            NP_TypeKeys::Map => { Err(NP_Error::new("Can't extract map type!")) },
            NP_TypeKeys::List => { Err(NP_Error::new("Can't extract list type!")) },
            NP_TypeKeys::Tuple => { Err(NP_Error::new("Can't extract tuple type!")) },
            _ => {

                let pointer: NP_Ptr<X> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
    
                // casting to ANY type -OR- schema is ANY type
                if X::type_idx().0 == NP_TypeKeys::Any as i64 || self.schema.type_data.0 == NP_TypeKeys::Any as i64  {
                    return callback(pointer);
                }
        
                // casting matches root schema
                if X::type_idx().0 == self.schema.type_data.0 {
                    return callback(pointer);
                }
        
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(X::type_idx().1.as_str());
                err.push_str(") to schema of type (");
                err.push_str(self.schema.type_data.1.as_str());
                err.push_str(")");
                Err(NP_Error::new(err))
            }
        }
    }

    /// Copy the entire buffer into a JSON object
    /// 
    pub fn json_encode(&self) -> NP_JSON {
        let root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
        root.json_encode()
    }

    /// Export the underlyig Vec<u8>, consumes the buffer
    /// 
    pub fn close(self) -> Vec<u8> {
        Rc::try_unwrap(self.memory).unwrap().dump()
    }

    /// Used to set scalar values inside the buffer, the path only works with dot notation.
    /// This does not work with collection types or `NP_JSON`.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    pub fn deep_set<X: NP_Value + Default>(&mut self, path: &str, value: X) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::JSON => { Err(NP_Error::new("Can't deep set with JSON type!")) },
            NP_TypeKeys::Table => { Err(NP_Error::new("Can't deep set table type!")) },
            NP_TypeKeys::Map => { Err(NP_Error::new("Can't deep set map type!")) },
            NP_TypeKeys::List => { Err(NP_Error::new("Can't deep set list type!")) },
            NP_TypeKeys::Tuple => { Err(NP_Error::new("Can't deep set tuple type!")) },
            _ => {
                let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
                let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
                pointer._deep_set::<X>(vec_path, 0, value)
            }
        }
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

        match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::Table => { Err(NP_Error::new("Can't deep get table type from here!")) },
            NP_TypeKeys::Map => { Err(NP_Error::new("Can't deep get map type from here!")) },
            NP_TypeKeys::List => { Err(NP_Error::new("Can't deep get list type from here!")) },
            NP_TypeKeys::Tuple => { Err(NP_Error::new("Can't deep get tuple type from here!")) },
            _ => {
                let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
                let pointer: NP_Ptr<NP_Any> = NP_Ptr::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));
                pointer._deep_get::<X>(vec_path, 0)
            }
        }
    }

    /// This performs a compaction if the closure provided as the second argument returns `true`, otherwise it just returns the original buffer.
    /// The closure is provided an argument that contains the original size of the buffer, how many bytes could be saved by compaction, and how large the new buffer would be after compaction.
    /// 
    pub fn maybe_compact<F>(&mut self, new_capacity: Option<u32>, mut callback: F) -> Result<(), NP_Error> where F: FnMut(NP_Compact_Data) -> bool {

        let wasted_bytes = self.calc_wasted_bytes()?;
        let old_size = self.memory.read_bytes().len() as u32;

        let compact_data = NP_Compact_Data { 
            old_buffer_size: old_size,
            new_buffer_size: if old_size > wasted_bytes { old_size - wasted_bytes } else  { 0 },
            wasted_bytes: wasted_bytes
        };

        let do_compact = callback(compact_data);

        if do_compact {
            self.compact(new_capacity)?
        }

        Ok(())
    }

    /// Compacts a buffer to remove an unused bytes or free space after a mutation.
    /// This is a pretty expensive operation so should be done sparingly.
    /// 
    pub fn compact(&mut self, new_capacity: Option<u32>) -> Result<(), NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let old_root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&self.memory));

        let new_bytes = Rc::new(NP_Memory::new(Some(capacity)));
        let new_root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&new_bytes));

        old_root._compact(new_root)?;

        self.memory = new_bytes;

        Ok(())
    }

    /// Recursively measures how many bytes each element in the buffer is using and subtracts that from the size of the buffer.
    /// This will let you know how many bytes can be saved from a compaction.
    /// 
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