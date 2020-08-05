// #![deny(missing_docs, missing_debug_implementations, trivial_casts, trivial_numeric_casts, unused_results)]
#![allow(non_camel_case_types)]
#![no_std]

//! ## High Performance Serialization Library
//! FlatBuffers/CapNProto with Flexible Runtime Schemas
//! 
//! [Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)
//! 
//! ### TODO: 
//! - [x] Finish implementing Lists, Tuples & Maps
//! - [x] Collection Iterator
//! - [x] Compaction
//! - [ ] Documentation
//! - [ ] Tests
//! 
//! ### Features  
//! - Zero dependencies
//! - #![no_std] support, WASM ready
//! - Supports bytewise sorting of buffers
//! - Thorough Documentation
//! - Automatic & instant serilization
//! - Nearly instant deserialization
//! - Schemas are dynamic/flexible at runtime
//! - Mutate/Update/Delete values in existing buffers
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
//! *Compared to FlatBuffers / Cap'N Proto*
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Bytewise sorting is first class operation
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
//! - Bytewise sorting is first class operation
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
//! | BSON             | 𐄂                     | ~16KB      | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | MessagePack      | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
//! | FlatBuffers      | ✓                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Protocol Buffers | 𐄂                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Cap'N Proto      | ✓                     | 2^64 Bytes | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
//! | Serde            | 𐄂                     | ?          | ✓         | ✓       | 𐄂                 | 𐄂               | 𐄂                |
//! | **NoProto**      | ✓                     | ~4GB       | ✓         | ✓       | ✓                 | ✓               | ✓                |
//! 
//! 
//! #### Limitations
//! - Buffers cannot be larger than 2^32 bytes (~4GB).
//! - Tables & List collections cannot have more than 2^16 items (~16k).
//! - Enum/Option types are limited to 2^8 or 255 choices.
//! - Tuple types are limited to 2^8 or 255 items.
//! - Buffers are not validated or checked before deserializing.
//! 
//! # Quick Example
//! ```rust
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
//! // user_buffer contains a serialized Vec<u8> containing our data
//! 
//! let user_vec: Vec<u8> = user_factory.open(NP::new, |mut buffer| {
//!     
//!     // set "name" column to "some name"
//!     buffer.deep_set("name", "some name".to_owned())?;
//! 
//!     // set "age" column to 75
//!     buffer.deep_set("age", 75u16)?;
//! 
//!    // done with the buffer
//!    Ok(buffer)
//! })?;
//!  
//! // open the new buffer, `user_vec`, we just created
//! // user_vec_2 contains the serialized Vec<u8>
//! let user_vec_2: Vec<u8> = user_factory.open(NP::buffer(user_vec), |mut buffer| {
//! 
//!    // read the name column
//!    let mut user_name = buffer.deep_get::<String>("name")?;
//!    assert_eq!(user_name, Some(Box::new(String::from("some name"))));
//! 
//!    // password value will be None since we haven't set it.
//!    let mut password = buffer.deep_get::<String>("pass")?;
//!    assert_eq!(password, None);
//! 
//!    // read age value    
//!    let mut age = buffer.deep_get::<u16>("age")?;
//!    assert_eq!(age, Some(Box::new(75)));    
//! 
//!    // done with the buffer
//!    Ok(buffer)
//! })?;
//! 
//! // we can now save user_vec_2 to disk, 
//! // send it over the network, or whatever else is needed with the data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ## Guided Learning / Next Steps:
//! 1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
//! 2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
//! 3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/index.html) - How to create, update & compact buffers.
//! 4. [`Pointers`](https://docs.rs/no_proto/latest/no_proto/pointer/index.html) - How to add, remove and edit values in a buffer.
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

use alloc::boxed::Box;
use crate::pointer::NP_Value;
use crate::json_flex::json_decode;
use crate::error::NP_Error;
use crate::schema::NP_Schema;
use crate::memory::NP_Memory;
use buffer::{NP_Compact_Data, NP_Buffer};
use alloc::vec::Vec;
use alloc::vec;
use alloc::{rc::Rc, borrow::ToOwned};
use json_flex::NP_JSON;
use pointer::{any::NP_Any, NP_Ptr};
use schema::NP_TypeKeys;

const PROTOCOL_VERSION: u8 = 1;


/// Factories are created from schemas.  Once you have a factory you can use it to decode, encode, edit and compact buffers
/// 
/// The correct way to create a factory is to pass a JSON string schema into the static `new` method.  [Learn about schemas here.](./schema/index.html)
/// 
/// # Example
/// ```
/// use no_proto::error::NP_Error;
/// use no_proto::NP_Factory;
/// use no_proto::NP;
/// use no_proto::collection::table::NP_Table;
/// use no_proto::pointer::NP_Ptr;
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
/// // user_factory can now be used to make or modify buffers that contain the data in the schema.
/// 
/// // create new buffer
/// let mut user_vec: Vec<u8> = user_factory.open(NP::new, |mut buffer| {
///     
///     // set the "name" column of the table
///     buffer.deep_set("name", "Billy".to_owned())?;
/// 
///     // set the first todo
///     buffer.deep_set("todos.0", "Write a rust library.".to_owned())?;
///     
///     // read column value
///     let name_column = buffer.deep_get::<String>("name")?;
///     assert_eq!(name_column, Some(Box::new("Billy".to_owned())));
///     
///     // close buffer
///     Ok(buffer)
/// })?;
/// 
/// // user_vec is a Vec<u8> with our data
/// 
/// // open buffer and read value
/// user_vec = user_factory.open(NP::buffer(user_vec), |mut buffer| {
///
///     let name_column = buffer.deep_get::<String>("name")?;
///     assert_eq!(name_column, Some(Box::new("Billy".to_owned())));
/// 
///     // close buffer
///     Ok(buffer)
/// })?;
/// 
/// # Ok::<(), NP_Error>(()) 
/// ```
/// 
/// 
/// 
pub struct NP_Factory {
    schema: Rc<NP_Schema>,
    // _phantom: &'a PhantomData<u8>
}

/// The different options for opening a buffer
/// 
/// You can either create a new buffer with no specified capacity (1024 is the deafult),
/// create a buffer with a specific capcity with the `size` option,
/// or provide an existing buffer to be opened.
pub enum NP {
    buffer(Vec<u8>),
    size(u32),
    new
}

impl NP {
    // convert a string into a Vec<u8>, useful for NP_Map keys.
    pub fn str_to_vec<S: AsRef<str>>(string: S) -> Vec<u8> {
        string.as_ref().as_bytes().to_vec()
    }
    // convert an i64 into a Vec<u8>, useful for NP_Map keys.
    pub fn int_to_vec(int: i64) -> Vec<u8> {
        int.to_be_bytes().to_vec()
    }
    // convert a float into a Vec<u8>, useful for NP_Map keys.
    pub fn float_to_vec(float: f64) -> Vec<u8> {
        float.to_be_bytes().to_vec()
    }
}


impl NP_Factory {

    pub fn new(json_schema: &str) -> core::result::Result<NP_Factory, NP_Error> {

        let parsed = json_decode(json_schema.to_owned());

        match parsed {
            Ok(good_parsed) => {
                Ok(NP_Factory {
                    schema:  Rc::new(NP_Schema::from_json(good_parsed)?)
                })
            },
            Err(_x) => {
                Err(NP_Error::new("JSON Parse Error"))
            }
        }
    }

    pub fn open<F>(&self, buffer: NP, mut callback: F) -> Result<Vec<u8>, NP_Error>
        where F: FnMut(NP_Buffer) -> Result<NP_Buffer, NP_Error>
    {   

        let bytes = {

            let use_buffer = match buffer {
                NP::buffer(x) => x,
                NP::size(x) => {
                    NP_Factory::new_buffer(Some(x as usize))
                }
                NP::new => {
                    NP_Factory::new_buffer(None)
                }
            };
    
            let bytes = Rc::new(NP_Memory::new(use_buffer));
    
            let buffer = callback(NP_Buffer::new(Rc::clone(&self.schema), Rc::clone(&bytes)))?;

            buffer
        };

        Ok(Rc::try_unwrap(bytes.memory).unwrap().dump())
    }

    #[doc(hidden)]
    pub fn new_buffer(capacity: Option<usize>) -> Vec<u8> {

        let use_size = match capacity {
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

    pub fn maybe_compact<F>(&self, buffer: Vec<u8>, new_capacity: Option<u32>, callback: F) -> Result<Vec<u8>, NP_Error> where F: FnMut(NP_Compact_Data) -> bool {

        let result = {
            let bytes = Rc::new(NP_Memory::new(buffer));

            let buffer = NP_Buffer::new(Rc::clone(&self.schema), bytes);
    
            buffer.maybe_compact(new_capacity, callback)?
        };

        let bytes = Rc::try_unwrap(result.memory).unwrap().dump();
        Ok(bytes)
    }

    pub fn compact(&self, new_capacity: Option<u32>, buffer: Vec<u8>) -> Result<Vec<u8>, NP_Error> {

        let result = {
            let bytes = Rc::new(NP_Memory::new(buffer));

            let buffer = NP_Buffer::new(Rc::clone(&self.schema), bytes);
    
            buffer.compact(new_capacity)?
        };

        let bytes = Rc::try_unwrap(result.memory).unwrap().dump();
        Ok(bytes)
    }

    pub fn json_encode(&self, buffer: Vec<u8>) -> (NP_JSON, Vec<u8>) {
        let bytes = Rc::new(NP_Memory::new(buffer));

        let json_data = {
            let root = NP_Ptr::<NP_Any>::new_standard_ptr(1, Rc::clone(&self.schema), Rc::clone(&bytes));
            root.json_encode()
        };

        (json_data, Rc::try_unwrap(bytes).unwrap().dump())
    }

    pub fn extract_scalar<X: NP_Value + Default, F>(&self, buffer: Vec<u8>, mut callback: F) -> Result<(X, Vec<u8>), NP_Error> 
        where F: FnMut(NP_Buffer) -> Result<X, NP_Error>
    {

        match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::Table => { Err(NP_Error::new("Can't extract table type!")) },
            NP_TypeKeys::Map => { Err(NP_Error::new("Can't extract map type!")) },
            NP_TypeKeys::List => { Err(NP_Error::new("Can't extract list type!")) },
            NP_TypeKeys::Tuple => { Err(NP_Error::new("Can't extract tuple type!")) },
            _ => {
                let bytes = Rc::new(NP_Memory::new(buffer));
                let result = {
                    callback(NP_Buffer::new(Rc::clone(&self.schema), Rc::clone(&bytes)))?
                };
                Ok((result, Rc::try_unwrap(bytes).unwrap().dump()))
            }
        }

    }

    pub fn deep_clear(&self, buffer: Vec<u8>,  path: &str) -> Result<Vec<u8>, NP_Error> {
        let bytes = Rc::new(NP_Memory::new(buffer));

        let _result = {
            let buffer = NP_Buffer::new(Rc::clone(&self.schema), Rc::clone(&bytes));
            buffer.deep_clear(path)?
        };
 
        Ok(Rc::try_unwrap(bytes).unwrap().dump())
    }
    
    pub fn deep_set<X: NP_Value + Default>(&self, buffer: Vec<u8>, path: &str, value: X) -> Result<Vec<u8>, NP_Error> {
        
        let bytes = Rc::new(NP_Memory::new(buffer));

        {
            let mut buffer = NP_Buffer::new(Rc::clone(&self.schema), Rc::clone(&bytes));
            buffer._deep_set_scalar(path, value)?;
        }
 
        Ok(Rc::try_unwrap(bytes).unwrap().dump())
    }

    pub fn deep_get<X: NP_Value + Default>(&self, buffer: Vec<u8>,  path: &str) -> Result<(Option<Box<X>>, Vec<u8>), NP_Error> {
        let bytes = Rc::new(NP_Memory::new(buffer));

        let result = {
            let buffer = NP_Buffer::new(Rc::clone(&self.schema), Rc::clone(&bytes));
            buffer._deep_get_scalar(path)?
        };
 
        Ok((result, Rc::try_unwrap(bytes).unwrap().dump()))
    }
    
}

#[cfg(test)]
mod tests {

    /*
    use crate::pointer::NP_Ptr;
    use crate::collection::table::NP_Table;
    use collection::{map::NP_Map, list::NP_List};*/
    use super::*;


    #[test]
    fn it_works() -> core::result::Result<(), NP_Error> {

/*
        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "list",
            "of": {
                "type": "table",
                "columns": [
                    ["name", {"type": "string", "default": "no name"}],
                    ["age",  {"type": "u16", "default": 10}]
                ]
            }
        }"#)?;

        
        let return_buffer = factory.open(NP::new, |mut buffer| {

            /*let root: NP_Ptr<NP_List<String>> = buffer.root()?;

            let mut list = root.into()?.unwrap();

            
            list.select(10)?.set("something".to_owned())?;
            list.select(20)?.set("something2".to_owned())?;
            // list.select(20)?.set("something crazy here".to_owned())?;
            
            // list.select(20)?.clear()?;

            // root.clear()?;

            println!("WASTED BYTES: {}", buffer.calc_wasted_bytes()?);

            Ok(())*/
            buffer.deep_set("10.name", "something".to_owned())?;
            buffer.deep_set("10.name", "someth\"ing22".to_owned())?;
            buffer.deep_set("9.age", 45u16)?;
            println!("Size: {}", buffer.calc_wasted_bytes()?);
            let mut buffer = buffer.compact(None)?;

            let root = buffer.root::<NP_List<NP_Table>>()?;

            println!("JSON: {}", root.json_encode().to_string());

            // println!("JSON: {:?}", json.get::<NP_JSON>()?);

            // let list = buffer.deep_get::<String>("2.name")?.unwrap();

            // let mut table = list.select::<u16>("age")?.into()?.unwrap();

            // let column = table.select::<String>("name")?;
            // println!("VALUE: {}", list);

            // buffer.deep_set("20", "something2".to_owned())?;
            Ok(buffer)
        })?;

        let (value, buffer) = factory.deep_get::<NP_JSON>(return_buffer, "10")?;

        println!("name: {} {:?}", value.unwrap().to_string(), buffer);

        // let buffer2 = factory.deep_set::<String>(return_buffer, "15", "hello, world".to_owned())?;

        // println!("value {:?}", factory.deep_get::<String>(return_buffer, "10.name")?);

*/

        /*
        println!("BYTES: {}: {:?}", return_buffer.len(), return_buffer);

        let mut compacted = factory.compact(None, return_buffer)?;

        println!("BYTES 2: {}: {:?}", compacted.len(), compacted);

        compacted = factory.open(NP::bytes(compacted), |mut buffer| {

            let root: NP_Ptr<NP_List<String>> = buffer.root()?;

            let mut list = root.into()?.unwrap();

            println!("VALUE LIST: {:?}", list.select(20)?.get()?);

            println!("WASTED BYTES 2: {}", buffer.calc_wasted_bytes()?);

            Ok(())
        })?;


       
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

        println!("BYTES: {:?}", return_buffer_2);

        println!("GT {:?}", return_buffer_2 < return_buffer);

        // let json = factory.json_encode(return_buffer_2);
        // println!("JSON {:?}", json.0.to_json());

   
        
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
        */
        /*

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

        let return_buffer = factory.open(NP::new, |mut buffer| {

            // buffer.deep_set(".userID", "something".to_owned())?;

            // myvalue = buffer.deep_get::<String>(".userID")?;

            let root: NP_Ptr<NP_Table> = buffer.root()?;

            let mut table = root.into()?.unwrap();

            let mut x = table.select::<String>("userID")?;
            x.set("username".to_owned())?;
            // x.set("something else".to_owned())?;
    
            let mut x = table.select::<String>("pass")?;
            x.set("password123 hello".to_owned())?;

            myvalue = x.get()?;

            let mut color = table.select::<NP_List<String>>("colors")?.into()?.unwrap();

            let mut first_test_item = color.select(20)?;

            first_test_item.set("blue".to_owned())?;

            let mut second_test_item = color.select(10)?;

            second_test_item.set("orange".to_owned())?;

            let mut x = table.select::<u16>("age")?;
            x.set(1039)?;

            let mut meta = table.select::<NP_Map<String>>("meta")?.into()?.unwrap();
 
            meta.select(&NP::str_to_vec("some key"))?.set("some value".to_string())?;

            println!("VALUE 0: {:?}", meta.select(&NP::str_to_vec("some key"))?.get());

            Ok(())
        })?;

        let return_buffer_2 = factory.open(NP::bytes(return_buffer), |mut buffer| {

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

            //color.debug(|i, addr, next| {
            //    println!("I: {}, ADDR: {}, NEXT: {}", i, addr, next);
            //})?;

            println!("Value 21 GET: {:?}", color.select(21)?.get()?);
            println!("Value 22 GET: {:?}", color.select(22)?.get()?);
            println!("LENGTH: {:?}", color.len());

            // let mut meta = table.select::<NP_Map<String>>("meta")?.into()?.unwrap();

            // println!("Some Key: {:?}", meta.select(&"some key".to_string().as_bytes().to_vec())?.get()?);

            let mut x = table.select::<String>("userID")?;
            println!("VALUE: {:?}", x.get()?);
    
            let mut x = table.select::<String>("pass")?;
            println!("VALUE 2: {:?}", x.get()?);

            println!("VALUE 3: {:?}", table.select::<u16>("age")?.get()?);

            //let color2 = color.it();

            //for mut x in color2.into_iter() {
            //    println!("Column Loop: {:?} {} {} {}", x.select()?.get()?, x.index, x.has_value.0, x.has_value.1);
            //}

            println!("WASTED BYTES {:?}", buffer.calc_wasted_bytes()?);

            Ok(())
        })?;

        // println!("BYTES: {:?}", xx);

        println!("BYTES: {} {:?}", return_buffer_2.len(), return_buffer_2);

        assert_eq!(2 + 2, 4);

        let json = factory.json_encode(return_buffer_2);
        println!("JSON {:?}", json.0.to_json());
        */

        Ok(())
    }
    
}
