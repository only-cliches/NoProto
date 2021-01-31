//! Schemas are used to describe the shape and types of buffer objects
//! 
//! NoProto schemas describe how the data in a buffer is stored and what types of data are stored.  Schemas are required to create buffers and each buffer is a descendant of the schema that created it.
//! 
//! Schemas can be created with JSON, ES6 or Bytes.
//! 
//! As a quick example, the schemas below are indentical in what they describe, only different in syntax.
//! ```text
//! /* List Of Strings */
//! 
//! // JSON Schema
//! {"type": "list", "of": {"type": "string"}}
//! 
//! // ES6 Schema
//! list({of: string()})
//! 
//! // Byte schema (not human readable)
//! [23, 2, 0, 0, 0, 0, 0]
//! ```
//! 
//! NoProto provides complete import and export interop for all schema syntax variants.  You can create a NoProto factory using any schema syntax then export to any syntax.  This means you can compile your schema into bytes using the runtime, then later expand the bytes schema to JSON or IDL if you need to inspect it.
//! 
//! Buffers are forever related to the schema that created them, buffers created from a given schema can only later be decoded, edited or compacted by that same schema or a safe mutation of it.
//! 
//! Schemas are validated and sanity checked upon creation.  You cannot pass an invalid JSON or ES6 schema into a factory constructor and build/parse buffers with it.  
//! 
//! Schemas can be as simple as a single scalar type, for example a perfectly valid schema for a buffer that contains only a string:
//! ```text
//! // JSON
//! {
//!     "type": "string"
//! }
//! // ES6
//! string()
//! ```
//! 
//! However, you will likely want to store more complicated objects, so that's easy to do as well.
//! ```text
//! // JSON
//! {
//!     "type": "struct",
//!     "fields": [
//!         ["userID",   {"type": "string"}], // userID field contains a string
//!         ["password", {"type": "string"}], // password field contains a string
//!         ["email",    {"type": "string"}], // email field contains a string
//!         ["age",      {"type": "u8"}]     // age field contains a Uint8 number (0 - 255)
//!     ]
//! }
//! // ES6
//! struct({fields: {
//!     userID: string(),    // userID field contains a string
//!     password: string(),  // password field contains a string
//!     email: string(),     // email field contains a string
//!     age: u8()            // age field contains a Uint8 number (0 - 255)
//! }})
//! ```
//! 
//! There are multiple collection types, and they can be nested.
//! 
//! For example, this is a list of structs.  Every item in the list is a struct with two fields: id and title.  Both fields are a string type.
//! ```text
//! // JSON
//! {
//!     "type": "list",
//!     "of": {
//!         "type": "struct",
//!         "fields": [
//!             ["id",    {"type": "string"}]
//!             ["title", {"type": "string"}]
//!         ]
//!     }
//! }
//! 
//! // ES6
//! list({of: struct({fields: {
//!     id: string(),
//!     title: string()
//! }})})
//! ```
//! You can nest collections as much and however you'd like, up to 255 levels.
//! 
//! A list of strings is just as easy...
//! 
//! ```text
//! // JSON
//! {
//!     "type": "list",
//!     "of": { "type": "string" }
//! }
//! 
//! // ES6
//! list({of: string()})
//! ```
//! 
//! **ES6 Schemas**<br/>
//! NoProto's ES6/Javascript IDL schemas use a **very** strict subset of the ES6 syntax. Expressions like `2 + 3`, variables and most other javascripty things aren't supported.  The ES6 IDL is not intended to provide a JS runtime, only a familiar syntax.
//! 
//! The following ES6 syntax is supported:
//! - Calling functions with or without arguments like `myFn()`, `myFn(1, 2)`, or `myFn("hello", [1, 2])`
//! - Single line comments on their own line or at the end of a line using double slash `//`.
//! - Arrays with any valid JS object.  Examples: `[]`, `[1, 2]`, `["hello", myFn()]`
//! - Objects with string keys and any valid JS object for values.  **Keys cannot use quotes**.  Examples: `{}`, `{key: "value"}`, `{foo: "bar", baz: myFn()}`
//! - Arrays and objects can be safely nested.  There is a nesting limit of 255 levels.
//! - Numbers, Strings contained in double quotes '`"`', and Boolean values.
//! - Strings can safely contain escaped double quotes `\"` inside them.
//! - ES6 arrow methods that contain comments or statements seperated by semicolons. Example: `() => { string(); }`
//! 
//! If the syntax is not in the above list, it will not be parsed correctly by NoProto.
//! 
//! ES6 schemas are not as expensive to parse as JSON schemas, but nowhere near as fast to parse as byte schemas.
//! 
//! **JSON Schemas**<br/>
//! 
//! If you're familiar with Typescript, JSON schemas can be described by this recursive interface:
//! 
//! ```typescript
//! interface NP_Schema {
//!     // table, string, bytes, etc
//!     type: string; 
//!     
//!     // used by string & bytes types
//!     size?: number;
//!     
//!     // used by decimal type, the number of decimal places every value has
//!     exp?: number;
//!     
//!     // used by tuple to indicite bytewise sorting of children
//!     sorted?: boolean;
//!     
//!     // used by list types
//!     of?: NP_Schema
//!     
//!     // used by map types
//!     value?: NP_Schema
//! 
//!     // used by tuple types
//!     values?: NP_Schema[]
//! 
//!     // used by struct types
//!     fields?: [string, NP_Schema][];
//! 
//!     // used by option/enum types
//!     choices?: string[];
//!     
//!     // used by unions
//!     types?: [string, NP_Schema][];
//!     
//!     // used by portals
//!     to?: string
//! 
//!     // default value for this item
//!     default?: any;
//! }
//! ```
//! 
//! ## Schema Data Types
//! Each type has trade offs associated with it.  The table and documentation below go into further detail.
//! 
//! ### Supported Data Types
//! 
//! | Schema Type                            | Rust Type                                                                | Zero Copy Type   |Bytewise Sorting  | Bytes (Size)    | Limits / Notes                                                           |
//! |----------------------------------------|--------------------------------------------------------------------------|------------------|------------------|-----------------|--------------------------------------------------------------------------|
//! | [`struct`](#struct)                    | [`NP_Struct`](../collection/table/struct.NP_Struct.html)                 | -                |𐄂                 | 2 bytes - ~64Kb | Set of vtables with up to 255 named fields.                             |
//! | [`list`](#list)                        | [`NP_List`](../collection/list/struct.NP_List.html)                      | -                |𐄂                 | 4 bytes - ~64Kb | Linked list with integer indexed values and  up to 255 items.            |
//! | [`map`](#map)                          | [`NP_Map`](../collection/map/struct.NP_Map.html)                         | -                |𐄂                 | 2 bytes - ~64Kb | Linked list with `&str` keys, up to 255 items.                           |
//! | [`tuple`](#tuple)                      | [`NP_Tuple`](../collection/tuple/struct.NP_Tuple.html)                   | -                |✓ *               | 2 bytes - ~64Kb | Static sized collection of specific values.  Up to 255 values.           |
//! | [`any`](#any)                          | [`NP_Any`](../pointer/any/struct.NP_Any.html)                            | -                |𐄂                 | 2 bytes - ~64Kb | Generic type.                                                            |
//! | [`string`](#string)                    | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      | &str             |✓ **              | 2 bytes - ~64Kb | Utf-8 formatted string.                                                  |
//! | [`bytes`](#bytes)                      | [`Vec<u8>`](https://doc.rust-lang.org/std/vec/struct.Vec.html)           | &[u8]            |✓ **              | 2 bytes - ~64Kb | Arbitrary bytes.                                                         |
//! | [`int8`](#int8-int16-int32-int64)      | [`i8`](https://doc.rust-lang.org/std/primitive.i8.html)                  | -                |✓                 | 1 byte          | -127 to 127                                                              |
//! | [`int16`](#int8-int16-int32-int64)     | [`i16`](https://doc.rust-lang.org/std/primitive.i16.html)                | -                |✓                 | 2 bytes         | -32,768 to 32,768                                                        |
//! | [`int32`](#int8-int16-int32-int64)     | [`i32`](https://doc.rust-lang.org/std/primitive.i32.html)                | -                |✓                 | 4 bytes         | -2,147,483,648 to 2,147,483,648                                          |
//! | [`int64`](#int8-int16-int32-int64)     | [`i64`](https://doc.rust-lang.org/std/primitive.i64.html)                | -                |✓                 | 8 bytes         | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,808                  |
//! | [`uint8`](#uint8-uint16-uint32-uint64) | [`u8`](https://doc.rust-lang.org/std/primitive.u8.html)                  | -                |✓                 | 1 byte          | 0 - 255                                                                  |
//! | [`uint16`](#uint8-uint16-uint32-uint64)| [`u16`](https://doc.rust-lang.org/std/primitive.u16.html)                | -                |✓                 | 2 bytes         | 0 - 65,535                                                               |
//! | [`uint32`](#uint8-uint16-uint32-uint64)| [`u32`](https://doc.rust-lang.org/std/primitive.u32.html)                | -                |✓                 | 4 bytes         | 0 - 4,294,967,295                                                        |
//! | [`uint64`](#uint8-uint16-uint32-uint64)| [`u64`](https://doc.rust-lang.org/std/primitive.u64.html)                | -                |✓                 | 8 bytes         | 0 - 18,446,744,073,709,551,616                                           |
//! | [`float`](#float-double)               | [`f32`](https://doc.rust-lang.org/std/primitive.f32.html)                | -                |𐄂                 | 4 bytes         | -3.4e38 to 3.4e38                                                        |
//! | [`double`](#float-double)              | [`f64`](https://doc.rust-lang.org/std/primitive.f64.html)                | -                |𐄂                 | 8 bytes         | -1.7e308 to 1.7e308                                                      |
//! | [`enum`](#enum)                        | [`NP_Enum`](../pointer/option/struct.NP_Enum.html)                       | -                |✓                 | 1 byte          | Up to 255 string based options in schema.                                |
//! | [`bool`](#bool)                        | [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)              | -                |✓                 | 1 byte          |                                                                          |
//! | [`decimal`](#decimal)                  | [`NP_Dec`](../pointer/dec/struct.NP_Dec.html)                            | -                |✓                 | 8 bytes         | Fixed point decimal number based on i64.                                 |
//! | [`geo4`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/geo/struct.NP_Geo.html)                            | -                |✓                 | 4 bytes         | 1.1km resolution (city) geographic coordinate                            |
//! | [`geo8`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/geo/struct.NP_Geo.html)                            | -                |✓                 | 8 bytes         | 11mm resolution (marble) geographic coordinate                           |
//! | [`geo16`](#geo4-geo8-geo16)            | [`NP_Geo`](../pointer/geo/struct.NP_Geo.html)                            | -                |✓                 | 16 bytes        | 110 microns resolution (grain of sand) geographic coordinate             |
//! | [`ulid`](#ulid)                        | [`NP_ULID`](../pointer/ulid/struct.NP_ULID.html)                         | &NP_ULID         |✓                 | 16 bytes        | 6 bytes for the timestamp (5,224 years), 10 bytes of randomness (1.2e24) |
//! | [`uuid`](#uuid)                        | [`NP_UUID`](../pointer/uuid/struct.NP_UUID.html)                         | &NP_UUID         |✓                 | 16 bytes        | v4 UUID, 2e37 possible UUIDs                                             |
//! | [`date`](#date)                        | [`NP_Date`](../pointer/date/struct.NP_Date.html)                         | -                |✓                 | 8 bytes         | Good to store unix epoch (in milliseconds) until the year 584,866,263    |
//! | [`portal`](#portal)                    | -                                                                        | -                |𐄂                 | 0 bytes         | A type that just points to another type in the buffer.                   | 
//! 
//! - \* `sorting` must be set to `true` in the schema for this object to enable sorting.
//! - \*\* String & Bytes can be bytewise sorted only if they have a `size` property in the schema
//! 
//! # Legend
//! 
//! **Bytewise Sorting**<br/>
//! Bytewise sorting means that two buffers can be compared at the byte level *without deserializing* and a correct ordering between the buffer's internal values will be found.  This is extremely useful for storing ordered keys in databases.
//! 
//! Each type has specific notes on wether it supports bytewise sorting and what things to consider if using it for that purpose.
//! 
//! You can sort by multiple types/values if a tuple is used.  The ordering of values in the tuple will determine the sort order.  For example if you have a tuple with types (A, B) the ordering will first sort by A, then B where A is identical.  This is true for any number of items, for example a tuple with types (A,B,C,D) will sort by D when A, B & C are identical.
//! 
//! **Compaction**<br/>
//! Campaction is an optional operation you can perform at any time on a buffer, typically used to recover free space.  NoProto Buffers are contiguous, growing arrays of bytes.  When you add or update a value sometimes additional memory is used and the old value is dereferenced, meaning the buffer is now occupying more space than it needs to.  This space can be recovered with compaction.  Compaction involves a recursive, full copy of all referenced & valid values of the buffer, it's an expensive operation that should be avoided.
//! 
//! Sometimes the space you can recover with compaction is minimal or you can craft your schema and upates in such a way that compactions are never needed, in these cases compaction can be avoided with little to no consequence.
//! 
//! Deleting a value will almost always mean space can be recovered with compaction, but updating values can have different outcomes to the space used depending on the type and options.
//! 
//! Each type will have notes on how updates can lead to wasted bytes and require compaction to recover the wasted space.
//! 
//! - [How do you run compaction on a buffer?](../buffer/struct.NP_Buffer.html#method.compact)
//! 
//! **Schema Mutations**<br/> 
//! Once a schema is created all the buffers it creates depend on that schema for reliable de/serialization, data access, and compaction.
//! 
//! There are safe ways you can mutate a schema after it's been created without breaking old buffers, however those updates are limited.  The safe mutations will be mentioned for each type, consider any other schema mutations unsafe.
//! 
//! Changing the `type` property of any value in the schame is unsafe.  It's only sometimes safe to modify properties besides `type`.
//! 
//! # Schema Types
//! 
//! Every schema type maps exactly to a native data type in your code.
//! 
//! ## struct
//! Structs represnt a fixed number of named fields, with each field having it's own data type.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Fields without values will be removed from the buffer durring compaction.
//! - **Schema Mutations**: The ordering of items in the `fields` property must always remain the same.  It's safe to add new fields to the bottom of the field list or rename fields, but never to remove fields.  field types cannot be changed safely.  If you need to depreciate a field, set it's name to an empty string. 
//! 
//! Struct schemas have a single required property called `fields`.  The `fields` property is an array of arrays that represent all possible fields in the struct and their data types.  Any type can be used in fields, including other structs.  Structs cannot have more than 255 fields, and the field names cannot be longer than 255 UTF8 bytes.
//! 
//! Structs do not store the field names in the buffer, only the field index, so this is a very efficient way to store associated data.
//! 
//! If you need flexible field names use a `map` type instead.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "struct",
//!     "fields": [ // can have between 1 and 255 fields
//!         ["field name",  {"type": "data type for this field"}],
//!         ["name",         {"type": "string"}],
//!         ["tags",         {"type": "list", "of": { // nested list of strings
//!             "type": "string"
//!         }}],
//!         ["age",          {"type": "u8"}], // Uint8 number
//!         ["meta",         {"type": "struct", "fields": [ // nested struct
//!             ["favorite_color",  {"type": "string"}],
//!             ["favorite_sport",  {"type": "string"}]
//!         ]}]
//!     ]
//! }
//! 
//! // ES6
//! struct({fields: {
//!     // data_type() isn't a real data type...
//!     field_name: data_type(),
//!     name: string(),
//!     tags: list({of: string()}),
//!     age: u8(),
//!     meta: struct({fields: {
//!         favorite_color: string(),
//!         favorite_sport: string()
//!     }})
//! }})
//! ```
//! 
//! ## list
//! Lists represent a dynamically sized list of items.  The type for every item in the list is identical and the order of entries is mainted in the buffer.  Lists do not have to contain contiguous entries, gaps can safely and efficiently be stored.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Indexes that have had their value cleared will be removed from the buffer.  If a specific index never had a value, it occupies *zero* space.
//! - **Schema Mutations**: None
//! 
//! Lists have a single required property in the schema, `of`.  The `of` property contains another schema for the type of data contained in the list.  Any type is supported, including another list.  
//! 
//! The more items you have in a list, the slower it will be to seek to values towards the end of the list or loop through the list.
//! 
//! ```text
//! // a list of list of strings
//! // JSON
//! {
//!     "type": "list",
//!     "of": {
//!         "type": "list",
//!         "of": {"type": "string"}
//!     }
//! }
//! // ES6
//! list({of: list({of: string()})})
//! 
//! // list of numbers
//! // JSON
//! {
//!     "type": "list",
//!     "of": {"type": "i32"}
//! }
//! 
//! // ES6
//! list({of: i32()})
//! ```
//! 
//! 
//! ## map
//! A map is a dynamically sized list of items where each key is a `&str`.  Every value of a map has the same type.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Keys without values are removed from the buffer
//! - **Schema Mutations**: None
//! 
//! Maps have a single required property in the schema, `value`. The property is used to describe the schema of the values for the map.  Values can be any schema type, including another map.
//! 
//! If you expect to have fixed, predictable keys then use a `table` type instead.  Maps are less efficient than tables because keys are stored in the buffer.  
//! 
//! The more items you have in a map, the slower it will be to seek to values or loop through the map.  Tables are far more performant for seeking to values.
//! 
//! ```text
//! // a map where every value is a string
//! // JSON
//! {
//!     "type": "map",
//!     "value": {
//!         "type": "string"
//!     }
//! }
//! // ES6
//! map({value: string()})
//! ```
//! 
//! 
//! ## tuple
//! A tuple is a fixed size list of items.  Each item has it's own type and index.  Tuples support up to 255 items.
//! 
//! - **Bytewise Sorting**: Supported if all children are scalars that support bytewise sorting and schema `sorted` is set to `true`.
//! - **Compaction**: If `sorted` is true, compaction will not save space.  Otherwise, tuples only reduce in size if children are deleted or children with a dyanmic size are updated.
//! - **Schema Mutations**: No mutations are safe
//! 
//! Tuples have a single required property in the schema called `values`.  It's an array of schemas that represnt the tuple values.  Any schema is allowed, including other Tuples.
//! 
//! **Sorting**<br/>
//! You can use tuples to support compound bytewise sorting across multiple values of different types.  By setting the `sorted` property to `true` you enable a strict mode for the tuple that enables sorting features.  When `sorted` is enabled only scalar values that support sorting are allowed in the schema.  For example, strings/bytes types can only be fixed size.
//! 
//! When `sorted` is true the order of values is gauranteed to be constant in every buffer and all buffers will be identical in size.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "tuple",
//!     "values": [
//!         {"type": "string"},
//!         {"type": "list", "of": {"type": "strings"}},
//!         {"type": "u64"}
//!     ]
//! }
//! // ES6
//! tuple({values: [string(), list({of: string()}), u64()]})
//! 
//! // tuple for bytewise sorting
//! // JSON
//! {
//!     "type": "tuple",
//!     "sorted": true,
//!     "values": [
//!         {"type": "string", "size": 25},
//!         {"type": "u8"},
//!         {"type": "i64"}
//!     ]
//! }
//! 
//! // ES6
//! tuple({storted: true, values: [
//!     string({size: 25}), 
//!     u8(), 
//!     i64()
//! ]})
//! ```
//!
//! 
//! 
//! ## string
//! A string is a fixed or dynamically sized collection of utf-8 encoded bytes.
//! 
//! - **Bytewise Sorting**: Supported only if `size` property is set in schema.
//! - **Compaction**: If `size` property is set, compaction cannot reclaim space.  Otherwise it will reclaim space unless all updates have been identical in length.
//! - **Schema Mutations**: If the `size` property is set it's safe to make it smaller, but not larger (this may cause existing string values to truncate, though).  If the field is being used for bytewise sorting, no mutation is safe.
//! 
//! The `size` property provides a way to have fixed size strings in your buffers.  If a provided string is larger than the `size` property it will be truncated.  Smaller strings will be padded with white space.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "string"
//! }
//! // ES6
//! string()
//! 
//! 
//! // fixed size
//! // JSON
//! {
//!     "type": "string",
//!     "size": 20
//! }
//! // ES6
//! string({size: 20})
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "string",
//!     "default": "Default string value"
//! }
//! 
//! // ES6
//! string({default: "Default string value"})
//! ```
//! 
//! More Details:
//! - [Using String data type](../pointer/string/index.html)
//! 
//! ## bytes
//! Bytes are fixed or dynimcally sized Vec<u8> collections. 
//! 
//! - **Bytewise Sorting**: Supported only if `size` property is set in schema.
//! - **Compaction**: If `size` property is set, compaction cannot reclaim space.  Otherwise it will reclaim space unless all updates have been identical in length.
//! - **Schema Mutations**: If the `size` property is set it's safe to make it smaller, but not larger (this may cause existing bytes values to truncate, though).  If the field is being used for bytewise sorting, no mutation is safe.
//! 
//! The `size` property provides a way to have fixed size `&[u8]` in your buffers.  If a provided byte slice is larger than the `size` property it will be truncated.  Smaller byte slices will be padded with zeros.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "bytes"
//! }
//! // ES6
//! bytes()
//! 
//! // fixed size
//! // JSON
//! {
//!     "type": "bytes",
//!     "size": 20
//! }
//! // ES6
//! bytes({size: 20})
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "bytes",
//!     "default": [1, 2, 3, 4]
//! }
//! 
//! // ES6
//! bytes({default: [1, 2, 3, 4]})
//! ```
//! 
//! More Details:
//! - [Using NP_Bytes data type](../pointer/bytes/struct.NP_Bytes.html)
//! 
//! ## int8, int16, int32, int64
//! Signed integers allow positive or negative whole numbers to be stored.  The bytes are stored in big endian format and converted to unsigned types to allow bytewise sorting.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "i8"
//! }
//! 
//! // ES6
//! i8()
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "i8",
//!     "default": 20
//! }
//! 
//! // ES6
//! i8({default: 20})
//! ```
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! More Details:
//! - [Using number data types](../pointer/numbers/index.html)
//! 
//! ## uint8, uint16, uint32, uint64
//! Unsgined integers allow only positive whole numbers to be stored.  The bytes are stored in big endian format to allow bytewise sorting.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "u8"
//! }
//! 
//! // ES6
//! u8()
//! 
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "u8",
//!     "default": 20
//! }
//! // ES6
//! u8({default: 20})
//! ```
//! 
//! More Details:
//! - [Using number data types](../pointer/numbers/index.html)
//! 
//! ## float, double
//! Allows the storage of floating point numbers of various sizes.  Bytes are stored in big endian format.
//! 
//! - **Bytewise Sorting**: Unsupported, use decimal type.
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "f32"
//! }
//! 
//! // ES6
//! f32()
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "f32",
//!     "default": 20.283
//! }
//! 
//! // ES6
//! f32({default: 20.283})
//! 
//! ```
//! 
//! More Details:
//! - [Using number data types](../pointer/numbers/index.html)
//! 
//! ## enum
//! Allows efficeint storage of a selection between a known collection of ordered strings.  The selection is stored as a single u8 byte, limiting the max number of choices to 255.  Also the choices themselves cannot be longer than 255 UTF8 bytes each.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: You can safely add new choices to the end of the list or update the existing choices in place.  If you need to delete a choice, just make it an empty string.  Changing the order of the choices is destructive as this type only stores the index of the choice it's set to.
//! 
//! There is one required property of this schema called `choices`.  The property should contain an array of strings that represent all possible choices of the option.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "enum",
//!     "choices": ["choice 1", "choice 2", "etc"]
//! }
//! // ES6
//! enum({choices: ["choice 1", "choice 2", "etc"]})
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "enum",
//!     "choices": ["choice 1", "choice 2", "etc"],
//!     "default": "etc"
//! }
//! 
//! // ES6
//! enum({choices: ["choice 1", "choice 2", "etc"], default: "etc"})
//! ```
//! 
//! More Details:
//! - [Using NP_Enum data type](../pointer/option/struct.NP_Enum.html)
//! 
//! ## bool
//! Allows efficent storage of a true or false value.  The value is stored as a single byte that is set to either 1 or 0.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "bool"
//! }
//! // ES6
//! bool()
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "bool",
//!     "default": false
//! }
//! // ES6
//! bool({default: false})
//! ```
//! 
//! More Details:
//! 
//! ## decimal
//! Allows you to store fixed point decimal numbers.  The number of decimal places must be declared in the schema as `exp` property and will be used for every value.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! There is a single required property called `exp` that represents the number of decimal points every value will have.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "decimal",
//!     "exp": 3
//! }
//! // ES6
//! decimal({exp: 3})
//! 
//! // with default value
//! // JSON
//! {
//!     "type": "decimal",
//!     "exp": 3,
//!     "default": 20.293
//! }
//! // ES6
//! decimal({exp: 3, default: 20.293})
//! ```
//! 
//! More Details:
//! - [Using NP_Dec data type](../pointer/dec/struct.NP_Dec.html)
//! 
//! ## geo4, ge8, geo16
//! Allows you to store geographic coordinates with varying levels of accuracy and space usage.  
//! 
//! - **Bytewise Sorting**: Not supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! Larger geo values take up more space, but allow greater resolution.
//! 
//! | Type  | Bytes | Earth Resolution                       | Decimal Places |
//! |-------|-------|----------------------------------------|----------------|
//! | geo4  | 4     | 1.1km resolution (city)                | 2              |
//! | geo8  | 8     | 11mm resolution (marble)               | 7              |
//! | geo16 | 16    | 110 microns resolution (grain of sand) | 9              |
//! 
//! ```text
//! // JSON
//! {
//!     "type": "geo4"
//! }
//! // ES6
//! geo4()
//! 
//! // with default
//! {
//!     "type": "geo4",
//!     "default": {"lat": -20.283, "lng": 19.929}
//! }
//! // ES6
//! geo4({default: {lat: -20.283, lng: 19.929}})
//! ```
//! 
//! More Details:
//! - [Using NP_Geo data type](../pointer/geo/struct.NP_Geo.html)
//! 
//! ## ulid
//! Allows you to store a unique ID with a timestamp.  The timestamp is stored in milliseconds since the unix epoch.
//! 
//! - **Bytewise Sorting**: Supported, orders by timestamp. Order is random if timestamp is identical between two values.
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "ulid"
//! }
//! // ES6
//! ulid()
//! // no default supported
//! ```
//! 
//! More Details:
//! - [Using NP_ULID data type](../pointer/ulid/struct.NP_ULID.html)
//! 
//! ## uuid
//! Allows you to store a universally unique ID.
//! 
//! - **Bytewise Sorting**: Supported, but values are random
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "uuid"
//! }
//! // ES6
//! uuid()
//! // no default supported
//! ```
//! 
//! More Details:
//! - [Using NP_UUID data type](../pointer/uuid/struct.NP_UUID.html)
//! 
//! ## date
//! Allows you to store a timestamp as a u64 value.  This is just a thin wrapper around the u64 type.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "date"
//! }
//! // ES6
//! date()
//! 
//! // with default value (default should be in ms)
//! // JSON
//! {
//!     "type": "date",
//!     "default": 1605909163951
//! }
//! // ES6
//! date({default: 1605909163951})
//! ```
//! 
//! More Details:
//! - [Using NP_Date data type](../pointer/date/struct.NP_Date.html)
//!  
//! ## portal
//! Portals allow types/schemas to be "teleported" from one part of a schema to another.
//! 
//! You can use these for duplicating a type many times in a schema or for recursive data types.
//! 
//! The one required property is `to`, it should be a dot notated path to the type being teleported.  If `to` is an empty string, the root is used.
//! 
//! Recursion works up to 255 levels of depth.
//! 
//! - **Bytewise Sorting**: Not Supported
//! - **Compaction**: Same behavior as type being teleported.
//! - **Schema Mutations**: None
//! 
//! ```text
//! // JSON
//! {
//!     "type": "struct",
//!     "fields": [
//!         ["value", {"type": "u8"}],
//!         ["next", {"type": "portal", "to": ""}]
//!     ]
//! }
//! // ES6
//! struct({fields: {
//!     value: u8(),
//!     next: portal({to: ""})
//! }})
//! ```
//! 
//! With the above schema, values can be stored at `value`, `next.value`, `next.next.next.value`, etc.
//! 
//! Here is an example where `portal` is used to duplicate a type.
//! 
//! ```text
//! // JSON
//! {
//!     "type": "struct",
//!     "fields": [
//!         ["username", {"type": "string"}],
//!         ["email", {"type": "portal", "to": "username"}]
//!     ]
//! }
//! // ES6
//! struct({fields: {
//!     username: string(),
//!     email: portal({to: "username"})
//! }})
//! ```
//! 
//! In the schema above `username` and `email` are both resolved to the `string` type.
//! 
//! Even though structs are the only type used in the examples above, the `portal` type will work with any collection type.
//! 
//! 
//! ## Next Step
//! 
//! Read about how to initialize a schema into a NoProto Factory.
//! 
//! [Go to NP_Factory docs](../struct.NP_Factory.html)
//! 

use crate::idl::{JS_AST, JS_Schema};
use crate::{np_path, pointer::{NP_Cursor, union::NP_Union}};
use alloc::string::String;
use core::{fmt::Debug};
use crate::{buffer::DEFAULT_ROOT_PTR_ADDR, json_flex::NP_JSON, memory::NP_Memory_Owned, pointer::{portal::{NP_Portal}, ulid::NP_ULID, uuid::NP_UUID}};
use crate::pointer::any::NP_Any;
use crate::pointer::date::NP_Date;
use crate::pointer::geo::NP_Geo;
use crate::pointer::dec::NP_Dec;
use crate::collection::tuple::NP_Tuple;
use crate::pointer::bytes::NP_Bytes;
use crate::collection::{list::NP_List, struc::NP_Struct, map::NP_Map};
use crate::pointer::{option::NP_Enum, NP_Value};
use crate::error::NP_Error;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Simple enum to store the schema types
#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum NP_TypeKeys {
    None       =  0,
    Any        =  1,
    UTF8String =  2,
    Bytes      =  3,
    Int8       =  4,
    Int16      =  5,
    Int32      =  6,
    Int64      =  7,
    Uint8      =  8,
    Uint16     =  9,
    Uint32     = 10,
    Uint64     = 11,
    Float      = 12,
    Double     = 13,
    Decimal    = 14,
    Boolean    = 15,
    Geo        = 16,
    Uuid       = 17,
    Ulid       = 18,
    Date       = 19,
    Enum       = 20,
    Struct     = 21,
    Map        = 22, 
    List       = 23,
    Tuple      = 24,
    Portal     = 25,
    Union      = 26
}

impl From<u8> for NP_TypeKeys {
    fn from(value: u8) -> Self {
        if value > 26 { return NP_TypeKeys::None; }
        unsafe { core::mem::transmute(value) }
    }
}

impl NP_TypeKeys {
    /// Convert this NP_TypeKey into a specific type index
    pub fn into_type_idx<'idx>(&self) -> (&'idx str, NP_TypeKeys) {
        match self {
            NP_TypeKeys::None       => {    ("none", NP_TypeKeys::None) }
            NP_TypeKeys::Any        => {    NP_Any::type_idx() }
            NP_TypeKeys::UTF8String => {    String::type_idx() }
            NP_TypeKeys::Bytes      => {  NP_Bytes::type_idx() }
            NP_TypeKeys::Int8       => {        i8::type_idx() }
            NP_TypeKeys::Int16      => {       i16::type_idx() }
            NP_TypeKeys::Int32      => {       i32::type_idx() }
            NP_TypeKeys::Int64      => {       i64::type_idx() }
            NP_TypeKeys::Uint8      => {        u8::type_idx() }
            NP_TypeKeys::Uint16     => {       u16::type_idx() }
            NP_TypeKeys::Uint32     => {       u32::type_idx() }
            NP_TypeKeys::Uint64     => {       u64::type_idx() }
            NP_TypeKeys::Float      => {       f32::type_idx() }
            NP_TypeKeys::Double     => {       f64::type_idx() }
            NP_TypeKeys::Decimal    => {    NP_Dec::type_idx() }
            NP_TypeKeys::Boolean    => {      bool::type_idx() }
            NP_TypeKeys::Geo        => {    NP_Geo::type_idx() }
            NP_TypeKeys::Uuid       => {   NP_UUID::type_idx() }
            NP_TypeKeys::Ulid       => {   NP_ULID::type_idx() }
            NP_TypeKeys::Date       => {   NP_Date::type_idx() }
            NP_TypeKeys::Enum       => {   NP_Enum::type_idx() }
            NP_TypeKeys::Struct     => { NP_Struct::type_idx() }
            NP_TypeKeys::Map        => {    NP_Map::type_idx() }
            NP_TypeKeys::List       => {   NP_List::type_idx() }
            NP_TypeKeys::Tuple      => {  NP_Tuple::type_idx() },
            _ => ("", NP_TypeKeys::None)
        }
    }
}

/// Schema Address (usize alias)
pub type NP_Schema_Addr = usize;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum String_Case {
    None = 0,
    Lowercase = 1,
    Uppercase = 2,
}

impl From<u8> for String_Case {
    fn from(value: u8) -> Self {
        if value > 2 { return String_Case::None; }
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
#[allow(missing_docs)]
pub enum NP_Value_Kind {
    Pointer,
    Fixed(u32)
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct NP_Struct_Field {
    pub idx: u8,
    pub col: String,
    pub schema: usize,
    pub offset: usize
}

#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub struct NP_Tuple_Field {
    pub schema: usize,
    pub fixed: bool,
    pub size: usize,
    pub offset: usize
}

/// When a schema is parsed from JSON or Bytes, it is stored in this recursive type
/// 
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum NP_Parsed_Schema {
    None,
    Any        { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys },
    UTF8String { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<String>, size: u16, case: String_Case },
    Bytes      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<Vec<u8>>, size: u16 },
    Int8       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<i8> },
    Int16      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<i16> },
    Int32      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<i32> },
    Int64      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<i64> },
    Uint8      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<u8> },
    Uint16     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<u16> },
    Uint32     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<u32> },
    Uint64     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<u64> },
    Float      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<f32> },
    Double     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<f64> },
    Decimal    { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<NP_Dec>, exp: u8 },
    Boolean    { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<bool> },
    Geo        { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<NP_Geo>, size: u8 },
    Date       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<NP_Date> },
    Enum       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, default: Option<NP_Enum>, choices: Vec<NP_Enum> },
    Uuid       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys },
    Ulid       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys },
    Struct     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, fields: Vec<NP_Struct_Field>, empty: Vec<u8> },
    Map        { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, value: NP_Schema_Addr}, 
    List       { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, of: NP_Schema_Addr },
    Tuple      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, values: Vec<NP_Tuple_Field>, empty: Vec<u8>},
    Portal     { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, path: String, schema: usize, parent_schema: usize },
    Union      { val: NP_Value_Kind, sortable: bool, i:NP_TypeKeys, types: Vec<(u8, String, NP_Schema_Addr)>, default: usize },
}

impl NP_Parsed_Schema {

        /// Get the type key for this schema
        pub fn get_offset(&self) -> &NP_Value_Kind {
            match self {
                NP_Parsed_Schema::None                                 => { &NP_Value_Kind::Pointer }
                NP_Parsed_Schema::Any        { val, .. }     => { val }
                NP_Parsed_Schema::UTF8String { val, .. }     => { val }
                NP_Parsed_Schema::Bytes      { val, .. }     => { val }
                NP_Parsed_Schema::Int8       { val, .. }     => { val }
                NP_Parsed_Schema::Int16      { val, .. }     => { val }
                NP_Parsed_Schema::Int32      { val, .. }     => { val }
                NP_Parsed_Schema::Int64      { val, .. }     => { val }
                NP_Parsed_Schema::Uint8      { val, .. }     => { val }
                NP_Parsed_Schema::Uint16     { val, .. }     => { val }
                NP_Parsed_Schema::Uint32     { val, .. }     => { val }
                NP_Parsed_Schema::Uint64     { val, .. }     => { val }
                NP_Parsed_Schema::Float      { val, .. }     => { val }
                NP_Parsed_Schema::Double     { val, .. }     => { val }
                NP_Parsed_Schema::Decimal    { val, .. }     => { val }
                NP_Parsed_Schema::Boolean    { val, .. }     => { val }
                NP_Parsed_Schema::Geo        { val, .. }     => { val }
                NP_Parsed_Schema::Uuid       { val, .. }     => { val }
                NP_Parsed_Schema::Ulid       { val, .. }     => { val }
                NP_Parsed_Schema::Date       { val, .. }     => { val }
                NP_Parsed_Schema::Enum       { val, .. }     => { val }
                NP_Parsed_Schema::Struct     { val, .. }     => { val }
                NP_Parsed_Schema::Map        { val, .. }     => { val }
                NP_Parsed_Schema::List       { val, .. }     => { val }
                NP_Parsed_Schema::Tuple      { val, .. }     => { val }
                NP_Parsed_Schema::Portal     { val, .. }     => { val }
                NP_Parsed_Schema::Union      { val, .. }     => { val }
            }
        }

    /// Get the type key for this schema
    pub fn get_type_key(&self) -> &NP_TypeKeys {
        match self {
            NP_Parsed_Schema::None                                 => { &NP_TypeKeys::None }
            NP_Parsed_Schema::Any        { i, .. }     => { i }
            NP_Parsed_Schema::UTF8String { i, .. }     => { i }
            NP_Parsed_Schema::Bytes      { i, .. }     => { i }
            NP_Parsed_Schema::Int8       { i, .. }     => { i }
            NP_Parsed_Schema::Int16      { i, .. }     => { i }
            NP_Parsed_Schema::Int32      { i, .. }     => { i }
            NP_Parsed_Schema::Int64      { i, .. }     => { i }
            NP_Parsed_Schema::Uint8      { i, .. }     => { i }
            NP_Parsed_Schema::Uint16     { i, .. }     => { i }
            NP_Parsed_Schema::Uint32     { i, .. }     => { i }
            NP_Parsed_Schema::Uint64     { i, .. }     => { i }
            NP_Parsed_Schema::Float      { i, .. }     => { i }
            NP_Parsed_Schema::Double     { i, .. }     => { i }
            NP_Parsed_Schema::Decimal    { i, .. }     => { i }
            NP_Parsed_Schema::Boolean    { i, .. }     => { i }
            NP_Parsed_Schema::Geo        { i, .. }     => { i }
            NP_Parsed_Schema::Uuid       { i, .. }     => { i }
            NP_Parsed_Schema::Ulid       { i, .. }     => { i }
            NP_Parsed_Schema::Date       { i, .. }     => { i }
            NP_Parsed_Schema::Enum       { i, .. }     => { i }
            NP_Parsed_Schema::Struct     { i, .. }     => { i }
            NP_Parsed_Schema::Map        { i, .. }     => { i }
            NP_Parsed_Schema::List       { i, .. }     => { i }
            NP_Parsed_Schema::Tuple      { i, .. }     => { i }
            NP_Parsed_Schema::Portal     { i, .. }     => { i }
            NP_Parsed_Schema::Union      { i, .. }     => { i }
        }
    }

    /// Get the type data fo a given schema value
    pub fn get_type_data(&self) -> (&str, NP_TypeKeys) {
        match self {
            NP_Parsed_Schema::None =>      ("", NP_TypeKeys::None),
            NP_Parsed_Schema::Portal     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Any        { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::UTF8String { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Bytes      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Int8       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Int16      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Int32      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Int64      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Uint8      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Uint16     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Uint32     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Uint64     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Float      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Double     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Decimal    { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Boolean    { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Geo        { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Uuid       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Ulid       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Date       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Enum       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Struct     { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Map        { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::List       { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Tuple      { i, .. }     => { i.into_type_idx() }
            NP_Parsed_Schema::Union      { i, .. }     => { i.into_type_idx() }
        }
    }

    /// Return if this schema is sortable
    pub fn is_sortable(&self) -> bool {
        match self {
            NP_Parsed_Schema::None => false,
            NP_Parsed_Schema::Any        { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::UTF8String { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Bytes      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Int8       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Int16      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Int32      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Int64      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Uint8      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Uint16     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Uint32     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Uint64     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Float      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Double     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Decimal    { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Boolean    { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Geo        { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Uuid       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Ulid       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Date       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Enum       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Struct     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Map        { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::List       { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Tuple      { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Portal     { sortable, .. }     => { *sortable }
            NP_Parsed_Schema::Union      { sortable, .. }     => { *sortable }
        }
    }
}



/// New NP Schema
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Schema {
    /// is this schema sortable?
    pub is_sortable: bool,
    /// recursive parsed schema
    pub parsed: Vec<NP_Parsed_Schema>
}

impl NP_Schema {

    /// Get a IDL represenatation of this schema
    pub fn to_idl(&self) -> Result<String, NP_Error> {
        NP_Schema::_type_to_idl(&self.parsed, 0)
    }

    /// Recursive function parse schema into IDL
    #[doc(hidden)]
    pub fn _type_to_idl(parsed_schema: &Vec<NP_Parsed_Schema>, address: usize) -> Result<String, NP_Error> {
        match parsed_schema[address] {
            NP_Parsed_Schema::Any        { .. }      => {    NP_Any::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::UTF8String { .. }      => {    String::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Bytes      { .. }      => {  NP_Bytes::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Int8       { .. }      => {        i8::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Int16      { .. }      => {       i16::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Int32      { .. }      => {       i32::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Int64      { .. }      => {       i64::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Uint8      { .. }      => {        u8::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Uint16     { .. }      => {       u16::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Uint32     { .. }      => {       u32::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Uint64     { .. }      => {       u64::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Float      { .. }      => {       f32::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Double     { .. }      => {       f64::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Decimal    { .. }      => {    NP_Dec::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Boolean    { .. }      => {      bool::schema_to_idl(parsed_schema, address) } 
            NP_Parsed_Schema::Geo        { .. }      => {    NP_Geo::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Uuid       { .. }      => {   NP_UUID::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Ulid       { .. }      => {   NP_ULID::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Date       { .. }      => {   NP_Date::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Enum       { .. }      => {   NP_Enum::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Struct     { .. }      => { NP_Struct::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Map        { .. }      => {    NP_Map::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::List       { .. }      => {   NP_List::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Tuple      { .. }      => {  NP_Tuple::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Portal     { .. }      => { NP_Portal::schema_to_idl(parsed_schema, address) }
            NP_Parsed_Schema::Union      { .. }      => {  NP_Union::schema_to_idl(parsed_schema, address) }
            _ => { Ok(String::from("")) }
        }
    }

    /// Get a JSON represenatation of this schema
    pub fn to_json(&self) -> Result<NP_JSON, NP_Error> {
        NP_Schema::_type_to_json(&self.parsed, 0)
    }

    /// Recursive function parse schema into JSON
    #[doc(hidden)]
    pub fn _type_to_json(parsed_schema: &Vec<NP_Parsed_Schema>, address: usize) -> Result<NP_JSON, NP_Error> {
        match parsed_schema[address] {
            NP_Parsed_Schema::Any        { .. }      => {    NP_Any::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::UTF8String { .. }      => {    String::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Bytes      { .. }      => {  NP_Bytes::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Int8       { .. }      => {        i8::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Int16      { .. }      => {       i16::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Int32      { .. }      => {       i32::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Int64      { .. }      => {       i64::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Uint8      { .. }      => {        u8::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Uint16     { .. }      => {       u16::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Uint32     { .. }      => {       u32::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Uint64     { .. }      => {       u64::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Float      { .. }      => {       f32::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Double     { .. }      => {       f64::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Decimal    { .. }      => {    NP_Dec::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Boolean    { .. }      => {      bool::schema_to_json(parsed_schema, address) } 
            NP_Parsed_Schema::Geo        { .. }      => {    NP_Geo::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Uuid       { .. }      => {   NP_UUID::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Ulid       { .. }      => {   NP_ULID::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Date       { .. }      => {   NP_Date::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Enum       { .. }      => {   NP_Enum::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Struct     { .. }      => { NP_Struct::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Map        { .. }      => {    NP_Map::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::List       { .. }      => {   NP_List::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Tuple      { .. }      => {  NP_Tuple::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Portal     { .. }      => { NP_Portal::schema_to_json(parsed_schema, address) }
            NP_Parsed_Schema::Union      { .. }      => {  NP_Union::schema_to_json(parsed_schema, address) }
            _ => { Ok(NP_JSON::Null) }
        }
    }

    /// Get type string for this schema
    #[doc(hidden)]
    pub fn _get_type(json_schema: &Box<NP_JSON>) -> Result<String, NP_Error> {
        match &json_schema["type"] {
            NP_JSON::String(x) => {
                Ok(x.clone())
            },
            _ => {
                Err(NP_Error::new("Schemas must have a 'type' property!"))
            }
        }
    }

    /// Scan the schema for portals and resolve their locations
    pub fn resolve_portals(parsed: Vec<NP_Parsed_Schema>) -> Result<Vec<NP_Parsed_Schema>, NP_Error> {

        let temp_memory = NP_Memory_Owned::new(None, &parsed, DEFAULT_ROOT_PTR_ADDR);

        let mut completed: Vec<NP_Parsed_Schema> = Vec::with_capacity(parsed.len());

        for schema in parsed.iter() {
            match schema {
                NP_Parsed_Schema::Portal { path, .. } => {
                    let root_cursor = NP_Cursor::new(temp_memory.root, 0, 0);
                    let str_path = np_path!(path);
                    match NP_Cursor::select(&temp_memory, root_cursor, false, true, &str_path)? {
                        Some(next) => {
                            completed.push(NP_Parsed_Schema::Portal {
                                val: NP_Value_Kind::Pointer,
                                path: path.clone(),
                                schema: next.schema_addr,
                                parent_schema: next.parent_schema_addr,
                                i: NP_TypeKeys::Portal,
                                sortable: false
                            });
                        },
                        None => return Err(NP_Error::new("Portal 'to' property failed to reoslve!"))
                    }
                },
                _ => { 
                    completed.push(schema.clone());
                }
            }
        }

        Ok(completed)
    }

    /// Generate a schema from a parsed IDL
    pub fn from_idl(parsed: Vec<NP_Parsed_Schema>, idl: &JS_Schema, ast: &JS_AST) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        
        match ast {
            JS_AST::method { name, args } => {
                let type_name = idl.get_str(name).trim();

                match type_name {
                    "any"      => {    NP_Any::from_idl_to_schema(parsed, type_name, idl, args) },
                    "string"   => {    String::from_idl_to_schema(parsed, type_name, idl, args) },
                    "utf8"     => {    String::from_idl_to_schema(parsed, type_name, idl, args) },
                    "str"      => {    String::from_idl_to_schema(parsed, type_name, idl, args) },
                    "bytes"    => {  NP_Bytes::from_idl_to_schema(parsed, type_name, idl, args) },
                    "i8"       => {        i8::from_idl_to_schema(parsed, type_name, idl, args) },
                    "int8"     => {        i8::from_idl_to_schema(parsed, type_name, idl, args) },
                    "i16"      => {       i16::from_idl_to_schema(parsed, type_name, idl, args) },
                    "int16"    => {       i16::from_idl_to_schema(parsed, type_name, idl, args) },
                    "i32"      => {       i32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "int32"    => {       i32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "i64"      => {       i64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "int64"    => {       i64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "u8"       => {        u8::from_idl_to_schema(parsed, type_name, idl, args) },
                    "uint8"    => {        u8::from_idl_to_schema(parsed, type_name, idl, args) },
                    "u16"      => {       u16::from_idl_to_schema(parsed, type_name, idl, args) },
                    "uint16"   => {       u16::from_idl_to_schema(parsed, type_name, idl, args) },
                    "u32"      => {       u32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "uint32"   => {       u32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "u64"      => {       u64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "uint64"   => {       u64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "f32"      => {       f32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "float"    => {       f32::from_idl_to_schema(parsed, type_name, idl, args) },
                    "f64"      => {       f64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "double"   => {       f64::from_idl_to_schema(parsed, type_name, idl, args) },
                    "decimal"  => {    NP_Dec::from_idl_to_schema(parsed, type_name, idl, args) },
                    "dec"      => {    NP_Dec::from_idl_to_schema(parsed, type_name, idl, args) },
                    "bool"     => {      bool::from_idl_to_schema(parsed, type_name, idl, args) },
                    "boolean"  => {      bool::from_idl_to_schema(parsed, type_name, idl, args) },
                    "geo4"     => {    NP_Geo::from_idl_to_schema(parsed, type_name, idl, args) },
                    "geo8"     => {    NP_Geo::from_idl_to_schema(parsed, type_name, idl, args) },
                    "geo16"    => {    NP_Geo::from_idl_to_schema(parsed, type_name, idl, args) },
                    "uuid"     => {   NP_UUID::from_idl_to_schema(parsed, type_name, idl, args) },
                    "ulid"     => {   NP_ULID::from_idl_to_schema(parsed, type_name, idl, args) },
                    "date"     => {   NP_Date::from_idl_to_schema(parsed, type_name, idl, args) },
                    "enum"     => {   NP_Enum::from_idl_to_schema(parsed, type_name, idl, args) },
                    "option"   => {   NP_Enum::from_idl_to_schema(parsed, type_name, idl, args) },
                    "struct"   => { NP_Struct::from_idl_to_schema(parsed, type_name, idl, args) },
                    "list"     => {   NP_List::from_idl_to_schema(parsed, type_name, idl, args) },
                    "array"    => {   NP_List::from_idl_to_schema(parsed, type_name, idl, args) },
                    "map"      => {    NP_Map::from_idl_to_schema(parsed, type_name, idl, args) },
                    "tuple"    => {  NP_Tuple::from_idl_to_schema(parsed, type_name, idl, args) },
                    "portal"   => { NP_Portal::from_idl_to_schema(parsed, type_name, idl, args) },
                    "union"    => {  NP_Union::from_idl_to_schema(parsed, type_name, idl, args) },
                    _ => {
                        let mut err_msg = String::from("Can't find a type that matches this schema! ");
                        err_msg.push_str(idl.get_str(name));
                        Err(NP_Error::new(err_msg.as_str()))
                    }
                }
            },
            _ => { Err(NP_Error::new("Error parsing IDL Schema!")) }
        }
    }

    /// Parse a schema out of schema bytes
    pub fn from_bytes(mut cache: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let this_type = NP_TypeKeys::from(bytes[address]);
        match this_type {
            NP_TypeKeys::None       => {  cache.push(NP_Parsed_Schema::None);  (false, cache) }
            NP_TypeKeys::Any        => {       NP_Any::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::UTF8String => {       String::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Bytes      => {     NP_Bytes::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Int8       => {           i8::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Int16      => {          i16::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Int32      => {          i32::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Int64      => {          i64::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Uint8      => {           u8::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Uint16     => {          u16::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Uint32     => {          u32::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Uint64     => {          u64::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Float      => {          f32::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Double     => {          f64::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Decimal    => {       NP_Dec::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Boolean    => {         bool::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Geo        => {       NP_Geo::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Uuid       => {      NP_UUID::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Ulid       => {      NP_ULID::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Date       => {      NP_Date::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Enum       => {      NP_Enum::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Struct     => {    NP_Struct::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Map        => {       NP_Map::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::List       => {      NP_List::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Tuple      => {     NP_Tuple::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Portal     => {    NP_Portal::from_bytes_to_schema(cache, address, bytes) }
            NP_TypeKeys::Union      => {     NP_Union::from_bytes_to_schema(cache, address, bytes) }
        }
    }

    /// Parse schema from JSON object
    /// 
    /// Given a valid JSON schema, parse and validate, then provide a compiled byte schema.
    /// 
    /// If you need a quick way to convert JSON to schema bytes without firing up an NP_Factory, this will do the trick.
    /// 
    pub fn from_json(schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        match &json_schema["type"] {
            NP_JSON::String(x) => {
                match x.as_str() {
                    "any"      => {    NP_Any::from_json_to_schema(schema, &json_schema) },
                    "str"      => {    String::from_json_to_schema(schema, &json_schema) },
                    "string"   => {    String::from_json_to_schema(schema, &json_schema) },
                    "utf8"     => {    String::from_json_to_schema(schema, &json_schema) },
                    "utf-8"    => {    String::from_json_to_schema(schema, &json_schema) },
                    "bytes"    => {  NP_Bytes::from_json_to_schema(schema, &json_schema) },
                    "[u8]"     => {  NP_Bytes::from_json_to_schema(schema, &json_schema) },
                    "i8"       => {        i8::from_json_to_schema(schema, &json_schema) },
                    "int8"     => {        i8::from_json_to_schema(schema, &json_schema) },
                    "i16"      => {       i16::from_json_to_schema(schema, &json_schema) },
                    "int16"    => {       i16::from_json_to_schema(schema, &json_schema) },
                    "i32"      => {       i32::from_json_to_schema(schema, &json_schema) },
                    "int32"    => {       i32::from_json_to_schema(schema, &json_schema) },
                    "i64"      => {       i64::from_json_to_schema(schema, &json_schema) },
                    "int64"    => {       i64::from_json_to_schema(schema, &json_schema) },
                    "u8"       => {        u8::from_json_to_schema(schema, &json_schema) },
                    "uint8"    => {        u8::from_json_to_schema(schema, &json_schema) },
                    "u16"      => {       u16::from_json_to_schema(schema, &json_schema) },
                    "uint16"   => {       u16::from_json_to_schema(schema, &json_schema) },
                    "u32"      => {       u32::from_json_to_schema(schema, &json_schema) },
                    "uint32"   => {       u32::from_json_to_schema(schema, &json_schema) },
                    "u64"      => {       u64::from_json_to_schema(schema, &json_schema) },
                    "uint64"   => {       u64::from_json_to_schema(schema, &json_schema) },
                    "f32"      => {       f32::from_json_to_schema(schema, &json_schema) },
                    "float"    => {       f32::from_json_to_schema(schema, &json_schema) },
                    "f64"      => {       f64::from_json_to_schema(schema, &json_schema) },
                    "double"   => {       f64::from_json_to_schema(schema, &json_schema) },
                    "dec"      => {    NP_Dec::from_json_to_schema(schema, &json_schema) },
                    "decimal"  => {    NP_Dec::from_json_to_schema(schema, &json_schema) },
                    "bool"     => {      bool::from_json_to_schema(schema, &json_schema) },
                    "boolean"  => {      bool::from_json_to_schema(schema, &json_schema) },
                    "geo4"     => {    NP_Geo::from_json_to_schema(schema, &json_schema) },
                    "geo8"     => {    NP_Geo::from_json_to_schema(schema, &json_schema) },
                    "geo16"    => {    NP_Geo::from_json_to_schema(schema, &json_schema) },
                    "uuid"     => {   NP_UUID::from_json_to_schema(schema, &json_schema) },
                    "ulid"     => {   NP_ULID::from_json_to_schema(schema, &json_schema) },
                    "date"     => {   NP_Date::from_json_to_schema(schema, &json_schema) },
                    "enum"     => {   NP_Enum::from_json_to_schema(schema, &json_schema) },
                    "option"   => {   NP_Enum::from_json_to_schema(schema, &json_schema) },
                    "struct"   => { NP_Struct::from_json_to_schema(schema, &json_schema) },
                    "table"    => { NP_Struct::from_json_to_schema(schema, &json_schema) },
                    "list"     => {   NP_List::from_json_to_schema(schema, &json_schema) },
                    "array"    => {   NP_List::from_json_to_schema(schema, &json_schema) },
                    "map"      => {    NP_Map::from_json_to_schema(schema, &json_schema) },
                    "tuple"    => {  NP_Tuple::from_json_to_schema(schema, &json_schema) },
                    "portal"   => { NP_Portal::from_json_to_schema(schema, &json_schema) },
                    "union"    => {  NP_Union::from_json_to_schema(schema, &json_schema) },
                    _ => {
                        let mut err_msg = String::from("Can't find a type that matches this schema! ");
                        err_msg.push_str(json_schema.stringify().as_str());
                        return Err(NP_Error::new(err_msg.as_str()))
                    }
                }
            },
            _ => {
                Err(NP_Error::new("Schemas must have a 'type' property!"))
            }
        }
    }
}
