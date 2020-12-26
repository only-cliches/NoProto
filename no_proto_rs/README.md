## Simple, Performant & Safe Serialization with RPC
Performance of Protocol Buffers with flexibility of JSON

[Github](https://github.com/ClickSimply/NoProto) | [Crates.io](https://crates.io/crates/no_proto) | [Documentation](https://docs.rs/no_proto)

### Features  
- Zero dependencies
- Zero copy deserialization
- Safely accept untrusted buffers
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
- Panic/unwrap() free, this library will never cause a panic in your application.
- Simple, powerful transport agnostic [RPC Framework](https://docs.rs/no_proto/latest/no_proto/rpc/index.html).

NoProto allows you to store, read & mutate structured data with very little overhead. It's like Protocol Buffers except schemas are dynamic at runtime and buffers are mutable.  It's like JSON but way faster, type safe and supports native types.  Also unlike Protocol Buffers you can insert values in any order and values can later be removed or updated without rebuilding the whole buffer.

Like Protocol Buffers schemas are seperate from the data buffers and are required to read, create or update data buffers.

Byte-wise sorting comes in the box and is a first class operation. Two NoProto buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be the result.  This is extremely useful for storing ordered keys in databases. 

*Compared to Protocol Buffers*
- Faster serialization & deserialization performance
- Updating buffers is orders of magnitude faster
- Easier & Simpler API
- Schemas are dynamic at runtime, no compilation step
- Supports more types and better nested type support
- Byte-wise sorting is first class operation
- Updates without deserializng/serializing
- Works with `no_std`.
- Safely handle untrusted data.

*Compared to JSON / BSON*
- Far more space efficient
- Significantly faster serialization & deserialization
- Deserializtion is zero copy
- Has schemas / type safe
- Supports byte-wise sorting
- Supports raw bytes & other native types
- Updates without deserializng/serializing
- Works with `no_std`.
- Safely handle untrusted data.

*Compared to Flatbuffers / Bincode*
- Data types can change or be created at runtime
- Supports byte-wise sorting
- Updates without deserializng/serializing
- Works with `no_std`.
- Safely handle untrusted data.


| Format           | Zero-Copy | Size Limit | Mutable | Schemas | Language Agnostic | No Compiling    | Byte-wise Sorting |
|------------------|-----------|------------|---------|---------|-------------------|-----------------|-------------------|
| **NoProto**      | ✓         | ~64KB      | ✓       | ✓       | ✓                 | ✓               | ✓                 |
| JSON             | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| BSON             | 𐄂         | ~16MB      | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| MessagePack      | 𐄂         | Unlimited  | ✓       | 𐄂       | ✓                 | ✓               | 𐄂                 |
| FlatBuffers      | ✓         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
| Bincode          | ✓         | ?          | ✓       | ✓       | 𐄂                 | 𐄂               | 𐄂                 |
| Protocol Buffers | 𐄂         | ~2GB       | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
| Cap'N Proto      | ✓         | 2^64 Bytes | 𐄂       | ✓       | ✓                 | 𐄂               | 𐄂                 |
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

The format and data used in the benchmarks were taken from the `flatbuffers` benchmarks github repo.  You should always benchmark/test your own use case for each library before making any decisions on what to use.

**Legend**: Ops / Millisecond, higher is better

| Library            | Encode | Decode All | Decode 1 | Update 1 | Size (bytes) | Size (Zlib) |
|--------------------|--------|------------|----------|----------|--------------|-------------|
| **Runtime Libs**   |        |            |          |          |              |             | 
| *NoProto*          | 1,209  | 1,653      | 50,000   | 14,085   | 209          | 167         |
| JSON               | 606    | 471        | 605      | 445      | 439          | 184         |
| BSON               | 127    | 122        | 132      | 96       | 414          | 216         |
| MessagePack        | 154    | 242        | 271      | 136      | 296          | 187         |
| **Compiled Libs**  |        |            |          |          |              |             | 
| Flatbuffers        | 1,189  | 15,625     | 250,000  | 1,200    | 264          | 181         |
| Bincode            | 6,250  | 9,434      | 10,309   | 4,367    | 163          | 129         |
| Protocol Buffers 2 | 958    | 1,263      | 1,285    | 556      | 154          | 141         |

- **Encode**: Transfer a collection of fields of test data into a serialized `Vec<u8>`.
- **Decode All**: Deserialize the test object from the `Vec<u8>` into all fields.
- **Decode 1**: Deserialize the test object from the `Vec<u8>` into one field.
- **Update 1**: Deserialize, update a single field, then serialize back into `Vec<u8>`.

**Runtime VS Compiled Libs**: Some formats require your data types to be compiled into your application, which increases performance but means your data types *cannot change at runtime*.  If your data types need to mutate during runtime or you can't know them before your application is compiled (like with databases), you must use a format that doesn't compile data types into your application, like JSON or NoProto.

Complete benchmark source code is available [here](https://github.com/only-cliches/NoProto/tree/master/bench).

## NoProto Strengths
If your use case fits any of the points below, NoProto is a good choice for your application.  You should always benchmark to verify.

1. Flexible At Runtime<br/>
If you need to work with data types that will change or be created at runtime, you normally have to pick something like JSON since highly optimized formats like Flatbuffers and Bincode depend on compiling the data types into your application (making everything fixed at runtime). When it comes to formats that can change/implement data types at runtime, NoProto is fastest format I've been able to find (if you know if one that might be faster, let me know!).

2. Safely Accept Untrusted Data</br>
The worse case failure mode for NoProto buffers is junk data.  While other formats can cause denial of service attacks or allow unsafe memory access, there is no such failure case with NoProto.  There is no way to construct a NoProto buffer that would cause any detrement in performance to the host application or lead to unsafe memory access.  Also, there is no panic causing code in the library, meaning it will never crash your application.

3. Extremely Fast Updates<br/>
If you have a workflow in your application that is read -> modify -> write with buffers, NoProto will usually outperform every other format, including Bincode and Flatbuffers. This is because NoProto never actually deserializes, it doesn't need to. I wrote this library with databases in mind, if you want to support client requests like "change username field to X", NoProto will do this faster than any other format, usually orders of magnitude faster. This includes complicated mutations like "push a value onto the end of this nested list".

4. Incremental Deserializing<br/>
You only pay for the fields you read, no more. There is no deserializing step in NoProto, opening a buffer typically performs no operations (except for sorted buffers, which is opt in). Once you start asking for fields, the library will navigate the buffer using the format rules to get just what you asked for and nothing else. If you have a workflow in your application where you read a buffer and only grab a few fields inside it, NoProto will outperform most other libraries.

5. Bytewise Sorting<br/>
Almost all of NoProto's data types are designed to serialize into bytewise sortable values, *including signed integers*.  When used with Tuples, making database keys with compound sorting is extremly easy.  When you combine that with first class support for `UUID`s and `ULID`s NoProto makes an excellent tool for parsing and creating primary keys for databases like RocksDB, LevelDB and TiKV. 

6. `no_std` Support<br/>
If you need a serialization format with low memory usage that works in `no_std` environments, NoProto is likely the best format choice for you.


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