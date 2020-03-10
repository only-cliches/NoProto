//! # High Performance Serialization Library
//! ### Features
//! - Nearly instant deserilization & serialization
//! - Schemas are dynamic/flexible at runtime
//! - Mutate/Update/Delete values in existing buffers
//! - Supports native data types
//! - Supports collection types (list, map, table & tuple)
//! - Supports deep nesting of collection types
//! 
//! NoProto allows you to store and mutate structured data with near zero overhead.  It's like JSON but faster, type safe and allows native types.
//! 
//! NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.
//! 
//! #### Compared to FlatBuffers & Cap'N Proto:
//! - Schemas are dynamic at runtime, no compilation step
//! - Supports more types and better nested type support
//! - Mutate (add/delete/update) existing/imported buffers
//! 
//! #### Compared to JSON
//! - More space efficient
//! - Has schemas / type safe
//! - Faster serialization & deserialization
//! - Supports raw bytes & other native types
//! 
//! #### Compared to BSON
//! - Faster serialization & deserialization
//! - Has schemas / type safe
//! - Typically (but not always) more space efficient
//! - Supports much larger documents (4GB vs 16MB)
//! - Better collection support & more supported types
//! 
//! ## Limitations
//! - Buffers cannot be larger than 2^32 bytes (~4GB).
//! - Tables & List collections cannot have more than 2^16 direct child items (~16k).
//! - Enum/Option types are limited to 256 choices.
//! - Buffers are not validated or checked before deserializing.
//! 
//! # Quick Example
//! ```
//! use no_proto::error::NoProtoError;
//! use no_proto::NoProtoFactory;
//! 
//! // JSON is used to describe schema for the factory
//! // Each factory represents a single schema
//! // One factory can be used to serialize/deserialize any number of buffers
//! let user_factory = NoProtoFactory::new(r#"{
//!     "type": "table",
//!     "columns": [
//!         ["userID", {"type": "string"}],
//!         ["pass", {"type": "string"}],
//!         ["age", {"type": "uint16"}]
//!     ]
//! }"#)?;
//! 
//! // user_buffer contains a deserialized Vec<u8> containing our data
//! let user_buffer: Vec<u8> = user_factory.new_buffer(None, |mut buffer| {
//!    
//!     // we can mutate and read the buffer here
//!     buffer.open(|mut root| {
//!         
//!         // the root of our schema is a table type, so we have to convert the root pointer to a table.
//!         let mut table = root.as_table()?;
//!         // select a column. Selected columns can be mutated or read from.
//!         let mut user_id = table.select("userID")?;
//!         // set value of userID column
//!         user_id.set_string("some ID")?;
//!         // select age column and set it's value
//!         let mut age = table.select("age")?;
//!         age.set_uint16(75)?;
//!
//!         // done mutating the buffer
//!         Ok(())
//!    })?;
//!    
//!    // close a buffer when we're done with it
//!    buffer.close()
//! })?;
//!  
//! // read in the new buffer we just created
//! // user_buffer_2 contains the deserialized Vec<u8> of the buffer
//! let user_buffer_2: Vec<u8> = user_factory.load_buffer(user_buffer, |mut buffer| {
//! 
//!     // we can mutate and read the buffer here
//!     buffer.open(|mut root| {
//!         
//!         // get the table root again
//!         let mut table = root.as_table()?;
//!         // read the userID column
//!         let user_id = table.select("userID")?;
//!         assert_eq!(user_id.to_string()?, Some("some ID".to_owned()));
//!         // password value will be None since we haven't set it.
//!         let password = table.select("pass")?;
//!         assert_eq!(password.to_string()?, None);
//!         // read age value    
//!         let age = table.select("age")?;
//!         assert_eq!(age.to_uint16()?, Some(75));    
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
//! # Ok::<(), NoProtoError>(()) 
//! ```

pub mod pointer;
pub mod collection;
pub mod buffer;
pub mod schema;
pub mod error;
mod memory;

use crate::error::NoProtoError;
use crate::schema::NoProtoSchema;
use buffer::NoProtoBuffer;

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
pub struct NoProtoFactory {
    schema: NoProtoSchema
}

impl NoProtoFactory {
    pub fn new(json_schema: &str) -> std::result::Result<NoProtoFactory, NoProtoError> {

        match json::parse(json_schema) {
            Ok(x) => {
                let mut new_schema = NoProtoSchema::init();

                let valid_schema = new_schema.from_json(x)?;
        
                Ok(NoProtoFactory {
                    schema: valid_schema
                })
            },
            Err(e) => {
                Err(NoProtoError::new(format!("Error Parsing JSON Schema: {}", e.to_string()).as_str()))
            }
        }
    }

    pub fn new_buffer<F>(&self, capacity: Option<u32>, mut callback: F) -> std::result::Result<Vec<u8>, NoProtoError>
        where F: FnMut(NoProtoBuffer) -> std::result::Result<Vec<u8>, NoProtoError>
    {   
        callback(NoProtoBuffer::new(&self.schema, capacity))
    }

    pub fn load_buffer<F>(&self, buffer: Vec<u8>, mut callback: F) -> std::result::Result<Vec<u8>, NoProtoError>
        where F: FnMut(NoProtoBuffer) -> std::result::Result<Vec<u8>, NoProtoError>
    {   
        callback(NoProtoBuffer::load(&self.schema, buffer))
    }

    pub fn empty<T>() -> Option<T> {
        None
    }

    /*
    pub fn parse_buffer() -> NoProtoBuffer {

    }
    */
}


#[cfg(test)]
mod tests {
    use crate::{pointer::NoProtoGeo, NoProtoBuffer, pointer::NoProtoUUID, pointer::NoProtoPointer, NoProtoFactory, error::NoProtoError};
    use json::*;
    use std::{rc::Rc, cell::RefCell};
    use std::result::*;

    #[test]
    fn it_works() -> std::result::Result<(), NoProtoError> {

        let factory = NoProtoFactory::new(r#"{
            "type": "table",
            "columns": [
                ["userID", {"type": "string"}],
                ["pass", {"type": "string"}],
                ["age", {"type": "uint16"}]
            ]
        }"#)?;

        let mut myvalue = NoProtoFactory::empty::<String>();

        let return_buffer = factory.new_buffer(None, |mut buffer| {

            buffer.open(|mut root| {
            
                let mut table = root.as_table()?;
        
                let mut x = table.select("userID")?;
                x.set_string("some ID")?;
        
                let mut x = table.select("age")?;
                x.set_generic_integer(x.integer_truncate(2033293823998))?;
        
                let mut x = table.select("pass")?;
                x.set_string("password123")?;
        
                // let mut x = table.select("pass")?;
                // x.set_string("password203930223.")?;

                myvalue = x.to_string()?;
        
                let x = table.select("userID")?;
                println!("VALUE: {:?}", x.to_string()?);
        
                let x = table.select("pass")?;
                println!("VALUE 2: {:?}", x.to_string()?);

                println!("VALUE 3: {:?}", table.select("age")?.to_generic_integer());

                Ok(())
            })?;

            buffer.close()
        })?;

        // println!("BYTES: {:?}", xx);

        println!("BYTES: {:?}", return_buffer);

        assert_eq!(2 + 2, 4);

        Ok(())
    }
}
