// #![deny(missing_docs, missing_debug_implementations, trivial_casts, trivial_numeric_casts, unused_results)]
#![allow(non_camel_case_types)]
#![no_std]

//! ## High Performance Serialization Library
//! 
//! [Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)
//! 
//! ### TODO: 
//! - [x] Finish implementing Lists, Tuples & Maps
//! - [x] Collection Iterator
//! - [ ] Compaction
//! - [ ] Documentation
//! - [ ] Tests
//! 
//! ### Features
//! - Zero dependencies
//! - #![no_std] support, WASM ready
//! - Supports bytewise sorting of buffers
//! - Automatic & instant deserilization
//! - Nearly instant serialization
//! - Schemas are dynamic/flexible at runtime
//! - Mutate/Update/Delete values in existing buffers
//! - Supports native data types
//! - Supports collection types (list, map, table & tuple)
//! - Supports deep nesting of collection types
//! 
//! NoProto allows you to store, read & mutate structured data with near zero overhead.  It's like JSON but faster, type safe and allows native types.  It's like Cap'N Proto/Flatbuffers except buffers and schemas are dynamic at runtime instead of requiring compilation. 
//! 
//! Bytewise sorting comes in the box and is a first class operation. The result is two NoProto buffers can be compared at the byte level *without serializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 
//! 
//! NoProto moves the cost of serialization to the access methods instead of serializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.
//! 
//! *Compared to FlatBuffers / Cap'N Proto*
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Bytewise sorting is explicitly supported
//! - Mutate (add/delete/update) existing/imported buffers
//! 
//! *Compared to JSON*
//! - Has schemas / type safe
//! - Supports bytewise sorting
//! - Faster serialization & deserialization
//! - Supports raw bytes & other native types
//! 
//! *Compared to BSON*
//! - Faster serialization & deserialization
//! - Has schemas / type safe
//! - Bytewise sorting is explicitly supported
//! - Supports much larger documents (4GB vs 16MB)
//! - Better collection support & more supported types
//! 
//! *Compared to Serde*
//! - Supports bytewise sorting
//! - Objects & schemas are dynamic at runtime
//! - Faster serialization & deserialization
//! 
//! | Format           | Free De/Serialization | Size Limit | Mutatable | Schemas | Language Agnostic | Runtime Dynamic | Bytewise Sorting |
//! |------------------|-----------------------|------------|-----------|---------|-------------------|-----------------|------------------|
//! | JSON             | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | BSON             | 𐄂                     | ~16KB      | ✓         | 𐄂       | ✓                 | ✓               | ✓*               |
//! | MessagePack      | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | ✓*               |
//! | FlatBuffers      | ✓                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | ✓*               |
//! | Protocol Buffers | 𐄂                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | ✓*               |
//! | Cap'N Proto      | ✓                     | 2^64 Bytes | 𐄂         | ✓       | ✓                 | 𐄂               | ✓*               |
//! | Serde            | 𐄂                     | ?          | ✓         | ✓       | 𐄂                 | 𐄂               | 𐄂                |
//! | **NoProto**      | ✓                     | ~4GB       | ✓         | ✓       | ✓                 | ✓               | ✓                |
//! 
//! \* Bytewise sorting can *technically* be achieved with these libraries.  However, it's not a first class operation and requires extra effort, configuration and care.
//! 
//! #### Limitations
//! - Buffers cannot be larger than 2^32 bytes (~4GB).
//! - Tables & List collections cannot have more than 2^16 items (~16k).
//! - Enum/Option types are limited to 2^8 or 255 choices.
//! - Tuple types are limited to 2^8 or 255 items.
//! - Buffers are not validated or checked before deserializing.
//! 
//! # Quick Example
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::NP;
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
//!         ["pass",   {"type": "string"}],
//!         ["age",    {"type": "uint16"}]
//!     ]
//! }"#)?;
//! 
//! // creating a new buffer from the `user_factory` schema
//! // user_buffer contains a deserialized Vec<u8> containing our data
//! 
//! let user_buffer: Vec<u8> = user_factory.open(NP::new, |mut buffer| {
//!    
//!     // open the buffer to read or update values
//!     let root: NP_Ptr<NP_Table> = buffer.root()?;  // <- type cast the root
//!         
//!    // the root of our schema is a collection type (NP_Table), 
//!    // so we have to collapse the root pointer into the collection type.
//!    let mut table: NP_Table = root.into()?.unwrap();
//! 
//!    // Select a column and type cast it. Selected columns can be mutated or read from.
//!    let mut user_name = table.select::<String>("name")?;
//! 
//!    // set value of name column
//!    user_name.set("some name".to_owned())?;
//! 
//!    // select age column and set it's value
//!    let mut age = table.select::<u16>("age")?;
//!    age.set(75)?;
//!
//!    // done mutating/reading the buffer
//!    Ok(())
//! })?;
//!  
//! // open the new buffer, `user_buffer`, we just created
//! // user_buffer_2 contains the deserialized Vec<u8>
//! let user_buffer_2: Vec<u8> = user_factory.open(NP::bytes(user_buffer), |mut buffer| {
//! 
//!    let root: NP_Ptr<NP_Table> = buffer.root()?;
//!         
//!    // get the table root again
//!    let mut table = root.into()?.unwrap();
//! 
//!    // read the name column
//!    let mut user_name = table.select::<String>("name")?;
//!    assert_eq!(user_name.get()?, Some(String::from("some name")));
//! 
//!    // password value will be None since we haven't set it.
//!    let mut password = table.select::<String>("pass")?;
//!    assert_eq!(password.get()?, None);
//! 
//!    // read age value    
//!    let mut age = table.select::<u16>("age")?;
//!    assert_eq!(age.get()?, Some(75));    
//! 
//!    // done with the buffer
//!    Ok(())
//! })?;
//! 
//! // we can now save user_buffer_2 to disk, 
//! // send it over the network, or whatever else is needed with the data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ## Guided Learning / Next Steps:
//! 1. Schemas - Learn how to build & work with schemas.
//! 2. Factories - Parsing schemas into something you can work with.
//! 3. Buffers - How to create, update & compact buffers.
//! 
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

use core::marker::PhantomData;
use crate::pointer::NP_Value;
use crate::json_flex::json_decode;
use crate::error::NP_Error;
use crate::schema::NP_Schema;
use crate::memory::NP_Memory;
use buffer::NP_Buffer;
use alloc::vec::Vec;
use alloc::vec;
use alloc::borrow::ToOwned;
use json_flex::JFObject;
use pointer::{any::NP_Any, NP_Ptr, NP_ValueInto};

const PROTOCOL_VERSION: u8 = 0;


/// Factories allow you to serialize and deserialize buffers.
/// 
/// Each factory represents a schema, each factory can be reused for any number of buffers based on the factory's schema.
/// 
/// # Example
/// ```
/// 
/// 
/// ```
pub struct NP_Factory<'a> {
    schema: NP_Schema,
    phantom: &'a PhantomData<u8>
}

pub enum NP {
    bytes(Vec<u8>),
    size(usize),
    new
}

impl<'a> NP_Factory<'a> {
    pub fn new(json_schema: &str) -> core::result::Result<NP_Factory, NP_Error> {


        let parsed = json_decode(json_schema.to_owned());

        match parsed {
            Ok(good_parsed) => {
                Ok(NP_Factory {
                    schema:  NP_Schema::from_json(good_parsed)?,
                    phantom: &PhantomData
                })
            },
            Err(_x) => {
                Err(NP_Error::new("JSON Parse Error"))
            }
        }
    }

    pub fn open<F>(&self, buffer: NP, mut callback: F) -> Result<Vec<u8>, NP_Error>
        where F: FnMut(NP_Buffer) -> Result<(), NP_Error>
    {   
        let use_buffer = match buffer {
            NP::bytes(x) => x,
            NP::size(x) => {
                self.new_buffer(Some(x))
            }
            NP::new => {
                self.new_buffer(None)
            }
        };

        let bytes = NP_Memory::new(use_buffer);

        callback(NP_Buffer::new(&self.schema, &bytes))?;

        Ok(bytes.dump())
    }

    pub fn new_buffer(&self, size: Option<usize>) -> Vec<u8> {

        let use_size = match size {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        new_bytes.extend(vec![
            PROTOCOL_VERSION, // Protocol version (for breaking changes if needed later)
            0, 0, 0, 0        // u32 HEAD for root pointer (starts at zero)
        ]); 

        new_bytes
    }

    pub fn json_encode(&self, buffer: Vec<u8>) -> (JFObject, Vec<u8>) {
        let bytes = NP_Memory::new(buffer);

        let root = NP_Ptr::<NP_Any>::new_standard_ptr(1, &self.schema, &bytes);

        (root.json_encode(), bytes.dump())
    }

    pub fn extract<T, F>(&self, buffer: Vec<u8>, mut callback: F) -> Result<(T, Vec<u8>), NP_Error> 
        where F: FnMut(NP_Buffer) -> Result<T, NP_Error>
    {
        let bytes = NP_Memory::new(buffer);

        let result = callback(NP_Buffer::new(&self.schema, &bytes))?;

        Ok((result, bytes.dump()))
    }

    pub fn set<X: NP_Value + Default, S: AsRef<str>>(&self, buffer: Vec<u8>, path: S, value: X) -> Result<(bool, Vec<u8>), NP_Error> {
        let bytes = NP_Memory::new(buffer);

        let result = {
            let buffer = NP_Buffer::new(&self.schema, &bytes);

            buffer.set_value(path, value)
        }?;

        Ok((result, bytes.dump()))
    }

    pub fn get<X: NP_Value + Default, S: AsRef<str>>(&self, buffer: Vec<u8>,  path: S) -> Result<(Option<X>, Vec<u8>), NP_Error> {
        let bytes = NP_Memory::new(buffer);

        let result = {
            let buffer = NP_Buffer::new(&self.schema, &bytes);

            buffer.get_value(path)
        }?;

        Ok((result, bytes.dump()))
    } 
}

#[cfg(test)]
mod tests {

    use crate::pointer::NP_Ptr;
    use crate::collection::table::NP_Table;
    use super::*;
    use collection::{map::NP_Map, list::NP_List};

    #[test]
    fn it_works() -> core::result::Result<(), NP_Error> {

        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "float"
        }"#)?;

        /*
        let return_buffer = factory.open(NP::new, |mut buffer| {

            let mut root: NP_Ptr<f32> = buffer.root()?;

            root.set(1.0)?;

            Ok(())
        })?;

        println!("BYTES: {:?}", return_buffer);

        let return_buffer_2 = factory.open(NP::new, |mut buffer| {

            let mut root: NP_Ptr<f32> = buffer.root()?;

            root.set(0.5)?;

            Ok(())
        })?;

        let return_buffer_2 = factory.open(NP::bytes(return_buffer_2), |mut buffer| {

            let mut root: NP_Ptr<f32> = buffer.root()?;

            println!("VALUE {:?}", root.get()?);

            Ok(())
        })?;

        let va = factory.extract(return_buffer_2, |mut buffer| {
            let mut root: NP_Ptr<f32> = buffer.root()?;
            Ok((root.get()?, 0f32))
        });

        println!("BYTES: {:?}", return_buffer_2);

        println!("GT {:?}", return_buffer_2 < return_buffer);

        let json = factory.json_encode(return_buffer_2);
        println!("JSON {:?}", json.0.to_json());

   

        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "table",
            "columns": [
                ["userID",  {"type": "string"}],
                ["pass",    {"type": "string"}],
                ["age",     {"type": "uint16"}],
                ["colors",  {"type": "list", "of": {
                    "type": "string"
                }}],
                ["meta",    {"type": "map", "value": {"type": "string"}}]
            ]
        }"#)?;

        let mut myvalue: Option<String> = None;

        let return_buffer = factory.new_buffer(None, |mut buffer| {

            // buffer.deep_set(".userID", "something".to_owned())?;

            // myvalue = buffer.deep_get::<String>(".userID")?;

            let root: NP_Ptr<NP_Table> = buffer.root()?;

            let mut table = root.into()?.unwrap();

            let mut x = table.select::<String>("userID")?;
            x.set("username".to_owned())?;
    
            let mut x = table.select::<String>("pass")?;
            x.set("password123 hello".to_owned())?;

            myvalue = x.get()?;

            let mut color = table.select::<NP_List<String>>("colors")?.into()?.unwrap();

            let mut first_test_item = color.select(20)?;

            first_test_item.set("blue".to_owned())?;

            let mut second_test_item = color.select(10)?;

            second_test_item.set("orange".to_owned())?;

            // let mut x = table.select::<u16>("age")?;
            // x.set(1039)?;

            let mut meta = table.select::<NP_Map<String>>("meta")?.into()?.unwrap();

            meta.select(&"some key".to_string().as_bytes().to_vec())?.set("some value".to_string())?;

            // println!("VALUE 0: {:?}", table.select::<u16>("age")?.get()?);

            Ok(())
        })?;

        let return_buffer_2 = factory.load_buffer(return_buffer, |mut buffer| {

            let root: NP_Ptr<NP_Table> = buffer.root()?;
            
            let mut table = root.into()?.unwrap();

            let mut color = table.select::<NP_List<String>>("colors")?.into()?.unwrap();

            let mut first_test_item = color.select(20)?;

            println!("BLUE: {:?}", first_test_item.get()?);

            let mut second_test_item = color.select(10)?;

            println!("ORANGE: {:?}", second_test_item.get()?);

            println!("i10: {:?}", color.has(10)?);
            println!("i15: {:?}", color.has(15)?);

            color.push()?.0.set("hello, world!".to_owned())?;
            color.push()?.0.set("hello, world! 3".to_owned())?;
            color.push()?.0.set("hello, world! 2".to_owned())?;

            color.select(5)?.set("hello".to_owned())?;

            color.shift()?;

            color.debug(|i, addr, next| {
                println!("I: {}, ADDR: {}, NEXT: {}", i, addr, next);
            })?;

            println!("Value 21 GET: {:?}", color.select(21)?.get()?);
            println!("Value 22 GET: {:?}", color.select(22)?.get()?);
            println!("LENGTH: {:?}", color.len());

            let mut meta = table.select::<NP_Map<String>>("meta")?.into()?.unwrap();

            println!("Some Key: {:?}", meta.select(&"some key".to_string().as_bytes().to_vec())?.get()?);

            let mut x = table.select::<String>("userID")?;
            println!("VALUE: {:?}", x.get()?);
    
            let mut x = table.select::<String>("pass")?;
            println!("VALUE 2: {:?}", x.get()?);

            println!("VALUE 3: {:?}", table.select::<u16>("age")?.get()?);

            let color2 = color.it();

            for mut x in color2.into_iter() {
                println!("Column Loop: {:?} {} {} {}", x.select()?.get()?, x.index, x.has_value.0, x.has_value.1);
            }

            Ok(())
        })?;

        // println!("BYTES: {:?}", xx);

        println!("BYTES: {} {:?}", return_buffer_2.len(), return_buffer_2);

        assert_eq!(2 + 2, 4);

        let json = factory.json_encode(return_buffer_2);
        println!("JSON {:?}", json.0.to_json());

        Ok(())*/
        Ok(())
    }
    
}
