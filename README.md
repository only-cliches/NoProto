## NoProto: Flexible, Fast & Compact Serialization with RPC

<img src="https://github.com/only-cliches/NoProto/raw/master/logo_small.png"/>

[Github](https://github.com/only-cliches/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)

[![MIT license](https://img.shields.io/badge/License-MIT-blue.svg)](https://lbesson.mit-license.org/)
[![crates.io](https://img.shields.io/crates/v/no_proto.svg)](https://crates.io/crates/no_proto)
[![docs.rs](https://docs.rs/no_proto/badge.svg)](https://docs.rs/no_proto/latest/no_proto/)
[![GitHub stars](https://img.shields.io/github/stars/only-cliches/NoProto.svg?style=social&label=Star&maxAge=2592000)](https://GitHub.com/only-cliches/NoProto/stargazers/)
### Features  

**Lightweight**<br/>
- Zero dependencies
- `no_std` support, WASM ready
- Most compact non compiling storage format

**Stable**<br/>
- Safely accept untrusted buffers
- Passes Miri compiler safety checks
- Panic and unwrap free

**Easy**<br/>
- Extensive Documentation & Testing
- Full interop with JSON, Import and Export JSON values
- [Thoroughly documented](https://docs.rs/no_proto/latest/no_proto/format/index.html) & simple data storage format

**Fast**<br/>
- Zero copy deserialization
- Most updates are append only
- Deserialization is incrimental

**Powerful**<br/>
- Native byte-wise sorting
- Supports recursive data types
- Supports most common native data types
- Supports collections (list, map, struct & tuple)
- Supports arbitrary nesting of collection types
- Schemas support default values and non destructive updates
- Transport agnostic [RPC Framework](https://docs.rs/no_proto/latest/no_proto/rpc/index.html).


### Why ANOTHER Serialization Format?
1. NoProto combines the **performance** of compiled formats with the **flexibilty** of dynamic formats:

**Compiled** formats like Flatbuffers, CapN Proto and bincode have amazing performance and extremely compact storage buffers, but you MUST compile the data types into your application.  This means if the schema of the data changes the application must be recompiled to accomodate the new schema.

**Dynamic** formats like JSON, MessagePack and BSON give flexibilty to store any data with any schema at runtime but the storage buffers are fat and performance is somewhere between horrible and hopefully acceptable.

NoProto takes the performance advantages of compiled formats and implements them in a flexible format.

2. NoProto is a **key-value database focused format**:

**Byte Wise Sorting** Ever try to store a signed integer as a sortable key in a database?  NoProto can do that.  Almost every data type is stored in the buffer as byte-wise sortable, meaning buffers can be compared at the byte level for sorting *without deserializing*.

**Primary Key Management** Compound sortable keys are extremely easy to generate, maintain and update with NoProto. You don't need a custom sort function in your key-value store, you just need this library.

**UUID & ULID Support** NoProto is one of the few formats that come with first class suport for these popular primary key data types.  It can easily encode, decode and generate these data types.

**Fastest Updates** NoProto is the only format that supports *all mutations* without deserializng.  It can do the common database read -> update -> write operation between 50x - 300x faster than other dynamic formats. [Benchamrks](#benchmarks)


### Comparison With Other Formats

<br/>
<details>
<summary><b>Compared to Apache Avro</b></summary>
- Far more space efficient<br/>
- Significantly faster serialization & deserialization<br/>
- All values are optional (no void or null type)<br/>
- Supports more native types (like unsigned ints)<br/>
- Updates without deserializng/serializing<br/>
- Works with `no_std`.<br/>
- Safely handle untrusted data.<br/>
</details>
<br/>
<details>
<summary><b>Compared to Protocol Buffers</b></summary>
- Comparable serialization & deserialization performance<br/>
- Updating buffers is an order of magnitude faster<br/>
- Schemas are dynamic at runtime, no compilation step<br/>
- All values are optional<br/>
- Supports more types and better nested type support<br/>
- Byte-wise sorting is first class operation<br/>
- Updates without deserializng/serializing<br/>
- Safely handle untrusted data.<br/>
</details>
<br/>
<details>
<summary><b>Compared to JSON / BSON</b></summary>
- Far more space efficient<br/>
- Significantly faster serialization & deserialization<br/>
- Deserializtion is zero copy<br/>
- Has schemas / type safe<br/>
- Supports byte-wise sorting<br/>
- Supports raw bytes & other native types<br/>
- Updates without deserializng/serializing<br/>
- Works with `no_std`.<br/>
- Safely handle untrusted data.<br/>
</details>
<br/>
<details>
<summary><b>Compared to Flatbuffers / Bincode</b></summary>
- Data types can change or be created at runtime<br/>
- Updating buffers is an order of magnitude faster<br/>
- Supports byte-wise sorting<br/>
- Updates without deserializng/serializing<br/>
- Works with `no_std`.<br/>
- Safely handle untrusted data.<br/>
</details>
<br/><br/>

| Format           | Zero-Copy | Size Limit | Mutable | Schemas  | Byte-wise Sorting |
|------------------|-----------|------------|---------|----------|-------------------|
| **Runtime Libs** |           |            |         |          |                   | 
| *NoProto*        | ✓         | ~64KB      | ✓       | ✓        | ✓                 |
| Apache Avro      | 𐄂         | 2^63 Bytes | 𐄂       | ✓        | ✓                 |
| JSON             | 𐄂         | Unlimited  | ✓       | 𐄂        | 𐄂                 |
| BSON             | 𐄂         | ~16MB      | ✓       | 𐄂        | 𐄂                 |
| MessagePack      | 𐄂         | Unlimited  | ✓       | 𐄂        | 𐄂                 |
| **Compiled Libs**|           |            |         |          |                   | 
| FlatBuffers      | ✓         | ~2GB       | 𐄂       | ✓        | 𐄂                 |
| Bincode          | ✓         | ?          | ✓       | ✓        | 𐄂                 |
| Protocol Buffers | 𐄂         | ~2GB       | 𐄂       | ✓        | 𐄂                 |
| Cap'N Proto      | ✓         | 2^64 Bytes | 𐄂       | ✓        | 𐄂                 |
| Veriform         | 𐄂         | ?          | 𐄂       | 𐄂        | 𐄂                 |


# Quick Example
```rust
use no_proto::error::NP_Error;
use no_proto::NP_Factory;

// JSON is used to describe schema for the factory
// Each factory represents a single schema
// One factory can be used to serialize/deserialize any number of buffers
let user_factory = NP_Factory::new(r#"{
    "type": "struct",
    "fields": [
        ["name",   {"type": "string"}],
        ["age",    {"type": "u16", "default": 0}],
        ["tags",   {"type": "list", "of": {
            "type": "string"
        }}]
    ]
}"#)?;


// create a new empty buffer
let mut user_buffer = user_factory.empty_buffer(None); // optional capacity

// set the "name" field
user_buffer.set(&["name"], "Billy Joel")?;

// read the "name" field
let name = user_buffer.get::<&str>(&["name"])?;
assert_eq!(name, Some("Billy Joel"));

// set a nested value, the first tag in the tag list
user_buffer.set(&["tags", "0"], "first tag")?;

// read the first tag from the tag list
let tag = user_buffer.get::<&str>(&["tags", "0"])?;
assert_eq!(tag, Some("first tag"));

// close buffer and get internal bytes
let user_bytes: Vec<u8> = user_buffer.close();

// open the buffer again
let user_buffer = user_factory.open_buffer(user_bytes);

// read the "name" field again
let name = user_buffer.get::<&str>(&["name"])?;
assert_eq!(name, Some("Billy Joel"));

// get the age field
let age = user_buffer.get::<u16>(&["age"])?;
// returns default value from schema
assert_eq!(age, Some(0u16));

// close again
let user_bytes: Vec<u8> = user_buffer.close();


// we can now save user_bytes to disk, 
// send it over the network, or whatever else is needed with the data


# Ok::<(), NP_Error>(()) 
```

## Guided Learning / Next Steps:
1. [`Schemas`](https://docs.rs/no_proto/latest/no_proto/schema/index.html) - Learn how to build & work with schemas.
2. [`Factories`](https://docs.rs/no_proto/latest/no_proto/struct.NP_Factory.html) - Parsing schemas into something you can work with.
3. [`Buffers`](https://docs.rs/no_proto/latest/no_proto/buffer/struct.NP_Buffer.html) - How to create, update & compact buffers/data.
4. [`RPC Framework`](https://docs.rs/no_proto/latest/no_proto/rpc/index.html) - How to use the RPC Framework APIs.
5. [`Data & Schema Format`](https://docs.rs/no_proto/latest/no_proto/format/index.html) - Learn how data is saved into the buffer and schemas.

## Benchmarks
While it's difficult to properly benchmark libraries like these in a fair way, I've made an attempt in the graph below.  These benchmarks are available in the `bench` folder and you can easily run them yourself with `cargo run --release`. 

The format and data used in the benchmarks were taken from the `flatbuffers` benchmarks github repo.  You should always benchmark/test your own use case for each library before making any choices on what to use.

**Legend**: Ops / Millisecond, higher is better

| Library            | Encode | Decode All | Decode 1 | Update 1 | Size (bytes) | Size (Zlib) |
|--------------------|--------|------------|----------|----------|--------------|-------------|
| **Runtime Libs**   |        |            |          |          |              |             |
| *NoProto*          |   1006 |       1575 |    38462 |    11628 |          209 |         167 |
| Apache Avro        |    156 |         57 |       57 |       41 |          702 |         338 |
| FlexBuffers        |    449 |        954 |    24390 |      298 |          490 |         309 |
| JSON               |    587 |        489 |      581 |      436 |          439 |         184 |
| BSON               |    129 |        116 |      124 |       91 |          414 |         216 |
| MessagePack        |    670 |        620 |      818 |      200 |          311 |         193 |
| **Compiled Libs**  |        |            |          |          |              |             |
| Flatbuffers        |   3279 |      16393 |   200000 |     2674 |          264 |         181 |
| Bincode            |   5988 |       9901 |    10526 |     4651 |          163 |         129 |
| Protobuf           |    991 |       1290 |     1307 |      530 |          154 |         141 |
| Prost              |   1520 |       2114 |     2217 |     1091 |          154 |         142 |
| Rkyv               |   2618 |      31250 |   200000 |        0 |          180 |         152 |


- **Encode**: Transfer a collection of fields of test data into a serialized `Vec<u8>`.
- **Decode All**: Deserialize the test object from the `Vec<u8>` into all fields.
- **Decode 1**: Deserialize the test object from the `Vec<u8>` into one field.
- **Update 1**: Deserialize, update a single field, then serialize back into `Vec<u8>`.

**Runtime VS Compiled Libs**: Some formats require data types to be compiled into the application, which increases performance but means data types *cannot change at runtime*.  If data types need to mutate during runtime or can't be known before the application is compiled (like with databases), you must use a format that doesn't compile data types into the application, like JSON or NoProto.

Complete benchmark source code is available [here](https://github.com/only-cliches/NoProto/tree/master/bench).

## NoProto Strengths
If your use case fits any of the points below, NoProto is a good choice for your application.  You should always benchmark to verify.

1. Flexible At Runtime<br/>
If you need to work with data types that will change or be created at runtime, you normally have to pick something like JSON since highly optimized formats like Flatbuffers and Bincode depend on compiling the data types into your application (making everything fixed at runtime). When it comes to formats that can change/implement data types at runtime, NoProto is fastest format I've been able to find (if you know if one that might be faster, let me know!).

2. Safely Accept Untrusted Data</br>
The worse case failure mode for NoProto buffers is junk data.  While other formats can cause denial of service attacks or allow unsafe memory access, there is no such failure case with NoProto.  There is no way to construct a NoProto buffer that would cause any detrement in performance to the host application or lead to unsafe memory access.  Also, there is no panic causing code in the library, meaning it will never crash your application.

3. Extremely Fast Updates<br/>
If you have a workflow in your application that is read -> modify -> write with buffers, NoProto will usually outperform every other format, including Bincode and Flatbuffers. This is because NoProto never actually deserializes, it doesn't need to.  This includes complicated mutations like pushing a value onto a list or adding a value into the middle of a list.

4. Incremental Deserializing<br/>
You only pay for the fields you read, no more. There is no deserializing step in NoProto, opening a buffer typically performs no operations (except for sorted buffers, which is opt in). Once you start asking for fields, the library will navigate the buffer using the format rules to get just what you asked for and nothing else. If you have a workflow in your application where you read a buffer and only grab a few fields inside it, NoProto will outperform most other libraries.

5. Bytewise Sorting<br/>
Almost all of NoProto's data types are designed to serialize into bytewise sortable values, *including signed integers*.  When used with Tuples, making database keys with compound sorting is extremly easy.  When you combine that with first class support for `UUID`s and `ULID`s NoProto makes an excellent tool for parsing and creating primary keys for databases like RocksDB, LevelDB and TiKV. 

6. `no_std` Support<br/>
If you need a serialization format with low memory usage that works in `no_std` environments, NoProto is one of the few good choices.


### When to use Flatbuffers / Bincode / CapN Proto
If you can safely compile all your data types into your application, all the buffers/data is trusted, and you don't intend to mutate buffers after they're created, Bincode/Flatbuffers/CapNProto is a better choice for you.

### When to use JSON / BSON / MessagePack
If your data changes so often that schemas don't really make sense or the format you use must be self describing, JSON/BSON/MessagePack is a better choice.   Although I'd argue that if you *can* make schemas work you should.  Once you can use a format with schemas you save a ton of space in the resulting buffers and performance far better.

## Limitations
- Collections (Map, Tuple, List & Struct) cannot have more than 255 items.  You can nest to get more capacity, for example a list of lists can have up to 255 * 255 items.
- You cannot nest more than 255 levels deep.
- Struct field names cannot be longer than 255 UTF8 bytes.
- Enum/Option types are limited to 255 options and each option cannot be more than 255 UTF8 Bytes.
- Map keys cannot be larger than 255 UTF8 bytes.
- Buffers cannot be larger than 2^16 bytes or ~64KB.

----------------------

MIT License

Copyright (c) 2021 Scott Lott

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