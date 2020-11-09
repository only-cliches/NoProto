## High Performance Serialization Library
Faster than JSON with Schemas and Native Types.  Like Mutable Protocol Buffers with Compile Free Schemas.

[Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)

### Features  
- Zero dependencies
- #![no_std] support, WASM ready
- Supports byte-wise sorting of buffers
- Thorough Documentation
- Automatic & instant sterilization
- Nearly instant deserialization
- Schemas are dynamic/flexible at runtime
- Mutate/Insert/Delete values in existing buffers
- Supports native data types
- Supports collection types (list, map, table & tuple)
- Supports deep nesting of collection types
- [Thoroughly documented](https://docs.rs/no_proto/latest/no_proto/format/index.html) & simple data storage format

NoProto allows you to store, read & mutate structured data with near zero overhead. It's like Cap'N Proto/Flatbuffers except buffers and schemas are dynamic at runtime instead of requiring compilation.  It's like JSON but faster, type safe and allows native types.

Byte-wise sorting comes in the box and is a first class operation. The result is two NoProto buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 

NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time. This makes it a perfect use case for things like database storage or file storage of structured data.

*Compared to FlatBuffers / Cap'N Proto / Protocol Buffers*
- Comparable serialization & deserialization performance
- Easier & Simpler API
- Schemas are dynamic at runtime, no compilation step
- Supports more types and better nested type support
- Byte-wise sorting is first class operation
- Mutate (add/delete/update) existing/imported buffers

*Compared to JSON*
- Far more space efficient
- Faster serialization & deserialization
- Has schemas / type safe
- Supports byte-wise sorting
- Supports raw bytes & other native types

*Compared to BSON*
- Far more space efficient
- Faster serialization & deserialization
- Has schemas / type safe
- Byte-wise sorting is first class operation
- Supports much larger documents (4GB vs 16KB)
- Better collection support & more supported types

*Compared to Serde*
- Supports byte-wise sorting
- Objects & schemas are dynamic at runtime
- Faster serialization & deserialization
- Language agnostic

| Format           | Incrimental De/Serialization | Size Limit | Mutable | Schemas | Language Agnostic | No Compiling    | Byte-wise Sorting |
|------------------|-----------------------|------------|-----------|---------|-------------------|-----------------|------------------|
| **NoProto**      | ✓                     | ~4GB       | ✓         | ✓       | ✓                 | ✓               | ✓                |
| JSON             | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
| BSON             | 𐄂                     | ~16KB      | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
| MessagePack      | 𐄂                     | Unlimited  | ✓         | 𐄂       | ✓                 | ✓               | 𐄂                |
| FlatBuffers      | ✓                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
| Protocol Buffers | 𐄂                     | ~2GB       | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
| Cap'N Proto      | ✓                     | 2^64 Bytes | 𐄂         | ✓       | ✓                 | 𐄂               | 𐄂                |
| Serde            | 𐄂                     | ?          | 𐄂         | ✓       | 𐄂                 | 𐄂               | 𐄂                |
| Veriform         | 𐄂                     | ?          | 𐄂         | 𐄂       | 𐄂                 | 𐄂               | 𐄂                |


#### Limitations
- Buffers cannot be larger than 2^32 bytes (~4GB).
- Lists cannot have more than 65,535 items.
- Enum/Option types are limited to 255 choices.
- Tables are limited to 255 columns.
- Tuple types are limited to 255 items.
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
        ["age",    {"type": "u16", "default": 0}],
        ["tags",   {"type": "list", "of": {
            "type": "string"
        }}]
    ]
}"#)?;


// create a new empty buffer
let mut user_buffer = user_factory.empty_buffer(None, None); // optional capacity, optional address size (u16 by default)

// set an internal value of the buffer, set the  "name" column
user_buffer.deep_set("name", String::from("Billy Joel"))?;

// assign nested internal values, sets the first tag element
user_buffer.deep_set("tags.0", String::from("first tag"))?;

// get an internal value of the buffer from the "name" column
let name = user_buffer.deep_get::<String>("name")?;
assert_eq!(name, Some(Box::new(String::from("Billy Joel"))));

// close buffer and get internal bytes
let user_bytes: Vec<u8> = user_buffer.close();

// open the buffer again
let user_buffer = user_factory.open_buffer(user_bytes);

// get nested internal value, first tag from the tag list
let tag = user_buffer.deep_get::<String>("tags.0")?;
assert_eq!(tag, Some(Box::new(String::from("first tag"))));

// get nested internal value, the age field
let age = user_buffer.deep_get::<u16>("age")?;
// returns default value from schema
assert_eq!(age, Some(Box::new(0u16)));

// close again
let user_bytes: Vec<u8> = user_buffer.close();

// we can now save user_bytes to disk, 
// send it over the network, or whatever else is needed with the data

// The schema can also be compiled into a byte array for more efficient schema parsing.
let byte_schema: Vec<u8> = user_factory.compile_schema();

// The byte schema can be used just like JSON schema, but it's WAY faster to parse.
let user_factory2 = NP_Factory::new_compiled(byte_schema);

// confirm the new byte schema works with existing buffers
let user_buffer = user_factory2.open_buffer(user_bytes);
let tag = user_buffer.deep_get::<String>("tags.0")?;
assert_eq!(tag, Some(Box::new(String::from("first tag"))));


# Ok::<(), NP_Error>(()) 
```

## Non Goals / Known Tradeoffs
There are formats that focus on being as compact as possible.  While NoProto is not intentionally wasteful with space, it will likely never be the most compact way to store data.  If you need the smallest possible format MessagePack is a good choice.

If every CPU cycle counts, FlatBuffers/CapnProto is probably the way to go.  While NoProto makes good tradeoffs with flexibility and performance, it cannot be as fast as languages that compile the schema into source code.  In the future compiling schema to source code could be a feature, but for now I'm happy leaving that edge to the other libraries.

## Guided Learning / Next Steps:
1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/struct.NP_Buffer.html) - How to create, update & compact buffers.
4. [`Pointers`](https://docs.rs/no_proto/latest/no_proto/pointer/struct.NP_Ptr.html) - How to add, remove and edit values in a buffer.
5. [`Data Format`](https://docs.rs/no_proto/latest/no_proto/format/index.html) - Learn how data is saved into the buffer.


----------------------

MIT License

Copyright (c) 2020 Scott Lott

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