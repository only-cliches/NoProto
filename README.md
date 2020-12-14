## Simple & Performant Zero-Copy Serialization
Performance of Protocol Buffers with flexibility of JSON

[Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)

### Features  
- Zero dependencies
- Zero copy deserialization
- `no_std` support, WASM ready
- Native byte-wise sorting
- Extensive Documentation & Testing
- Easily mutate, add or delete values in existing buffers
- Schemas allow default values and non destructive updates
- Supports most common native data types
- Supports collection types (list, map, table & tuple)
- Supports deep nesting of collection types
- Easy and performant export to JSON.
- [Thoroughly documented](https://docs.rs/no_proto/latest/no_proto/format/index.html) & simple data storage format

NoProto allows you to store, read & mutate structured data with near zero overhead. It's like Protocol Buffers except buffers and schemas are dynamic at runtime instead of requiring compilation.  It's like JSON but faster, type safe and allows native types.

Byte-wise sorting comes in the box and is a first class operation. Two NoProto buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 

NoProto moves the cost of deserialization to the access methods instead of deserializing the entire object ahead of time (Incremental Deserialization). This makes it a perfect use case for things like database storage or file storage of structured data.

*Compared to Protocol Buffers*
- Comparable serialization & deserialization performance
- Updating buffers is orders of magnitude faster
- Easier & Simpler API
- Schemas are dynamic at runtime, no compilation step
- Supports more types and better nested type support
- Byte-wise sorting is first class operation
- Mutate (add/delete/update) existing/imported buffers

*Compared to JSON / BSON*
- Far more space efficient
- Significantly faster serialization & deserialization
- Deserializtion is zero copy
- Has schemas / type safe
- Supports byte-wise sorting
- Supports raw bytes & other native types


*Compared to Serde*
- Supports byte-wise sorting
- Objects & schemas are dynamic at runtime
- Deserializtion is zero copy
- Language agnostic

| Format           | Zero-Copy | Size Limit | Mutable | Schemas | Language Agnostic | No Compiling    | Byte-wise Sorting |
|------------------|-----------|------------|---------|---------|-------------------|-----------------|-------------------|
| **NoProto**      | ✓         | ~16KB      | ✓       | ✓       | ✓                 | ✓               | ✓                 |
| JSON             | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| BSON             | 𐄂         | ~16KB      | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| MessagePack      | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| FlatBuffers      | ✓         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
| Protocol Buffers | 𐄂         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
| Cap'N Proto      | ✓         | 2^64 Bytes | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
| Serde            | 𐄂         | ?          | 𐄂       | ✓       | 𐄂                 | 𐄂               | 𐄂                 |
| Veriform         | 𐄂         | ?          | 𐄂       | 𐄂       | 𐄂                 | 𐄂               | 𐄂                 |


# Quick Example
```rust
use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::collection::table::NP_Table;

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
let mut user_buffer = user_factory.empty_buffer(None); // optional capacity, optional address size (u16 by default)

// set an internal value of the buffer, set the  "name" column
user_buffer.set(&["name"], "Billy Joel")?;

// assign nested internal values, sets the first tag element
user_buffer.set(&["tags", "0"], "first tag")?;

// get an internal value of the buffer from the "name" column
let name = user_buffer.get::<&str>(&["name"])?;
assert_eq!(name, Some("Billy Joel"));

// close buffer and get internal bytes
let user_bytes: Vec<u8> = user_buffer.close();

// open the buffer again
let user_buffer = user_factory.open_buffer(user_bytes)?;

// get nested internal value, first tag from the tag list
let tag = user_buffer.get::<&str>(&["tags", "0"])?;
assert_eq!(tag, Some("first tag"));

// get nested internal value, the age field
let age = user_buffer.get::<u16>(&["age"])?;
// returns default value from schema
assert_eq!(age, Some(0u16));

// close again
let user_bytes: Vec<u8> = user_buffer.close();


// we can now save user_bytes to disk, 
// send it over the network, or whatever else is needed with the data

// The schema can also be compiled into a byte array for more efficient schema parsing.
let byte_schema: Vec<u8> = user_factory.compile_schema();

// The byte schema can be used just like JSON schema, but it's WAY faster to parse.
let user_factory2 = NP_Factory::new_compiled(byte_schema);

// confirm the new byte schema works with existing buffers
let user_buffer = user_factory2.open_buffer(user_bytes)?;
let tag = user_buffer.get::<&str>(&["tags", "0"])?;
assert_eq!(tag, Some("first tag"));

```

## Guided Learning / Next Steps:
1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/struct.NP_Buffer.html) - How to create, update & compact buffers/data.
4. [`Data Format`](https://docs.rs/no_proto/latest/no_proto/format/index.html) - Learn how data is saved into the buffer.

## Benchmarks
While it's difficult to properly benchmark libraries like these in a fair way, I've made an attempt in the graph below.  These benchmarks are available in the `bench` folder and you can easily run them yourself with `cargo run`. 

The format and data used in the benchmarks were taken from the `flatbuffers` benchmarks github repo.  You should always benchmark/test your own use case for each library before making any decisions on what to use.

**Legend**: Higher % is better, 200% means the competing library did the same task as NoProto in half the time, 50% means it took twice as long.

| Library            | Encode | Decode All | Decode 1 | Update 1 | Size | Size (Zlib) |
|--------------------|--------|------------|----------|----------|------|-------------|
| NoProto            | 100%   | 100%       | 100%     | 100%     | 283  | 226         |
| FlatBuffers        | 180%   | 800%       | 1600%    | 8%       | 336  | 214         |
| Protocol Buffers 2 | 99%    | 80%        | 7%       | 2%       | 220  | 163         |
| MessagePack        | 12%    | 15%        | 1%       | 1%       | 431  | 245         |
| JSON               | 65%    | 30%        | 2%       | 2%       | 673  | 246         |
| BSON               | 1%     | 8%         | 1%       | 1%       | 600  | 279         |


- **Encode**: Transfer a collection of data into a serialized form 1,000,000 times.
- **Decode All**: Decode/Deserialize an object into all it's properties 1,000,000 times.
- **Decode 1**: Decode/Deserialize one property of an object 1,000,000 times.
- **Update 1**: Deserialize, update a single property, then serialize an object 1,000,000 times.

Complete benchmark source code is available [here](https://github.com/only-cliches/NoProto/tree/master/bench).

#### Limitations
- Buffers cannot be larger than 2^16 bytes (~16kb).
- Collections (Lists, Maps, Tuples & Tables) cannot have more than 255 immediate child items.
- Enum/Option types are limited to 255 choices and choice strings cannot be larger than 255 bytes.
- Tables are limited to 255 columns and column names cannot be larger than 255 bytes.
- Buffers are not validated or checked before deserializing.

#### Non Goals / Known Tradeoffs 
If every CPU cycle counts, you don't mind compiling fixed schemas and you don't plan to mutate your buffers/objects, FlatBuffers/CapnProto is probably the way to go.  It's impossible to make a flexible format like NoProto as fast as formats that compile your schemas ahead of time.

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