#![warn(missing_docs, missing_debug_implementations)]
#![allow(non_camel_case_types)]
// #![no_std]

//! ## High Performance Serialization Library
//! Faster than JSON with Schemas and Native Types.  Like Mutable Protocol Buffers with Compile Free Schemas.
//! 
//! [Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)
//! 
//! ### Features  
//! - Zero dependencies
//! - #![no_std] support, WASM ready
//! - Supports bytewise sorting of buffers
//! - Thorough Documentation
//! - Automatic & instant serilization
//! - Nearly instant deserialization
//! - Schemas are dynamic/flexible at runtime
//! - Mutate/Insert/Delete values in existing buffers
//! - Supports native data types
//! - Supports collection types (list, map, table & tuple)
//! - Supports deep nesting of collection types
//! 
//! NoProto allows you to store, read & mutate structured data with near zero overhead. It's like Cap'N Proto/Flatbuffers except buffers and schemas are dynamic at runtime instead of requiring compilation.  It's like JSON but faster, type safe and allows native types.
//! 
//! Bytewise sorting comes in the box and is a first class operation. The result is two NoProto buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 
//! 
//! NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.
//! 
//! *Compared to FlatBuffers / Cap'N Proto / Protocol Buffers*
//! - Comparable serialization & deserialization performance
//! - Easier & Simpler API
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Bytewise sorting is first class operation
//! - Mutate (add/delete/update) existing/imported buffers
//! 
//! *Compared to JSON*
//! - Far more space efficient
//! - Faster serialization & deserialization
//! - Has schemas / type safe
//! - Supports bytewise sorting
//! - Supports raw bytes & other native types
//! 
//! *Compared to BSON*
//! - Far more space efficient
//! - Faster serialization & deserialization
//! - Has schemas / type safe
//! - Bytewise sorting is first class operation
//! - Supports much larger documents (4GB vs 16KB)
//! - Better collection support & more supported types
//! 
//! *Compared to Serde*
//! - Supports bytewise sorting
//! - Objects & schemas are dynamic at runtime
//! - Faster serialization & deserialization
//! - Language agnostic
//! 
//! | Format           | Free De/Serialization | Size Limit | Mutatable | Schemas | Language Agnostic | No Compiling    | Bytewise Sorting |
//! |------------------|-----------------------|------------|-----------|---------|-------------------|-----------------|------------------|
//! | **NoProto**      | ✓                     | ~4GB       | ✓         | ✓       | ✓                 | ✓               | ✓                |
//! | JSON             | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | BSON             | 𐄂                     | ~16KB      | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | MessagePack      | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | FlatBuffers      | ✓                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Protocol Buffers | 𐄂                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Cap'N Proto      | ✓                     | 2^64 Bytes | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Serde            | 𐄂                     | ?          | 𐄂         | ✓       | 𐄂                 | 𐄂               | 𐄂                |
//! 
//! 
//! #### Limitations
//! - Buffers cannot be larger than 2^32 bytes (~4GB).
//! - Lists cannot have more than 65,535 items.
//! - Enum/Option types are limited to 255 choices.
//! - Tables are limited to 255 columns.
//! - Tuple types are limited to 255 items.
//! - Buffers are not validated or checked before deserializing.
//! 
//! # Quick Example
//! ```rust
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::collection::table::NP_Table;
//! use no_proto::pointer::NP_Ptr;
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
//! let mut user_buffer = user_factory.empty_buffer(None, None); // optional capacity, optional address size (u16 by default)
//! 
//! // set an internal value of the buffer, set the  "name" column
//! user_buffer.deep_set("name", String::from("Billy Joel"))?;
//! 
//! // assign nested internal values, sets the first tag element
//! user_buffer.deep_set("tags.0", String::from("first tag"))?;
//! 
//! // get an internal value of the buffer from the "name" column
//! let name = user_buffer.deep_get::<String>("name")?;
//! assert_eq!(name, Some(Box::new(String::from("Billy Joel"))));
//! 
//! // close buffer and get internal bytes
//! let user_bytes: Vec<u8> = user_buffer.close();
//! 
//! // open the buffer again
//! let user_buffer = user_factory.open_buffer(user_bytes);
//! 
//! // get nested internal value, first tag from the tag list
//! let tag = user_buffer.deep_get::<String>("tags.0")?;
//! assert_eq!(tag, Some(Box::new(String::from("first tag"))));
//! 
//! // get nested internal value, the age field
//! let age = user_buffer.deep_get::<u16>("age")?;
//! // returns default value from schema
//! assert_eq!(age, Some(Box::new(0u16)));
//! 
//! // close again
//! let user_bytes: Vec<u8> = user_buffer.close();
//! 
//! // we can now save user_bytes to disk, 
//! // send it over the network, or whatever else is needed with the data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ## Guided Learning / Next Steps:
//! 1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
//! 2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
//! 3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/struct.NP_Buffer.html) - How to create, update & compact buffers.
//! 4. [`Pointers`](https://docs.rs/no_proto/latest/no_proto/pointer/struct.NP_Ptr.html) - How to add, remove and edit values in a buffer.
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
mod memory;
mod utils;

extern crate alloc;

use crate::schema::NP_Schema;
use crate::json_flex::json_decode;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use buffer::{NP_Buffer};
use alloc::vec::Vec;
use alloc::{rc::Rc, borrow::ToOwned};
use memory::NP_Size;

const PROTOCOL_VERSION: u8 = 1;


/// Factories are created from schemas.  Once you have a factory you can use it to create new buffers or open existing ones.
/// 
/// The correct way to create a factory is to pass a JSON string schema into the static `new` method.  [Learn about schemas here.](./schema/index.html)
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
/// let mut user_buffer = user_factory.empty_buffer(None, None); // optional capacity, optional size
///    
/// // set the "name" column of the table
/// user_buffer.deep_set("name", "Billy Joel".to_owned())?;
/// 
/// // set the first todo
/// user_buffer.deep_set("todos.0", "Write a rust library.".to_owned())?;
/// 
/// // close buffer 
/// let user_vec:Vec<u8> = user_buffer.close();
/// 
/// // open existing buffer for reading
/// let user_buffer_2 = user_factory.open_buffer(user_vec);
/// 
/// // read column value
/// let name_column = user_buffer_2.deep_get::<String>("name")?;
/// assert_eq!(name_column, Some(Box::new("Billy Joel".to_owned())));
/// 
/// // read first todo
/// let todo_value = user_buffer_2.deep_get::<String>("todos.0")?;
/// assert_eq!(todo_value, Some(Box::new("Write a rust library.".to_owned())));
/// 
/// // read second todo
/// let todo_value = user_buffer_2.deep_get::<String>("todos.1")?;
/// assert_eq!(todo_value, None);
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
pub struct NP_Factory {
    schema: Rc<NP_Schema>
}

impl NP_Factory {
    
    /// Generate a new factory from the given schema.
    /// 
    /// This operation will fail if the schema provided is invalid or if the schema is not valid JSON.  If it fails you should get a useful error message letting you know what the problem is.
    /// 
    pub fn new(json_schema: &str) -> Result<NP_Factory, NP_Error> {

        let parsed = json_decode(json_schema.to_owned());

        match parsed {
            Ok(good_parsed) => {
                Ok(NP_Factory {
                    schema:  Rc::new(NP_Schema::from_json(good_parsed)?)
                })
            },
            Err(_x) => {
                Err(NP_Error::new("Schema JSON Parse Error"))
            }
        }
    }

    /// Open existing Vec<u8> as buffer for this factory.  
    /// This just moves the Vec<u8> into the buffer object, no deserialization is done here.
    /// 
    pub fn open_buffer(&self, bytes: Vec<u8>) -> NP_Buffer {
        NP_Buffer::_new(Rc::clone(&self.schema), Rc::new(NP_Memory::existing(bytes)))
    }

    /// Generate a new empty buffer from this factory.
    /// 
    /// The first opional argument, capacity, can be used to set the space of the underlying Vec<u8> when it's created.  If you know you're going to be putting lots of data into the buffer, it's a good idea to set this to a large number comparable to the amount of data you're putting in.  The default is 1,024 bytes.
    /// 
    /// The second optional argument, ptr_size, controls how much address space you get in the buffer.  `NP_Size::U16` (the default) gives you an address space of just over 16KB but is more space efficeint since the address pointers are only 2 bytes each.  `NP_Size::U32` gives you an address space of just over 4GB, but the addresses take up twice as much space in the buffer.
    /// You can change the address size through compaction after the buffer is created, so it's fine to start with a smaller address space and convert it to a larger one later as needed.  It's also possible to go the other way, you can convert larger address space down to a smaller one durring compaction.
    /// 
    pub fn empty_buffer(&self, capacity: Option<usize>, ptr_size: Option<NP_Size>) -> NP_Buffer {
        let use_size = match ptr_size {
            Some(x) => x,
            None => NP_Size::U16
        };
        NP_Buffer::_new(Rc::clone(&self.schema), Rc::new(NP_Memory::new(capacity, use_size)))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use collection::{table::NP_Table, list::NP_List};
    use json_flex::NP_JSON;
    // use pointer::misc::NP_Date;

    #[test]
    fn it_works() -> core::result::Result<(), NP_Error> {


        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "list",
            "of": {
                "type": "table",
                "columns": [
                    ["name", {"type": "string", "default": "no name"}],
                    ["age",  {"type": "i16", "default": 10}]
                ]
            }
        }"#)?;

        let mut new_buffer = factory.empty_buffer(None, None);

        new_buffer.open::<NP_List<NP_Table>>(&mut |_list| {

            Ok(())
        })?;

        new_buffer.deep_set("10.name", "something".to_owned())?;
        // new_buffer.deep_set("10.name", "someth\"ing22".to_owned())?;
        new_buffer.deep_set("9.age", -29383i16)?;
        println!("Size: {:?}", new_buffer.calc_bytes()?);
        // new_buffer.compact(None, None)?;
        println!("Size: {:?}", new_buffer.calc_bytes()?);

        // println!("JSON: {}", new_buffer.json_encode().stringify());
        
        let value = new_buffer.deep_get::<NP_JSON>("9")?;

        println!("name: {}", value.unwrap().stringify());

        println!("BYTES: {:?}", new_buffer.close());

        // let buffer2 = factory.deep_set::<String>(return_buffer, "15", "hello, world".to_owned())?;

        // println!("value {:?}", factory.deep_get::<String>(return_buffer, "10.name")?);
 
        Ok(())
    }
    
}
