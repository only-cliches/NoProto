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
//! - Safely accept untrusted buffers
//! - `no_std` support, WASM ready
//! - Native byte-wise sorting
//! - Supports recursive data types
//! - Extensive Documentation & Testing
//! - Passes Miri compiler safety checks
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
//! *Compared to Apache Avro*
//! - Far more space efficient
//! - Significantly faster serialization & deserialization
//! - All values are optional (no void or null type)
//! - Supports more native types (like unsigned ints)
//! - Updates without deserializng/serializing
//! - Works with `no_std`.
//! - Safely handle untrusted data.
//! 
//! *Compared to Protocol Buffers*
//! - Comparable serialization & deserialization performance
//! - Updating buffers is an order of magnitude faster
//! - Schemas are dynamic at runtime, no compilation step
//! - All values are optional
//! - Supports more types and better nested type support
//! - Byte-wise sorting is first class operation
//! - Updates without deserializng/serializing
//! - Safely handle untrusted data.
//! 
//! *Compared to JSON / BSON*
//! - Far more space efficient
//! - Significantly faster serialization & deserialization
//! - Deserializtion is zero copy
//! - Has schemas / type safe
//! - Supports byte-wise sorting
//! - Supports raw bytes & other native types
//! - Updates without deserializng/serializing
//! - Works with `no_std`.
//! - Safely handle untrusted data.
//! 
//! *Compared to Flatbuffers / Bincode*
//! - Data types can change or be created at runtime
//! - Updating buffers is an order of magnitude faster
//! - Supports byte-wise sorting
//! - Updates without deserializng/serializing
//! - Works with `no_std`.
//! - Safely handle untrusted data.
//! 
//! 
//! | Format           | Zero-Copy | Size Limit | Mutable | Schemas | Language Agnostic | No Compiling    | Byte-wise Sorting |
//! |------------------|-----------|------------|---------|---------|-------------------|-----------------|-------------------|
//! | **NoProto**      | ✓         | ~64KB      | ✓       | ✓       | ✓                 | ✓               | ✓                 |
//! | Apache Avro      | 𐄂         | Unlimited  | 𐄂       | ✓       | ✓                 | ✓               | ✓                 |
//! | JSON             | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | BSON             | 𐄂         | ~16MB      | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | MessagePack      | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
//! | FlatBuffers      | ✓         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
//! | Bincode          | ✓         | ?          | ✓       | ✓       | 𐄂                 | 𐄂               | 𐄂                 |
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
//! let mut user_buffer = user_factory.empty_buffer(None); // optional capacity
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
//! 5. [`Data & Schema Format`](https://docs.rs/no_proto/latest/no_proto/format/index.html) - Learn how data is saved into the buffer and schemas.
//! 
//! ## Benchmarks
//! While it's difficult to properly benchmark libraries like these in a fair way, I've made an attempt in the graph below.  These benchmarks are available in the `bench` folder and you can easily run them yourself with `cargo run --release`. 
//! 
//! The format and data used in the benchmarks were taken from the `flatbuffers` benchmarks github repo.  You should always benchmark/test your own use case for each library before making any choices on what to use.
//! 
//! **Legend**: Ops / Millisecond, higher is better
//! 
//! | Library            | Encode | Decode All | Decode 1 | Update 1 | Size (bytes) | Size (Zlib) |
//! |--------------------|--------|------------|----------|----------|--------------|-------------|
//! | **Runtime Libs**   |        |            |          |          |              |             |
//! | *NoProto*          |   1057 |       1437 |    47619 |    12195 |          208 |         166 |
//! | Apache Avro        |    138 |         51 |       52 |       37 |          702 |         336 |
//! | FlexBuffers        |    401 |        855 |    23256 |      264 |          490 |         309 |
//! | JSON               |    550 |        438 |      544 |      396 |          439 |         184 |
//! | BSON               |    115 |        103 |      109 |       80 |          414 |         216 |
//! | MessagePack        |    135 |        222 |      237 |      119 |          296 |         187 |
//! | **Compiled Libs**  |        |            |          |          |              |             |
//! | Flatbuffers        |   1046 |      14706 |   250000 |     1065 |          264 |         181 |
//! | Bincode            |   5882 |       8772 |     9524 |     4016 |          163 |         129 |
//! | Protobuf           |    859 |       1140 |     1163 |      480 |          154 |         141 |
//! | Prost              |   1225 |       1866 |     1984 |      962 |          154 |         142 |
//! 
//! - **Encode**: Transfer a collection of fields of test data into a serialized `Vec<u8>`.
//! - **Decode All**: Deserialize the test object from the `Vec<u8>` into all fields.
//! - **Decode 1**: Deserialize the test object from the `Vec<u8>` into one field.
//! - **Update 1**: Deserialize, update a single field, then serialize back into `Vec<u8>`.
//! 
//! **Runtime VS Compiled Libs**: Some formats require data types to be compiled into the application, which increases performance but means data types *cannot change at runtime*.  If data types need to mutate during runtime or can't be known before the application is compiled (like with databases), you must use a format that doesn't compile data types into the application, like JSON or NoProto.
//! 
//! Complete benchmark source code is available [here](https://github.com/only-cliches/NoProto/tree/master/bench).
//! 
//! ## NoProto Strengths
//! If your use case fits any of the points below, NoProto is a good choice for your application.  You should always benchmark to verify.
//! 
//! 1. Flexible At Runtime<br/>
//! If you need to work with data types that will change or be created at runtime, you normally have to pick something like JSON since highly optimized formats like Flatbuffers and Bincode depend on compiling the data types into your application (making everything fixed at runtime). When it comes to formats that can change/implement data types at runtime, NoProto is fastest format I've been able to find (if you know if one that might be faster, let me know!).
//! 
//! 2. Safely Accept Untrusted Data</br>
//! The worse case failure mode for NoProto buffers is junk data.  While other formats can cause denial of service attacks or allow unsafe memory access, there is no such failure case with NoProto.  There is no way to construct a NoProto buffer that would cause any detrement in performance to the host application or lead to unsafe memory access.  Also, there is no panic causing code in the library, meaning it will never crash your application.
//! 
//! 3. Extremely Fast Updates<br/>
//! If you have a workflow in your application that is read -> modify -> write with buffers, NoProto will usually outperform every other format, including Bincode and Flatbuffers. This is because NoProto never actually deserializes, it doesn't need to. I wrote this library with databases in mind, if you want to support client requests like "change username field to X", NoProto will do this faster than any other format, usually orders of magnitude faster. This includes complicated mutations like "push a value onto the end of this nested list".
//! 
//! 4. Incremental Deserializing<br/>
//! You only pay for the fields you read, no more. There is no deserializing step in NoProto, opening a buffer typically performs no operations (except for sorted buffers, which is opt in). Once you start asking for fields, the library will navigate the buffer using the format rules to get just what you asked for and nothing else. If you have a workflow in your application where you read a buffer and only grab a few fields inside it, NoProto will outperform most other libraries.
//! 
//! 5. Bytewise Sorting<br/>
//! Almost all of NoProto's data types are designed to serialize into bytewise sortable values, *including signed integers*.  When used with Tuples, making database keys with compound sorting is extremly easy.  When you combine that with first class support for `UUID`s and `ULID`s NoProto makes an excellent tool for parsing and creating primary keys for databases like RocksDB, LevelDB and TiKV. 
//! 
//! 6. `no_std` Support<br/>
//! If you need a serialization format with low memory usage that works in `no_std` environments, NoProto is one of the few good choices.
//! 
//! 
//! ### When to use Flatbuffers / Bincode / CapN Proto
//! If you can safely compile all your data types into your application, all the buffers/data is trusted, and you don't intend to mutate buffers after they're created, Bincode/Flatbuffers/CapNProto is a better choice for you.
//! 
//! ### When to use JSON / BSON / MessagePack
//! If your data changes so often that schemas don't really make sense or the format you use must be self describing, JSON/BSON/MessagePack is a better choice.   Although I'd argue that if you *can* make schemas work you should.  Once you can use a format with schemas you save a ton of space in the resulting buffers and performance far better.
//! 
//! ## Limitations
//! - Collections (Map, Tuple, List & Table) cannot have more than 255 columns/items.  You can nest to get more capacity, for example a list of lists can have up to 255 * 255 items.
//! - You cannot nest more than 255 levels deep.
//! - Table colum names cannot be longer than 255 UTF8 bytes.
//! - Enum/Option types are limited to 255 options and each option cannot be more than 255 UTF8 Bytes.
//! - Map keys cannot be larger than 255 UTF8 bytes.
//! - Buffers cannot be larger than 2^16 bytes or ~64KB.
//! 
//! ----------------------
//! 
//! MIT License
//! 
//! Copyright (c) 2021 Scott Lott
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

#[cfg(test)]
#[macro_use]
extern crate std;


pub mod pointer;
pub mod collection;
pub mod buffer;
pub mod buffer_ro;
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

use core::ops::{Deref, DerefMut};
use crate::buffer_ro::NP_Buffer_RO;
use crate::memory::NP_Memory;
use crate::json_flex::NP_JSON;
use crate::schema::NP_Schema;
use crate::json_flex::json_decode;
use crate::error::NP_Error;
use buffer::{NP_Buffer, DEFAULT_ROOT_PTR_ADDR};
use alloc::vec::Vec;
use alloc::string::String;
use memory::{NP_Memory_ReadOnly, NP_Memory_Writable};
use schema::NP_Parsed_Schema;


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
#[derive(Debug)]
pub struct NP_Factory<'fact> {
    /// schema data used by this factory
    pub schema: NP_Schema,
    schema_bytes: NP_Schema_Bytes<'fact>
}

/// The schema bytes container
#[derive(Debug, Clone)]
pub enum NP_Schema_Bytes<'bytes> {
    /// Borrwed schema
    Borrwed(&'bytes [u8]),
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

impl<'fact> NP_Factory<'fact> {
    
    /// Generate a new factory from the given schema.
    /// 
    /// This operation will fail if the schema provided is invalid or if the schema is not valid JSON.  If it fails you should get a useful error message letting you know what the problem is.
    /// 
    pub fn new<S>(json_schema: S) -> Result<Self, NP_Error> where S: Into<String> {

        let parsed_value = json_decode(json_schema.into())?;

        let (is_sortable, schema_bytes, mut schema) = NP_Schema::from_json(Vec::new(), &parsed_value)?;

        schema = NP_Schema::resolve_portals(schema)?;

        Ok(Self {
            schema_bytes: NP_Schema_Bytes::Owned(schema_bytes),
            schema:  NP_Schema {
                is_sortable: is_sortable,
                parsed: schema
            }
        })      
        
    }

    /// Create a new factory from a compiled schema byte array.
    /// The byte schemas are at least an order of magnitude faster to parse than JSON schemas.
    /// 
    pub fn new_compiled(schema_bytes: &'fact [u8]) -> Result<Self, NP_Error> {
        
        let (is_sortable, mut schema) = NP_Schema::from_bytes(Vec::new(), 0, schema_bytes);

        schema = NP_Schema::resolve_portals(schema)?;

        Ok(Self {
            schema_bytes: NP_Schema_Bytes::Borrwed(schema_bytes),
            schema:  NP_Schema { 
                is_sortable: is_sortable,
                parsed: schema
            }
        })
    }

    /// Generate factory from *const [u8], probably not safe to use generally speaking
    #[doc(hidden)]
    pub unsafe fn new_compiled_ptr(schema_bytes: *const [u8]) -> Result<Self, NP_Error> {
        
        let (is_sortable, mut schema) = NP_Schema::from_bytes(Vec::new(), 0, &*schema_bytes );

        schema = NP_Schema::resolve_portals(schema)?;

        Ok(Self {
            schema_bytes: NP_Schema_Bytes::Borrwed(&*schema_bytes),
            schema:  NP_Schema { 
                is_sortable: is_sortable,
                parsed: schema
            }
        })
    }

    /// Get a copy of the compiled schema byte array
    /// 
    pub fn compile_schema(&self) -> &[u8] {
        match &self.schema_bytes {
            NP_Schema_Bytes::Owned(x) => x,
            NP_Schema_Bytes::Borrwed(x) => *x
        }
    }


    /// Exports this factorie's schema to JSON.  This works regardless of wether the factory was created with `NP_Factory::new` or `NP_Factory::new_compiled`.
    /// 
    pub fn export_schema(&self) -> Result<NP_JSON, NP_Error> {
        self.schema.to_json()
    }

    /// Open existing Vec<u8> sortable buffer that was closed with `.close_sortable()` 
    /// 
    /// There is typically 10 bytes or more in front of every sortable buffer that is identical between all sortable buffers for a given schema.
    /// 
    /// This method is used to open buffers that have had the leading identical bytes trimmed from them using `.close_sortale()`.
    /// 
    /// This operation fails if the buffer is not sortable.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "sorted": true,
    ///    "values": [
    ///         {"type": "u8"},
    ///         {"type": "string", "size": 6}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&["0"], 55u8)?;
    /// new_buffer.set(&["1"], "hello")?;
    /// 
    /// // the buffer with it's vtables take up 21 bytes!
    /// assert_eq!(new_buffer.read_bytes().len(), 21usize);
    /// 
    /// // close buffer and get sortable bytes
    /// let bytes: Vec<u8> = new_buffer.close_sortable()?;
    /// // with close_sortable() we only get the bytes we care about!
    /// assert_eq!([55, 104, 101, 108, 108, 111, 32].to_vec(), bytes);
    /// 
    /// // you can always re open the sortable buffers with this call
    /// let new_buffer = factory.open_sortable_buffer(bytes)?;
    /// assert_eq!(new_buffer.get(&["0"])?, Some(55u8));
    /// assert_eq!(new_buffer.get(&["1"])?, Some("hello "));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// 
    pub fn open_sortable_buffer<'buffer>(&'buffer self, bytes: Vec<u8>) -> Result<NP_Buffer<'buffer>, NP_Error> {
        
        match &self.schema.parsed[0] {
            NP_Parsed_Schema::Tuple { values, sortable,  ..} => {
                if *sortable == false {
                    Err(NP_Error::new("Attempted to open sorted buffer when root wasn't sortable!"))
                } else {
                    let mut vtables = 1usize;
                    let mut length = values.len();
                    while length > 4 {
                        vtables +=1;
                        length -= 4;
                    }
                    // how many leading bytes are identical across all buffers with this schema
                    let root_offset = DEFAULT_ROOT_PTR_ADDR + 2 + (vtables * 10);

                    let default_buffer = NP_Buffer::_new(NP_Memory_Writable::new(Some(root_offset + bytes.len()), &self.schema.parsed, DEFAULT_ROOT_PTR_ADDR));
                    let mut use_bytes = default_buffer.close()[0..root_offset].to_vec();
                    use_bytes.extend_from_slice(&bytes[..]);

                    Ok(NP_Buffer::_new(NP_Memory_Writable::existing(use_bytes, &self.schema.parsed, DEFAULT_ROOT_PTR_ADDR)))
                }
            },
            _ => return Err(NP_Error::new("Attempted to open sorted buffer when root wasn't tuple!"))
        }
    }


    /// Open existing Vec<u8> as buffer for this factory.  
    /// 
    pub fn open_buffer<'buffer>(&'buffer self, bytes: Vec<u8>) -> NP_Buffer<'buffer> {
        NP_Buffer::_new(NP_Memory_Writable::existing(bytes, &self.schema.parsed, DEFAULT_ROOT_PTR_ADDR))
    }

    /// Open existing buffer as ready only, much faster if you don't need to mutate anything.
    /// 
    /// Also, read only buffers are `Sync` and `Send` so good for multithreaded environments.
    /// 
    pub fn open_buffer_ro<'buffer>(&'buffer self, bytes: &'buffer [u8]) -> NP_Buffer_RO<'buffer> {
        NP_Buffer_RO::_new(NP_Memory_ReadOnly::existing(bytes, &self.schema.parsed, DEFAULT_ROOT_PTR_ADDR))
    }

    /// Generate a new empty buffer from this factory.
    /// 
    /// The first opional argument, capacity, can be used to set the space of the underlying Vec<u8> when it's created.  If you know you're going to be putting lots of data into the buffer, it's a good idea to set this to a large number comparable to the amount of data you're putting in.  The default is 1,024 bytes.
    /// 
    /// The second optional argument, ptr_size, controls how much address space you get in the buffer and how large the addresses are.  Every value in the buffer contains at least one address, sometimes more.  `NP_Size::U16` (the default) gives you an address space of just over 16KB but is more space efficeint since the address pointers are only 2 bytes each.  `NP_Size::U32` gives you an address space of just over 4GB, but the addresses take up twice as much space in the buffer compared to `NP_Size::U16`.
    /// You can change the address size through compaction after the buffer is created, so it's fine to start with a smaller address space and convert it to a larger one later as needed.  It's also possible to go the other way, you can convert larger address space down to a smaller one durring compaction.
    /// 
    pub fn empty_buffer<'buffer>(&'buffer self, capacity: Option<usize>) -> NP_Buffer<'buffer> {
        NP_Buffer::_new(NP_Memory_Writable::new(capacity, &self.schema.parsed, DEFAULT_ROOT_PTR_ADDR))
    }

    /// Convert a regular buffer into a packed buffer. A "packed" buffer contains the schema and the buffer data together.
    /// 
    /// You can optionally store buffers with their schema attached so you don't have to track the schema seperatly.
    /// 
    /// The schema is stored in a very compact, binary format.  A JSON version of the schema can be generated from the binary version at any time.
    /// 
    pub fn pack_buffer<'open>(&self, buffer: NP_Buffer) -> NP_Packed_Buffer<'open> {
        NP_Packed_Buffer {
            buffer: NP_Buffer::_new(NP_Memory_Writable::existing_owned(buffer.close(), self.schema.parsed.clone(), DEFAULT_ROOT_PTR_ADDR)),
            schema_bytes: self.compile_schema().to_vec(),
            schema: self.schema.clone()
        }
    }
}

/// Packed Buffer Container
pub struct NP_Packed_Buffer<'packed> {
    buffer: NP_Buffer<'packed>,
    schema_bytes: Vec<u8>,
    /// Schema data for this packed buffer
    pub schema: NP_Schema
}

impl<'packed> NP_Packed_Buffer<'packed> {

    /// Open a packed buffer
    pub fn open(buffer: Vec<u8>) -> Result<Self, NP_Error> {
        if buffer[0] != 1 {
            return Err(NP_Error::new("Trying to use NP_Packed_Buffer::open on non packed buffer!"))
        }

        let schema_len = u16::from_be_bytes(unsafe { *((&buffer[1..3]) as *const [u8] as *const [u8; 2]) }) as usize;

        let schema_bytes = &buffer[3..(3 + schema_len)];

        let (is_sortable, mut schema) = NP_Schema::from_bytes(Vec::new(), 0, schema_bytes);

        schema = NP_Schema::resolve_portals(schema)?;

        let buffer_bytes = &buffer[(3 + schema_len)..];

        Ok(Self {
            buffer: NP_Buffer::_new(NP_Memory_Writable::existing_owned(buffer_bytes.to_vec(), schema.clone(), DEFAULT_ROOT_PTR_ADDR)),
            schema_bytes: schema_bytes.to_vec(),
            schema: NP_Schema {
                is_sortable: is_sortable,
                parsed: schema
            }
        })
    }

    /// Close this buffer and pack it
    pub fn close_packed(self) -> Vec<u8> {
        let mut new_buffer: Vec<u8> = Vec::new();
        new_buffer.push(1); // indicate this is a packed buffer
        let schema = self.compile_schema();
        // schema size
        new_buffer.extend_from_slice(&(schema.len() as u16).to_be_bytes());
        // schema data
        new_buffer.extend_from_slice(self.compile_schema());
        // buffer data
        new_buffer.extend(self.buffer.close());
        new_buffer
    }

    /// Convert this packed buffer into a regular buffer
    pub fn into_buffer(self) -> NP_Buffer<'packed> {
        self.buffer
    }

    /// Get the schema bytes for this packed buffer
    pub fn compile_schema(&self) -> &[u8] {
        &self.schema_bytes[..]
    }
}

impl<'packed> Deref for NP_Packed_Buffer<'packed> {
    type Target = NP_Buffer<'packed>;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<'packed> DerefMut for NP_Packed_Buffer<'packed> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}