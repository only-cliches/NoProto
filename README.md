# NoProto
## High Performance Serialization Library

[Github](https://github.com/ClickSimply/NoProto)
[Crates.io](https://crates.io/crates/no_proto)

### TODO: 
- [ ] Finish implementing Lists, Tuples & Maps
- [ ] Compaction
- [ ] Documentation
- [ ] Test

### Features
- Nearly instant deserilization & serialization
- Schemas are dynamic/flexible at runtime
- Mutate/Update/Delete values in existing buffers
- Supports native data types
- Supports collection types (list, map, table & tuple)
- Supports deep nesting of collection types

NoProto allows you to store and mutate structured data with near zero overhead.  It's like JSON but faster, type safe and allows native types.  It's like Cap'N Proto/Flatbuffers except buffers and schemas are dynamic at runtime instead of requiring compilation.  

NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.

*Compared to FlatBuffers /Cap'N Proto*
- Schemas are dynamic at runtime, no compilation step
- Supports more types and better nested type support
- Mutate (add/delete/update) existing/imported buffers

*Compared to JSON*
- Has schemas / type safe
- Faster serialization & deserialization
- Supports raw bytes & other native types

*Compared to BSON*
- Faster serialization & deserialization
- Has schemas / type safe
- Supports much larger documents (4GB vs 16MB)
- Better collection support & more supported types

*Compared to Serde*
- Objects & schemas are dynamic at runtime
- Faster serialization & deserialization

#### Limitations
- Buffers cannot be larger than 2^32 bytes (~4GB).
- Tables & List collections cannot have more than 2^16 direct descendant child items (~16k).
- Enum/Option types are limited to 256 choices.
- Buffers are not validated or checked before deserializing.

# Quick Example
```rust
use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::collection::table::NP_Table;
use no_proto::pointer::NP_Ptr;

// JSON is used to describe schema for the factory
// Each factory represents a single schema
// One factory can be used to serialize/deserialize any number of buffers
let user_factory = NP_Factory::new(r#"{
    "type": "table",
    "columns": [
        ["name",   {"type": "string"}],
        ["pass",   {"type": "string"}],
        ["age",    {"type": "uint16"}]
    ]
}"#)?;

// creating a new buffer from the `user_factory` schema
// user_buffer contains a deserialized Vec<u8> containing our data
let user_buffer: Vec<u8> = user_factory.new_buffer(None, |mut buffer| {
   
    // open the buffer to read or update values
    buffer.open(|root: NP_Ptr<NP_Table>| { // <- type cast the root
        
        // the root of our schema is a collection type (NP_Table), 
        // so we have to collapse the root pointer into the collection type.
        let mut table: NP_Table = root.into()?.unwrap();

        // Select a column and type cast it. Selected columns can be mutated or read from.
        let mut user_name = table.select::<String>("name")?;

        // set value of name column
        user_name.set("some name".to_owned())?;

        // select age column and set it's value
        let mut age = table.select::<u16>("age")?;
        age.set(75)?;
//!
        // done mutating/reading the buffer
        Ok(())
   })?;
   
   // close a buffer when we're done with it
   buffer.close()
})?;
 
// open the new buffer, `user_buffer`, we just created
// user_buffer_2 contains the deserialized Vec<u8>
let user_buffer_2: Vec<u8> = user_factory.load_buffer(user_buffer, |mut buffer| {

    // we can mutate and read the buffer here
    buffer.open(|root: NP_Ptr<NP_Table>| {
        
        // get the table root again
        let mut table = root.into()?.unwrap();

        // read the name column
        let mut user_name = table.select::<String>("name")?;
        assert_eq!(user_name.get()?, Some(String::from("some name")));

        // password value will be None since we haven't set it.
        let mut password = table.select::<String>("pass")?;
        assert_eq!(password.get()?, None);

        // read age value    
        let mut age = table.select::<u16>("age")?;
        assert_eq!(age.get()?, Some(75));    

        // done with the buffer
        Ok(())
   })?;
   
   // close a buffer when we're done with it
   buffer.close()
})?;

// we can now save user_buffer_2 to disk, 
// send it over the network, or whatever else is needed with the data

```

MIT License

Copyright (c) 2019 Scott Lott

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.