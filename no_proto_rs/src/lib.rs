//! Docs!
//!

#![warn(missing_docs)]
#![allow(non_camel_case_types)]
#![no_std]


#[cfg(test)]
#[macro_use]
extern crate std;

use alloc::sync::Arc;


#[cfg(target_endian = "little")]
#[macro_use]
macro_rules! le_bytes_read {
    ($type: ty, $bytes: tt) => {
        unsafe { *($bytes as *const u8 as *const $type) }
    }
}

#[cfg(target_endian = "big")]
#[macro_use]
macro_rules! le_bytes_read {
    ($type: ty, $bytes: tt) => {
        <$type>::from_le_bytes(unsafe { *($bytes as *const u8 as *const [u8; core::mem::size_of::<$type>()]) })
    }
}

#[cfg(target_endian = "little")]
#[macro_use]
macro_rules! le_bytes_write {
    ($type: ty, $bytes: tt, $value: tt) => {
        unsafe {
            let ptr = $bytes as *mut u8 as *mut $type;
            *ptr = *$value as $type;
        }
    }
}

#[cfg(target_endian = "big")]
#[macro_use]
macro_rules! le_bytes_write {
    ($type: ty, $bytes: tt, $value: tt) => {
        unsafe {
            let ptr = $bytes as *mut u8 as *mut [u8; core::mem::size_of::<$type>()];
            *ptr = (*$value as $type).to_le_bytes();
        }
    }
}

use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::schema::{NP_Schema};
// use crate::buffer::{NP_Buffer, buffer_rpc};
use core::any::Any;

#[allow(dead_code)]
#[allow(missing_docs)]
#[doc(hidden)]
mod map;
mod utils;
mod error;
mod json_flex;
mod schema;
mod memory;
mod buffer;
mod values;
mod types;
mod format;

#[macro_use]
extern crate alloc;


#[allow(dead_code)]
#[derive(Debug)]
pub struct NP_Factory {
    /// schema data used by this factory
    schema: Arc<NP_Schema>
}

unsafe impl Send for NP_Factory {}
unsafe impl Sync for NP_Factory {}

/// When calling `maybe_compact` on a buffer, this struct is provided to help make a choice on wether to compact or not.
#[allow(dead_code)]
#[derive(Debug, Eq, PartialEq, Default)]
pub struct NP_Size_Data {
    /// The size of the existing buffer
    pub current_buffer: usize,
    /// The estimated size of buffer after compaction
    pub after_compaction: usize,
    /// How many known wasted bytes in existing buffer
    pub wasted_bytes: usize
}

// impl NP_Factory {

//     /// Get a factory from a human generated string schema
//     ///
//     pub fn from_schema<S: AsRef<str>>(schema: S) -> Result<Self, NP_Error> {
//         let parsed = NP_Schema::parse(schema)?;

//         Ok(Self {
//             schema: Arc::new(parsed)
//         })
//     }

//     /// Get a factory from a compiled schema
//     ///
//     pub fn from_compiled_schema(schema: &[u8]) -> Result<Self, NP_Error> {
//         let parsed = NP_Schema::from_bytes(schema)?;

//         Ok(Self {
//             schema: Arc::new(parsed)
//         })
//     }

//     /// Compiles the schema
//     ///
//     /// Compiled schemas parse much faster than string schemas, and take up far less space
//     ///
//     pub fn compile_schema(&self) -> Result<Vec<u8>, NP_Error> {
//         self.schema.to_bytes()
//     }

//     /// Get the schema information for a specific data type in the schema
//     ///
//     pub fn inspect_schema<S: AsRef<str>>(&self, type_path: S) -> Option<NP_Schema_Data> {
//         self.schema.get_schema_info(type_path.as_ref())
//     }

//     /// Retrieve arguments from the schema
//     ///
//     pub fn query_schema_args<S: AsRef<str>>(&self, type_path: S, args_path: S) -> NP_Args {
//         if let Some(schema) = self.schema.query_schema(type_path.as_ref()) {
//             if let Some(x) = schema.arguments.query(args_path.as_ref(), self.schema.get_source_as_str()) {
//                 x
//             } else {
//                 NP_Args::NULL
//             }
//         } else {
//             NP_Args::NULL
//         }
//     }

//     /// Get data from the info block of the schema
//     ///
//     pub fn query_info_args<S: AsRef<str>>(&self, args_path: S) -> NP_Args {
//         if let Some(info) = self.schema.name_index.get("__info") {
//             let result = self.schema.schemas[info.data].arguments.query(args_path.as_ref(), self.schema.get_source_as_str());
//             if let Some(x) = result {
//                 x
//             } else {
//                 NP_Args::NULL
//             }
//         } else {
//             NP_Args::NULL
//         }
//     }

//     /// Open existing Vec<u8> as buffer for this factory.
//     ///
//     pub fn open_buffer(&self, bytes: Vec<u8>) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_existing(NP_Memory::existing_owned(bytes, self.schema.clone(), 0))
//     }

//     /// Open existing buffer as ready only ref, can much faster if you don't need to mutate anything.
//     ///
//     /// All operations that would lead to mutation fail.  You can't perform any mutations on a buffer opened with this method.
//     ///
//     ///
//     pub fn open_buffer_ref(&self, bytes: &[u8]) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_existing( NP_Memory::existing_ref(bytes, self.schema.clone(), 0))
//     }

//     /// Open existing buffer as mutable ref, can be much faster to skip copying.  The `data_len` property is how many bytes the data in the buffer is using up.
//     ///
//     /// Some mutations cannot be done without appending bytes to the existing buffer.  Since it's impossible to append bytes to a `&mut [u8]` type, you should provide mutable slice with extra bytes on the end if you plan to mutate the buffer.
//     ///
//     /// The `data_len` is at which byte the data ends in the buffer, this will be moved as needed by compaction and mutation operations.
//     ///
//     /// If the `&mut [u8]` type has the same length as `data_len`, mutations that require additional bytes will fail. `&mut [u8].len() - data_len` is how many bytes the buffer has for new allocations.
//     ///
//     ///
//     pub fn open_buffer_ref_mut(&self, bytes: &mut [u8], data_len: usize) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_existing(NP_Memory::existing_ref_mut(bytes, data_len, self.schema.clone(), 0))
//     }

//     /// Generate a new empty buffer from this factory.
//     ///
//     /// The first opional argument, capacity, can be used to set the space of the underlying Vec<u8> when it's created.  If you know you're going to be putting lots of data into the buffer, it's a good idea to set this to a large number comparable to the amount of data you're putting in.  The default is 1,024 bytes.
//     ///
//     ///
//     pub fn new_buffer(&self, data_type: &str, capacity: Option<usize>) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_new(buffer_rpc::none, data_type , NP_Memory::new(capacity,  self.schema.clone(), 0))
//     }

//     /// Generate a new empty buffer from this factory.
//     ///
//     /// Make sure the mutable slice is large enough to fit all the data you plan on putting into it.
//     ///
//     pub fn new_buffer_ref_mut(&self, data_type: &str, bytes: &mut [u8]) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_new(buffer_rpc::none, data_type, NP_Memory::new_ref_mut(bytes,  self.schema.clone(), 0))
//     }

//     /// Generate a new RPC request
//     ///
//     pub fn rpc_call<S: AsRef<str>>(&self, request_name: S) -> Result<NP_Buffer, NP_Error> {
//         NP_Buffer::_new(buffer_rpc::request, request_name.as_ref() , NP_Memory::new(None,  self.schema.clone(), 0))
//     }

//     /// Generate a new RPC response
//     ///
//     pub fn rpc_return(&self, request_buffer: &NP_Buffer) -> Result<NP_Buffer, NP_Error> {
//         request_buffer._generate_response_buffer(NP_Memory::new(None,  self.schema.clone(), 0))
//     }
// }


// #[test]
// fn test_args() {
//     let fact = NP_Factory::from_schema(r##"

//         struct myType<X, Y> [id: 0] {
//             username: string myName [some: "data"],
//             list: Vec<Vec<u32 [myDate: false]> [someDate: "there"]>,
//             something: X
//         }

//     "##).unwrap();

//     println!("{:?}", fact.inspect_schema("myType.username"));
// }


// #[test]
// fn threading_works() -> Result<(), NP_Error> {
//     let fact = NP_Factory::from_schema("").unwrap();
//     let buffer = fact.new_buffer("string", None)?;
//     std::thread::spawn(move || {
//         let f = fact.export_schema_bytes();
//         let b = buffer;
//         assert_eq!(6, b.calc_bytes().unwrap().current_buffer);
//         assert_eq!(8, f.len());
//     }).join().unwrap();
//
//     Ok(())
// }