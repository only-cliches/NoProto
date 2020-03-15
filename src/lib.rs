// #![deny(warnings, missing_docs, missing_debug_implementations, trivial_casts, trivial_numeric_casts, unused_results)]

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
//! use no_proto::error::NoProtoError;
//! use no_proto::NoProtoFactory;
//! 
//! // JSON is used to describe schema for the factory
//! // Each factory represents a single schema
//! // One factory can be used to serialize/deserialize any number of buffers
//! let user_factory = NoProtoFactory::new(r#"{
//!     "type": "table",
//!     "columns": [
//!         ["name",   {"type": "string"}],
//!         ["pass",   {"type": "string"}],
//!         ["age",    {"type": "uint16"}]
//!     ]
//! }"#)?;
//! 
//! // user_buffer contains a deserialized Vec<u8> containing our data
//! let user_buffer: Vec<u8> = user_factory.new_buffer(None, |mut buffer| {
//!    
//!     // open the buffer to read or update values
//!     buffer.open(|mut root| {
//!         
//!         // the root of our schema is a table type, 
//!         // so we have to convert the root pointer to a table.
//!         let mut table = root.as_table()?;
//!         // Select a column. Selected columns can be mutated or read from.
//!         let mut user_name = table.select("name")?;
//!         // set value of name column
//!         user_name.set_string("some name")?;
//!         // select age column and set it's value
//!         let mut age = table.select("age")?;
//!         age.set_uint16(75)?;
//!
//!         // done mutating/reading the buffer
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
//!         // read the name column
//!         let user_name = table.select("name")?;
//!         assert_eq!(user_name.to_string()?, Some("some name".to_owned()));
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
use pointer::NoProtoValue;

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
pub struct NoProtoFactory<T> {
    schema: NoProtoSchema<T>
}

impl<T: NoProtoValue + Default> NoProtoFactory<T> {
    pub fn new(json_schema: &str) -> std::result::Result<NoProtoFactory<T>, NoProtoError> {

        match json::parse(json_schema) {
            Ok(x) => {
                Ok(NoProtoFactory {
                    schema: NoProtoSchema::from_json(x)?
                })
            },
            Err(e) => {
                Err(NoProtoError::new(format!("Error Parsing JSON Schema: {}", e.to_string()).as_str()))
            }
        }
    }

    pub fn new_buffer<F>(&self, capacity: Option<u32>, mut callback: F) -> std::result::Result<Vec<u8>, NoProtoError>
        where F: FnMut(NoProtoBuffer<T>) -> std::result::Result<Vec<u8>, NoProtoError>
    {   
        callback(NoProtoBuffer::new(&self.schema, capacity))
    }

    pub fn load_buffer<F>(&self, buffer: Vec<u8>, mut callback: F) -> std::result::Result<Vec<u8>, NoProtoError>
        where F: FnMut(NoProtoBuffer<T>) -> std::result::Result<Vec<u8>, NoProtoError>
    {   
        callback(NoProtoBuffer::load(&self.schema, buffer))
    }

    pub fn empty<X>() -> Option<X> {
        None
    }

    /*
    pub fn parse_buffer() -> NoProtoBuffer {

    }
    */
}


#[cfg(test)]
mod tests {
    use crate::pointer::any::NoProtoAny;
    use crate::collection::table::NoProtoTable;
    use super::*;

    #[test]
    fn it_works() -> std::result::Result<(), NoProtoError> {

        let factory: NoProtoFactory<NoProtoAny> = NoProtoFactory::new(r#"{
            "type": "table",
            "columns": [
                ["userID", {"type": "string"}],
                ["pass",   {"type": "string"}],
                ["age",    {"type": "uint16"}]
            ]
        }"#)?;

        let mut myvalue = None;

        let return_buffer = factory.new_buffer(None, |mut buffer| {


            buffer.deep_set("userID", "something");

            let userID = buffer.deep_get::<&str>("userID")?;

            buffer.open(|mut root| {
            
                let table = NoProtoAny::cast::<NoProtoTable>(root)?.into()?.unwrap();

                let mut x = table.select::<&str>("userID")?;
                x.set("some id")?;
        
                let mut x = table.select::<i16>("age")?;
                x.set(2032039398)?;
        
                let mut x = table.select::<String>("pass")?;
                x.set("password123".to_owned())?;

                myvalue = x.get()?;
        
                let x = table.select::<String>("userID")?;
                println!("VALUE: {:?}", x.get()?);
        
                let x = table.select::<String>("pass")?;
                println!("VALUE 2: {:?}", x.get()?);

                println!("VALUE 3: {:?}", table.select::<i16>("age")?.get()?);

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
