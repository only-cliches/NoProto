
## NoProto: Flexible, Fast & Compact Serialization with RPC

<img src="data:image/png;base64,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"/>

[Github](https://github.com/only-cliches/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)

[![MIT license](https://img.shields.io/badge/License-MIT-blue.svg)](https://lbesson.mit-license.org/)
[![GitHub stars](https://img.shields.io/github/stars/only-cliches/NoProto.svg?style=social&label=Star&maxAge=2592000)](https://GitHub.com/Naereen/StrapDown.js/stargazers/)
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
- Supports collections (list, map, table & tuple)
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
let mut user_buffer = user_factory.empty_buffer(None); // optional capacity

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
let user_buffer = user_factory.open_buffer(user_bytes);

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
| *NoProto*          |   1057 |       1437 |    47619 |    12195 |          208 |         166 |
| Apache Avro        |    138 |         51 |       52 |       37 |          702 |         336 |
| FlexBuffers        |    401 |        855 |    23256 |      264 |          490 |         309 |
| JSON               |    550 |        438 |      544 |      396 |          439 |         184 |
| BSON               |    115 |        103 |      109 |       80 |          414 |         216 |
| MessagePack        |    135 |        222 |      237 |      119 |          296 |         187 |
| **Compiled Libs**  |        |            |          |          |              |             |
| Flatbuffers        |   1046 |      14706 |   250000 |     1065 |          264 |         181 |
| Bincode            |   5882 |       8772 |     9524 |     4016 |          163 |         129 |
| Protobuf           |    859 |       1140 |     1163 |      480 |          154 |         141 |
| Prost              |   1225 |       1866 |     1984 |      962 |          154 |         142 |

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
If you have a workflow in your application that is read -> modify -> write with buffers, NoProto will usually outperform every other format, including Bincode and Flatbuffers. This is because NoProto never actually deserializes, it doesn't need to. This library was written with databases in mind, if you want to support client requests like "change username field to X", NoProto will do this faster than any other format, usually orders of magnitude faster. This includes complicated mutations like "push a value onto the end of this nested list".

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
- Collections (Map, Tuple, List & Table) cannot have more than 255 columns/items.  You can nest to get more capacity, for example a list of lists can have up to 255 * 255 items.
- You cannot nest more than 255 levels deep.
- Table colum names cannot be longer than 255 UTF8 bytes.
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