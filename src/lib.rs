// #![deny(warnings, missing_docs, missing_debug_implementations, trivial_casts, trivial_numeric_casts, unused_results)]
#![allow(non_camel_case_types)]

//! # High Performance Serialization Library
//! ### Features
//! - Nearly instant deserilization & serialization
//! - Schemas are dynamic/flexible at runtime
//! - Mutate/Update/Delete values in existing buffers
//! - Supports native data types
//! - Supports collection types (list, map, table & tuple)
//! - Supports deep nesting of collection types
//! 
//! NoProto allows you to store and mutate structured data with near zero overhead.  It's like JSON but faster, type safe and allows native types.  It's like Cap'N Proto/Flatbuffers except buffers and schemas are dynamic at runtime instead of requiring compilation.  
//! 
//! NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.
//! 
//! *Compared to FlatBuffers /Cap'N Proto*
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Mutate (add/delete/update) existing/imported buffers
//! 
//! *Compared to JSON*
//! - Typically more space efficient
//! - Has schemas / type safe
//! - Faster serialization & deserialization
//! - Supports raw bytes & other native types
//! 
//! *Compared to BSON*
//! - Faster serialization & deserialization
//! - Has schemas / type safe
//! - Typically more space efficient
//! - Supports much larger documents (4GB vs 16MB)
//! - Better collection support & more supported types
//! 
//! *Compared to Serde*
//! - Objects & schemas are dynamic at runtime
//! - Faster serialization & deserialization
//! 
//! #### Limitations
//! - Buffers cannot be larger than 2^32 bytes (~4GB).
//! - Tables & List collections cannot have more than 2^16 direct descendant child items (~16k).
//! - Enum/Option types are limited to 256 choices.
//! - Buffers are not validated or checked before deserializing.
//! 
//! # Quick Example
//! ```
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
//!         ["pass",   {"type": "string"}],
//!         ["age",    {"type": "uint16"}]
//!     ]
//! }"#)?;
//! 
//! // creating a new buffer from the `user_factory` schema
//! // user_buffer contains a deserialized Vec<u8> containing our data
//! let user_buffer: Vec<u8> = user_factory.new_buffer(None, |mut buffer| {
//!    
//!     // open the buffer to read or update values
//!     buffer.open(|root: NP_Ptr<NP_Table>| { // <- type cast the root
//!         
//!         // the root of our schema is a collection type (NP_Table), 
//!         // so we have to collapse the root pointer into the collection type.
//!         let mut table: NP_Table = root.into()?.unwrap();
//! 
//!         // Select a column and type cast it. Selected columns can be mutated or read from.
//!         let mut user_name = table.select::<String>("name")?;
//! 
//!         // set value of name column
//!         user_name.set("some name".to_owned())?;
//! 
//!         // select age column and set it's value
//!         let mut age = table.select::<u16>("age")?;
//!         age.set(75)?;
//!
//!         // done mutating/reading the buffer
//!         Ok(())
//!    })?;
//!    
//!    // close a buffer when we're done with it
//!    buffer.close()
//! })?;
//!  
//! // open the new buffer, `user_buffer`, we just created
//! // user_buffer_2 contains the deserialized Vec<u8>
//! let user_buffer_2: Vec<u8> = user_factory.load_buffer(user_buffer, |mut buffer| {
//! 
//!     // we can mutate and read the buffer here
//!     buffer.open(|root: NP_Ptr<NP_Table>| {
//!         
//!         // get the table root again
//!         let mut table = root.into()?.unwrap();
//! 
//!         // read the name column
//!         let mut user_name = table.select::<String>("name")?;
//!         assert_eq!(user_name.get()?, Some(String::from("some name")));
//! 
//!         // password value will be None since we haven't set it.
//!         let mut password = table.select::<String>("pass")?;
//!         assert_eq!(password.get()?, None);
//! 
//!         // read age value    
//!         let mut age = table.select::<u16>("age")?;
//!         assert_eq!(age.get()?, Some(75));    
//! 
//!         // done with the buffer
//!         Ok(())
//!    })?;
//!    
//!    // close a buffer when we're done with it
//!    buffer.close()
//! })?;
//! 
//! // we can now save user_buffer_2 to disk, 
//! // send it over the network, or whatever else is needed with the data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```

pub mod pointer;
pub mod collection;
pub mod buffer;
pub mod schema;
pub mod error;
mod memory;

use crate::pointer::NP_Ptr;
use crate::error::NP_Error;
use crate::schema::NP_Schema;
use buffer::NP_Buffer;
use pointer::NP_Value;

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
pub struct NP_Factory {
    schema: NP_Schema
}

impl NP_Factory {
    pub fn new(json_schema: &str) -> std::result::Result<NP_Factory, NP_Error> {

        match json::parse(json_schema) {
            Ok(x) => {
                Ok(NP_Factory {
                    schema:  NP_Schema::from_json(x)?
                })
            },
            Err(e) => {
                Err(NP_Error::new(format!("Error Parsing JSON Schema: {}", e.to_string()).as_str()))
            }
        }
    }

    pub fn new_buffer<F>(&self, capacity: Option<u32>, mut callback: F) -> std::result::Result<Vec<u8>, NP_Error>
        where F: FnMut(NP_Buffer) -> std::result::Result<Vec<u8>, NP_Error>
    {   
        callback(NP_Buffer::new(&self.schema, capacity))
    }

    pub fn load_buffer<F>(&self, buffer: Vec<u8>, mut callback: F) -> std::result::Result<Vec<u8>, NP_Error>
        where F: FnMut(NP_Buffer) -> std::result::Result<Vec<u8>, NP_Error>
    {   
        callback(NP_Buffer::load(&self.schema, buffer))
    }
}


#[cfg(test)]
mod tests {

    use crate::pointer::NP_Ptr;
    // use crate::pointer::any::NP_Any;
    use crate::collection::table::NP_Table;
    use super::*;
    use pointer::any::NP_Any;

    #[test]
    fn it_works() -> std::result::Result<(), NP_Error> {

        let factory: NP_Factory = NP_Factory::new(r#"{
            "type": "table",
            "columns": [
                ["userID", {"type": "string"}],
                ["pass",   {"type": "string"}],
                ["age",    {"type": "uint16"}]
            ]
        }"#)?;

        let mut myvalue: Option<String> = None;

        let mut return_buffer = factory.new_buffer(None, |mut buffer| {

            // buffer.deep_set(".userID", "something".to_owned())?;

            // myvalue = buffer.deep_get::<String>(".userID")?;

            buffer.open(|root: NP_Ptr<NP_Table>| {
            
                let mut table = root.into()?.unwrap();

                let mut x = table.select::<String>("userID")?;
                x.set("username".to_owned())?;
        
                let mut x = table.select::<String>("pass")?;
                x.set("password123".to_owned())?;

                myvalue = x.get()?;

                let mut x = table.select::<u16>("age")?;
                x.set(1039)?;

                Ok(())
            })?;

            buffer.close()
        })?;

        let return_buffer_2 = factory.load_buffer(return_buffer, |mut buffer| {

            buffer.open(|root: NP_Ptr<NP_Table>| {
            
                let mut table = root.into()?.unwrap();

                let mut x = table.select::<String>("userID")?;
                println!("VALUE: {:?}", x.get()?);
        
                let mut x = table.select::<String>("pass")?;
                println!("VALUE 2: {:?}", x.get()?);

                println!("VALUE 3: {:?}", table.select::<u16>("age")?.get()?);

                Ok(())
            })?;

            buffer.close()
        })?;

        // println!("BYTES: {:?}", xx);

        println!("BYTES: {:?}", return_buffer_2);

        assert_eq!(2 + 2, 4);

        Ok(())
    }
}
