#![warn(missing_docs)]
#![allow(non_camel_case_types)]
#![no_std]

//! ## Simple & Performant Serialization with RPC
//! Performance of Protocol Buffers with flexibility of JSON
//! 
//! [Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)
//! 
//! ### Features  
//! - Zero dependencies
//! - Zero copy deserialization
//! - `no_std` support, WASM ready
//! - Native byte-wise sorting
//! - Extensive Documentation & Testing
//! - Easily mutate, add or delete values in existing buffers
//! - Schemas allow default values and non destructive updates
//! - Supports most common native data types
//! - Supports collection types (list, map, table & tuple)
//! - Supports deep nesting of collection types
//! - Easy and performant export to JSON.
//! - [Thoroughly documented](https://docs.rs/no_proto/latest/no_proto/format/index.html) & simple data storage format
//! - Panic/unwrap() free, this library will never cause a panic in your application.
//! - Simple, powerful transport agnostic [RPC Framework](https://docs.rs/no_proto/latest/no_proto/rpc/index.html).
//! 
//! NoProto allows you to store, read & mutate structured data with very little overhead. It's like Protocol Buffers except schemas are dynamic at runtime and buffers are mutable.  It's like JSON but way faster, type safe and supports native types.  Also unlike Protocol Buffers you can insert values in any order and values can later be removed or updated without rebuilding the whole buffer.
//! 
//! Like Protocol Buffers schemas are seperate from the data buffers and are required to read, create or update data buffers.
//! 
//! Byte-wise sorting comes in the box and is a first class operation. Two NoProto buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 
//! 
//! *Compared to Protocol Buffers*
//! - Faster serialization & deserialization performance
//! - Updating buffers is orders of magnitude faster
//! - Easier & Simpler API
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Byte-wise sorting is first class operation
//! - Mutate (add/delete/update) existing/imported buffers
//! 
//! *Compared to JSON / BSON*
//! - Far more space efficient
//! - Significantly faster serialization & deserialization
//! - Deserializtion is zero copy
//! - Has schemas / type safe
//! - Supports byte-wise sorting
//! - Supports raw bytes & other native types
//! 
//! 
//! | Format           | Zero-Copy | Size Limit | Mutable | Schemas | Language Agnostic | No Compiling    | Byte-wise Sorting |
//! |------------------|-----------|------------|---------|---------|-------------------|-----------------|-------------------|
//! | **NoProto**      | ✓         | ~64KB      | ✓       | ✓       | ✓                 | ✓               | ✓                 |
//! | JSON             | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | BSON             | 𐄂         | ~16MB      | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | MessagePack      | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | FlatBuffers      | ✓         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
//! | Protocol Buffers | 𐄂         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
//! | Cap'N Proto      | ✓         | 2^64 Bytes | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
//! | Veriform         | 𐄂         | ?          | 𐄂       | 𐄂       | 𐄂                 | 𐄂               | 𐄂                 |
//! 
//! 
//! # Quick Example
//! ```rust
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::collection::table::NP_Table;
//! 
//! // JSON is used to describe schema for the factory
//! // Each factory represents a single schema
//! // One factory can be used to serialize/deserialize any number of buffers
//! let user_factory = NP_Factory::new(r#"{
//!     "type": "table",
//!     "columns": [
//!         ["name",   {"type": "string"}],
//!         ["age",    {"type": "u16", "default": 0}],
//!         ["tags",   {"type": "list", "of": {
//!             "type": "string"
//!         }}]
//!     ]
//! }"#)?;
//! 
//! 
//! // create a new empty buffer
//! let mut user_buffer = user_factory.empty_buffer(None); // optional capacity, optional address size (u16 by default)
//! 
//! // set an internal value of the buffer, set the  "name" column
//! user_buffer.set(&["name"], "Billy Joel")?;
//! 
//! // assign nested internal values, sets the first tag element
//! user_buffer.set(&["tags", "0"], "first tag")?;
//! 
//! // get an internal value of the buffer from the "name" column
//! let name = user_buffer.get::<&str>(&["name"])?;
//! assert_eq!(name, Some("Billy Joel"));
//! 
//! // close buffer and get internal bytes
//! let user_bytes: Vec<u8> = user_buffer.close();
//! 
//! // open the buffer again
//! let user_buffer = user_factory.open_buffer(user_bytes);
//! 
//! // get nested internal value, first tag from the tag list
//! let tag = user_buffer.get::<&str>(&["tags", "0"])?;
//! assert_eq!(tag, Some("first tag"));
//! 
//! // get nested internal value, the age field
//! let age = user_buffer.get::<u16>(&["age"])?;
//! // returns default value from schema
//! assert_eq!(age, Some(0u16));
//! 
//! // close again
//! let user_bytes: Vec<u8> = user_buffer.close();
//! 
//! 
//! // we can now save user_bytes to disk, 
//! // send it over the network, or whatever else is needed with the data
//! 
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ## Guided Learning / Next Steps:
//! 1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
//! 2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
//! 3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/struct.NP_Buffer.html) - How to create, update & compact buffers/data.
//! 4. [`RPC Framework`](https://docs.rs/no_proto/latest/no_proto/rpc/index.html) - How to use the RPC Framework APIs.
//! 5. [`Data & Schema Format`](https://docs.rs/no_proto/latest/no_proto/format/index.html) - Learn how data is saved into the buffer.
//! 
//! ## Benchmarks
//! While it's difficult to properly benchmark libraries like these in a fair way, I've made an attempt in the graph below.  These benchmarks are available in the `bench` folder and you can easily run them yourself with `cargo run --release`. 
//! 
//! The format and data used in the benchmarks were taken from the `flatbuffers` benchmarks github repo.  You should always benchmark/test your own use case for each library before making any decisions on what to use.
//! 
//! **Legend**: Ops / Millisecond, higher is better
//! 
//! | Library            | Encode | Decode All | Decode 1 | Update 1 | Size (bytes) | Size (Zlib) |
//! |--------------------|--------|------------|----------|----------|--------------|-------------|
//! | NoProto            | 312    | 469        | 27027    | 3953     | 284          | 229         |
//! | Protocol Buffers 2 | 270    | 390        | 400      | 167      | 220          | 163         |
//! | MessagePack        | 38     | 70         | 80       | 35       | 431          | 245         |
//! | JSON               | 167    | 134        | 167      | 127      | 673          | 246         |
//! | BSON               | 28     | 34         | 35       | 26       | 600          | 279         |
//! 
//! 
//! - **Encode**: Transfer a collection of 33 fields of test data into a serialized `Vec<u8>`.
//! - **Decode All**: Deserialize the test object from the `Vec<u8>` into all 33 fields.
//! - **Decode 1**: Deserialize the test object from the `Vec<u8>` into one field.
//! - **Update 1**: Deserialize, update a single field, then serialize back into `Vec<u8>`.
//! 
//! Complete benchmark source code is available [here](https://github.com/only-cliches/NoProto/tree/master/bench).
//! 
//! In my opinion the benchmarks above make NoProto the clear winner if you ever plan to mutate or update your buffer data.  If buffer data can always be immutable and the fixed compiled schemas aren't an issue, Flatbuffers is the better choice.
//! 
//! I also think there's a strong argument here against using data without a schema.  The cost of an entirely flexible formats like JSON or BSON is crazy.  Putting schemas on your data not only increases your data hygiene but makes the storage of the data far more comapct while increasing the deserialization and serialization perfomrance substantially.
//! 
//! #### Limitations
//! - Buffers cannot be larger than 2^16 bytes (~64kb).
//! - Collections (Lists, Maps, Tuples & Tables) cannot have more than 255 immediate child items.
//! - Enum/Option types are limited to 255 choices and choice strings cannot be larger than 255 bytes.
//! - Tables are limited to 255 columns and column names cannot be larger than 255 bytes.
//! - Buffers are not validated or checked before deserializing.
//! 
//! #### Non Goals / Known Tradeoffs 
//! If every CPU cycle counts, you don't mind compiling fixed schemas and you don't plan to mutate your buffers/objects, FlatBuffers/CapnProto is probably the way to go.  It's impossible to make a flexible format like NoProto as fast as formats that compile your schemas ahead of time and store data immutably.
//! 
//! ----------------------
//! 
//! MIT License
//! 
//! Copyright (c) 2020 Scott Lott
//! 
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//! 
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

pub mod pointer;
pub mod collection;
pub mod buffer;
pub mod schema;
pub mod error;
pub mod json_flex;
pub mod format;
pub mod memory;
#[cfg(feature = "np_rpc")]
pub mod rpc;
#[cfg(feature = "np_rpc")]
#[allow(missing_docs)]
#[doc(hidden)]
pub mod hashmap;
mod utils;

#[macro_use]
extern crate alloc;

use crate::schema::NP_Schema;
use crate::memory::NP_Memory;
// use crate::json_flex::NP_JSON;
// use crate::schema::NP_Schema;
use crate::json_flex::json_decode;
use crate::error::NP_Error;
use buffer::{NP_Buffer, DEFAULT_ROOT_PTR_ADDR};
use alloc::vec::Vec;
use alloc::string::String;
use memory::{NP_Memory_Writable};
use schema::NP_Parsed_Schema;

// BEGIN WASM CODE
extern crate wasm_bindgen;
extern crate wee_alloc;

use wasm_bindgen::prelude::*;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
// END WASM CODE

/// Factories are created from schemas.  Once you have a factory you can use it to create new buffers or open existing ones.
/// 
/// The easiest way to create a factory is to pass a JSON string schema into the static `new` method.  [Learn about schemas here.](./schema/index.html)
/// 
/// You can also create a factory with a compiled byte schema using the static `new_compiled` method.
/// 
/// # Example
/// ```
/// use no_proto::error::NP_Error;
/// use no_proto::NP_Factory;
/// 
/// let user_factory = NP_Factory::new(r#"{
///     "type": "table",
///     "columns": [
///         ["name",   {"type": "string"}],
///         ["pass",   {"type": "string"}],
///         ["age",    {"type": "uint16"}],
///         ["todos",  {"type": "list", "of": {"type": "string"}}]
///     ]
/// }"#)?;
/// 
/// // user_factory can now be used to make or open buffers that contain the data in the schema.
/// 
/// // create new buffer
/// let mut user_buffer = user_factory.empty_buffer(None); // optional capacity, optional address size
///    
/// // set the "name" column of the table
/// user_buffer.set(&["name"], "Billy Joel")?;
/// 
/// // set the first todo
/// user_buffer.set(&["todos", "0"], "Write a rust library.")?;
/// 
/// // close buffer 
/// let user_vec:Vec<u8> = user_buffer.close();
/// 
/// // open existing buffer for reading
/// let user_buffer_2 = user_factory.open_buffer(user_vec);
/// 
/// // read column value
/// let name_column = user_buffer_2.get::<&str>(&["name"])?;
/// assert_eq!(name_column, Some("Billy Joel"));
/// 
/// 
/// // read first todo
/// let todo_value = user_buffer_2.get::<&str>(&["todos", "0"])?;
/// assert_eq!(todo_value, Some("Write a rust library."));
/// 
/// // read second todo
/// let todo_value = user_buffer_2.get::<&str>(&["todos", "1"])?;
/// assert_eq!(todo_value, None);
/// 
/// 
/// // close buffer again
/// let user_vec: Vec<u8> = user_buffer_2.close();
/// // user_vec is a Vec<u8> with our data
/// 
/// # Ok::<(), NP_Error>(()) 
/// ```
/// 
/// ## Next Step
/// 
/// Read about how to use buffers to access, mutate and compact data.
/// 
/// [Go to NP_Buffer docs](./buffer/struct.NP_Buffer.html)
/// 
#[wasm_bindgen]
#[derive(Debug)]
pub struct NP_Factory {
    /// schema data used by this factory
    schema: NP_Schema,
    schema_bytes: NP_Schema_Bytes
}

/// The schema bytes container
#[derive(Debug)]
pub enum NP_Schema_Bytes {
    /// Owned bytes
    Owned(Vec<u8>)
}

/// When calling `maybe_compact` on a buffer, this struct is provided to help make a choice on wether to compact or not.
#[derive(Debug, Eq, PartialEq)]
pub struct NP_Size_Data {
    /// The size of the existing buffer
    pub current_buffer: usize,
    /// The estimated size of buffer after compaction
    pub after_compaction: usize,
    /// How many known wasted bytes in existing buffer
    pub wasted_bytes: usize
}

#[wasm_bindgen]
impl NP_Factory {
    
    /// Generate a new factory from the given schema.
    /// 
    /// This operation will fail if the schema provided is invalid or if the schema is not valid JSON.  If it fails you should get a useful error message letting you know what the problem is.
    /// 
    #[wasm_bindgen(constructor)]
    pub fn new(json_schema: String) -> Self {

        let parsed_value = json_decode(json_schema.into()).unwrap();

        let (is_sortable, schema_bytes, schema) = NP_Schema::from_json(Vec::new(), &parsed_value).unwrap();

        Self {
            schema_bytes: NP_Schema_Bytes::Owned(Vec::new()),
            schema:  NP_Schema {
                is_sortable: is_sortable,
                parsed: schema
            }
        }
    }

    /// Create a new factory from a compiled schema byte array.
    /// The byte schemas are at least an order of magnitude faster to parse than JSON schemas.
    /// 
    #[wasm_bindgen]
    pub fn new_compiled(schema_bytes: Vec<u8>) -> Self {
        
        let (is_sortable, schema) = NP_Schema::from_bytes(Vec::new(), 0, &schema_bytes);

        Self {
            schema_bytes: NP_Schema_Bytes::Owned(schema_bytes),
            schema:  NP_Schema { 
                is_sortable: is_sortable,
                parsed: schema
            }
        }
    }

    /// Get a copy of the compiled schema byte array
    /// 
    #[wasm_bindgen]
    pub fn compile_schema(&self) -> Vec<u8> {
        match &self.schema_bytes {
            NP_Schema_Bytes::Owned(x) => x.clone(),
        }
    }




    // /// Open existing Vec<u8> sortable buffer that was closed with `.close_sortable()` 
    // /// 
    // /// There is typically 10 bytes or more in front of every sortable buffer that is identical between all sortable buffers for a given schema.
    // /// 
    // /// This method is used to open buffers that have had the leading identical bytes trimmed from them using `.close_sortale()`.
    // /// 
    // /// This operation fails if the buffer is not sortable.
    // /// 
    // /// ```
    // /// use no_proto::error::NP_Error;
    // /// use no_proto::NP_Factory;
    // /// use no_proto::NP_Size_Data;
    // /// 
    // /// let factory: NP_Factory = NP_Factory::new(r#"{
    // ///    "type": "tuple",
    // ///    "sorted": true,
    // ///    "values": [
    // ///         {"type": "u8"},
    // ///         {"type": "string", "size": 6}
    // ///     ]
    // /// }"#)?;
    // /// 
    // /// let mut new_buffer = factory.empty_buffer(None);
    // /// // set initial value
    // /// new_buffer.set(&["0"], 55u8)?;
    // /// new_buffer.set(&["1"], "hello")?;
    // /// 
    // /// // the buffer with it's vtables take up 20 bytes!
    // /// assert_eq!(new_buffer.read_bytes().len(), 20usize);
    // /// 
    // /// // close buffer and get sortable bytes
    // /// let bytes: Vec<u8> = new_buffer.close_sortable()?;
    // /// // with close_sortable() we only get the bytes we care about!
    // /// assert_eq!([55, 104, 101, 108, 108, 111, 32].to_vec(), bytes);
    // /// 
    // /// // you can always re open the sortable buffers with this call
    // /// let new_buffer = factory.open_sortable_buffer(bytes)?;
    // /// assert_eq!(new_buffer.get(&["0"])?, Some(55u8));
    // /// assert_eq!(new_buffer.get(&["1"])?, Some("hello "));
    // /// 
    // /// # Ok::<(), NP_Error>(()) 
    // /// ```
    // /// 
    // /// 
    // #[wasm_bindgen]
    // pub fn open_sortable_buffer(&self, bytes: Vec<u8>) -> NP_Buffer {
        
    //     match &self.schema.parsed[0] {
    //         NP_Parsed_Schema::Tuple { values, sortable,  ..} => {
    //             if *sortable == false {
    //                 NP_Buffer::_new(NP_Memory_Writable::existing(bytes, self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR))
    //             } else {
    //                 let mut vtables = 1usize;
    //                 let mut length = values.len();
    //                 while length > 4 {
    //                     vtables +=1;
    //                     length -= 4;
    //                 }
    //                 // how many leading bytes are identical across all buffers with this schema
    //                 let root_offset = DEFAULT_ROOT_PTR_ADDR + 2 + (vtables * 10);

    //                 let default_buffer = NP_Buffer::_new(NP_Memory_Writable::new(Some(root_offset + bytes.len()), self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR));
    //                 let mut use_bytes = default_buffer.close()[0..root_offset].to_vec();
    //                 use_bytes.extend_from_slice(&bytes[..]);

    //                 NP_Buffer::_new(NP_Memory_Writable::existing(use_bytes, self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR))
    //             }
    //         },
    //         _ => NP_Buffer::_new(NP_Memory_Writable::existing(bytes, self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR))
    //     }
    // }


    // /// Open existing Vec<u8> as buffer for this factory.  
    // /// 
    // pub fn open_buffer(&self, bytes: Vec<u8>) -> NP_Buffer {
    //     NP_Buffer::_new(NP_Memory_Writable::existing(bytes, self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR))
    // }

    // /// Generate a new empty buffer from this factory.
    // /// 
    // /// The first opional argument, capacity, can be used to set the space of the underlying Vec<u8> when it's created.  If you know you're going to be putting lots of data into the buffer, it's a good idea to set this to a large number comparable to the amount of data you're putting in.  The default is 1,024 bytes.
    // /// 
    // /// The second optional argument, ptr_size, controls how much address space you get in the buffer and how large the addresses are.  Every value in the buffer contains at least one address, sometimes more.  `NP_Size::U16` (the default) gives you an address space of just over 16KB but is more space efficeint since the address pointers are only 2 bytes each.  `NP_Size::U32` gives you an address space of just over 4GB, but the addresses take up twice as much space in the buffer compared to `NP_Size::U16`.
    // /// You can change the address size through compaction after the buffer is created, so it's fine to start with a smaller address space and convert it to a larger one later as needed.  It's also possible to go the other way, you can convert larger address space down to a smaller one durring compaction.
    // /// 
    // pub fn empty_buffer(&self, capacity: Option<usize>) -> NP_Buffer {
    //     NP_Buffer::_new(NP_Memory_Writable::new(capacity, self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR))
    // }
}
