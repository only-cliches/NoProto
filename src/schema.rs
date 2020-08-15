//! Schemas are JSON used to declare the shape of buffer objects
//! 
//! No Proto Schemas are JSON objects that describe how the data in a buffer is stored and what types of data is stored.  Schemas are required to create buffers and each buffer is a descendant of the schema that created it.
//! 
//! Buffers are forever related to the schema that created them, buffers created from a given schema can only later be decoded, edited or compacted by that same schema.
//! 
//! Schemas are validated and sanity checked upon creation.  You cannot pass an invalid schema into a factory constructor and build/parse buffers with it.
//! 
//! Properties that are not part of the schema are ignored.
//! 
//! If you're familiar with Typescript, schemas can be described by this recursive interface:
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
//!     // used by table types
//!     columns?: [string, NP_Schema][]
//! 
//!     // used by option/enum types
//!     choices?: string[];
//! 
//!     // default value for this item
//!     default?: any;
//! }
//! ```
//! 
//! Schemas can be as simple as a single scalar type, for example a perfectly valid schema for a buffer that contains only a string:
//! ```json
//! {
//!     "type": "string"
//! }
//! ```
//! 
//! However, you will likely want to store more complicated objects, so that's easy to do as well.
//! ```json
//! {
//!     "type": "table",
//!     "columns": [
//!         ["userID",   {"type": "string"}], // userID column contains a string
//!         ["password", {"type": "string"}], // password column contains a string
//!         ["email",    {"type": "string"}], // email column contains a string
//!         ["age",      {"type": "u8"}]     // age column contains a Uint8 number (0 - 255)
//!     ]
//! }
//! ```
//! 
//! There are multiple collection types, and they can be nested.
//! 
//! For example, this is a list of tables.  Each table has two columns: id and title.  Both columns are a string type.
//! ```json
//! {
//!     "type": "list",
//!     "of": {
//!         "type": "table",
//!         "columns": [
//!             ["id",    {type: "string"}]
//!             ["title", {type: "string"}]
//!         ]
//!     }
//! }
//! ```
//! 
//! A list of strings is just as easy...
//! 
//! ```json
//! {
//!     "type": "list",
//!     "of": { type: "string" }
//! }
//! ```
//! 
//! Each type has trade offs associated with it.  The table and documentation below go into further detail.
//! 
//! ## Supported Data Types
//! 
//! | Type                                   | Rust Type / Struct                                                       |Bytewise Sorting  | Bytes (Size)   | Limits / Notes                                                           |
//! |----------------------------------------|--------------------------------------------------------------------------|------------------|----------------|--------------------------------------------------------------------------|
//! | [`table`](#table)                      | [`NP_Table`](../collection/table/struct.NP_Table.html)                   |𐄂                 | 2 bytes - ~4GB | Linked list with indexed keys that map against up to 255 named columns.  |
//! | [`list`](#list)                        | [`NP_List`](../collection/list/struct.NP_List.html)                      |𐄂                 | 4 bytes - ~4GB | Linked list with integer indexed values and  up to 65,535 items.         |
//! | [`map`](#map)                          | [`NP_Map`](../collection/map/struct.NP_Map.html)                         |𐄂                 | 2 bytes - ~4GB | Linked list with `Vec<u8>` keys.                                         |
//! | [`tuple`](#tuple)                      | [`NP_Tuple`](../collection/tuple/struct.NP_Tuple.html)                   |✓ *               | 2 bytes - ~4GB | Static sized collection of specific values.                              |
//! | [`any`](#any)                          | [`NP_Any`](../pointer/any/struct.NP_Any.html)                            |𐄂                 | 2 bytes - ~4GB | Generic type.                                                            |
//! | [`string`](#string)                    | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      |✓ **              | 2 bytes - ~4GB | Utf-8 formatted string.                                                  |
//! | [`bytes`](#bytes)                      | [`NP_Bytes`](../pointer/bytes/struct.NP_Bytes.html)                      |✓ **              | 2 bytes - ~4GB | Arbitrary bytes.                                                         |
//! | [`int8`](#int8-int16-int32-int64)      | [`i8`](https://doc.rust-lang.org/std/primitive.i8.html)                  |✓                 | 1 byte         | -127 to 127                                                              |
//! | [`int16`](#int8-int16-int32-int64)     | [`i16`](https://doc.rust-lang.org/std/primitive.i16.html)                |✓                 | 2 bytes        | -32,768 to 32,768                                                        |
//! | [`int32`](#int8-int16-int32-int64)     | [`i32`](https://doc.rust-lang.org/std/primitive.i32.html)                |✓                 | 4 bytes        | -2,147,483,648 to 2,147,483,648                                          |
//! | [`int64`](#int8-int16-int32-int64)     | [`i64`](https://doc.rust-lang.org/std/primitive.i64.html)                |✓                 | 8 bytes        | -9,223,372,036,854,775,808 to 9,223,372,036,854,775,808                  |
//! | [`uint8`](#uint8-uint16-uint32-uint64) | [`u8`](https://doc.rust-lang.org/std/primitive.u8.html)                  |✓                 | 1 byte         | 0 - 255                                                                  |
//! | [`uint16`](#uint8-uint16-uint32-uint64)| [`u16`](https://doc.rust-lang.org/std/primitive.u16.html)                |✓                 | 2 bytes        | 0 - 65,535                                                               |
//! | [`uint32`](#uint8-uint16-uint32-uint64)| [`u32`](https://doc.rust-lang.org/std/primitive.u32.html)                |✓                 | 4 bytes        | 0 - 4,294,967,295                                                        |
//! | [`uint64`](#uint8-uint16-uint32-uint64)| [`u64`](https://doc.rust-lang.org/std/primitive.u64.html)                |✓                 | 8 bytes        | 0 - 18,446,744,073,709,551,616                                           |
//! | [`float`](#float-double)               | [`f32`](https://doc.rust-lang.org/std/primitive.f32.html)                |𐄂                 | 4 bytes        | -3.4e38 to 3.4e38                                                        |
//! | [`double`](#float-double)              | [`f64`](https://doc.rust-lang.org/std/primitive.f64.html)                |𐄂                 | 8 bytes        | -1.7e308 to 1.7e308                                                      |
//! | [`option`](#option)                    | [`NP_Option`](../pointer/misc/struct.NP_Option.html)                     |✓                 | 1 byte         | Up to 255 string based options in schema.                                |
//! | [`bool`](#bool)                        | [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)              |✓                 | 1 byte         |                                                                          |
//! | [`decimal`](#decimal)                  | [`NP_Dec`](../pointer/misc/struct.NP_Dec.html)                           |✓                 | 8 bytes        | Fixed point decimal number based on i64.                                 |
//! | [`geo4`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/misc/struct.NP_Geo.html)                           |✓                 | 4 bytes        | 1.1km resolution (city) geographic coordinate                            |
//! | [`geo8`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/misc/struct.NP_Geo.html)                           |✓                 | 8 bytes        | 11mm resolution (marble) geographic coordinate                           |
//! | [`geo16`](#geo4-geo8-geo16)            | [`NP_Geo`](../pointer/misc/struct.NP_Geo.html)                           |✓                 | 16 bytes       | 110 microns resolution (grain of sand) geographic coordinate             |
//! | [`ulid`](#ulid)                        | [`NP_ULID`](../pointer/misc/struct.NP_ULID.html)                         |✓                 | 16 bytes       | 6 bytes for the timestamp, 10 bytes of randomness.                       |
//! | [`uuid`](#uuid)                        | [`NP_UUID`](../pointer/misc/struct.NP_UUID.html)                         |✓                 | 16 bytes       | v4 UUID, 2e37 possible UUIDs                                             |
//! | [`date`](#date)                        | [`NP_Date`](../pointer/misc/struct.NP_Date.html)                         |✓                 | 8 bytes        | Good to store unix epoch (in seconds) until the year 584,942,417,355     |
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
//! - [How do you run compaction on a buffer?](../struct.NP_Factory.html#method.compact)
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
//! ## table
//! Tables represnt a fixed number of named columns, with each column having it's own data type.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Columns without values will be removed from the buffer.  If a column never had a value set it is not using *any* space.
//! - **Schema Mutations**: The ordering of items in the `columns` property must always remain the same.  It's safe to add new columns to the bottom of the column list or rename columns, but never to remove columns.  Column types cannot be changed safely.  If you need to depreciate a column, set it's name to an empty string. 
//! 
//! Table schemas have a single required property called `columns`.  The `columns` property is an array of arrays that represent all possible columns in the table and their data types.  Any type can be used in columns, including tables.
//! 
//! Tables do not store the column names in the buffer, only the column index, so this is a very efficient way to store associated data.
//! 
//! If you need dynamic column names a map may be a better use case.
//! 
//! ```json
//! {
//!     "type": "table",
//!     "columns": [ // can have between 1 and 255 columns
//!         ["column name",  {"type": "data type for this column"}],
//!         ["name",         {"type": "string"}],
//!         ["tags",         {"type": "list", "of": { // nested list of strings
//!             "type": "string"
//!         }}],
//!         ["age",          {"type": "u8"}], // Uint8 number
//!         ["meta",         {"type": "table", columns: [ // nested table
//!             ["favorite_color",  {"type": "string"}],
//!             ["favorite_sport",  {"type": "string"}]
//!         ]}]
//!     ]
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Table data type](../collection/table/struct.NP_Table.html)
//! - [Using collection values with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers)
//! 
//! ## list
//! Lists represent a dynamically sized list of items.  The type for every item in the list is identical and the order of entries is mainted in the buffer.  Lists do not have to contain contiguous entries, gaps can safely and efficiently be stored.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Indexes without valuse are removed from the buffer.  If a specific index never had a value, it occupies *zero* space.
//! - **Schema Mutations**: None
//! 
//! Lists have a single required property in the schema, `of`.  The `of` property contains another schema for the type of data contained in the list.  Any type is supported, including another list.
//! 
//! The more items you have in a list, the slower it will be to seek to values at the end of the list. 
//! 
//! ```json
//! // a list of list of strings
//! {
//!     "type": "list",
//!     "of": {
//!         "type": "list",
//!         "of": {"type": "string"}
//!     }
//! }
//! 
//! // list of numbers
//! {
//!     "type": "list",
//!     "of": {"type": "int32"}
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_List data type](../collection/list/struct.NP_List.html)
//! - [Using collection values with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers)
//! 
//! ## map
//! A map is a dynamically sized list of items where each key is a Vec<u8>.  Every value of a map has the same type.
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Keys without values are removed from the buffer
//! - **Schema Mutations**: None
//! 
//! Maps have a single required property in the schema, `value`. The property is used to describe the schema of the values for the map.  Keys are always `Vec<u8>`.  Values can be any schema type, including another map.
//! 
//! If you expect to have fixed, predictable keys then use a `table` instead.  This is less efficient than tables because keys are stored in the buffer.  
//! 
//! ```json
//! // a map where every values are strings
//! {
//!     "type": "map",
//!     "value": {
//!         "type": "string"
//!     }
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Map data type](../collection/map/struct.NP_Map.html)
//! - [Using collection values with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers)
//! 
//! ## tuple
//! A tuple is a fixed size list of items.  Each item has it's own type and index.  Tuples support up to 255 items.
//! 
//! - **Bytewise Sorting**: Supported if all children are scalars that support bytewise sorting and schema `sorted` is set to `true`.
//! - **Compaction**: If `sorted` is true, compaction will not save space.  Otherwise, tuples only reduce in size if children are deleted or children with a dyanmic size are updated.
//! - **Schema Mutations**: If `sorted` is true, none.  Otherwise adding new values to the end of the `values` schema property is safe.
//! 
//! Tuples have a single required property in the schema called `values`.  It's an array of schemas that represnt the tuple values.  Any schema is allowed, including other Tuples.
//! 
//! **Sorting**<br/>
//! You can use tuples to support bytewise sorting across a list of items.  By setting the `sorted` property to `true` you enable a strict mode for the tuple that enables bytewise sorting.  When `sorted` is enabled only scalar values that support sorting are allowed in the schema.  For example, strings/bytes types can only be fixed size.
//! 
//! When `sorted` is true the order of values is gauranteed to be constant across buffers, allowing compound bytewise sorting.
//! 
//! ```json
//! {
//!     "type": "tuple",
//!     "values": [
//!         {"type": "string"},
//!         {"type": "list", "of": {"type": "strings"}},
//!         {"type": "uint64"}
//!     ]
//! }
//! 
//! // tuple for bytewise sorting
//! {
//!     "type": "tuple",
//!     "sorted": true,
//!     "values": [
//!         {"type": "string", "size": 25},
//!         {"type": "uint8"},
//!         {"type": "int64"}
//!     ]
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Tuple data type](../collection/tuple/struct.NP_Tuple.html) 
//! - [Using collection values with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers)
//! 
//! ## any
//! Any types are used to declare that a specific type has no fixed schema but is dynamic.  It's generally not a good idea to use Any types.
//! 
//! When you set `any` in the schema that value can safely be type casted to *anything*, so you can use these to store any type of data.  **However, there is no way to conserve the data through compaction.**
//! 
//! - **Bytewise Sorting**: Unsupported
//! - **Compaction**: Any types are always compacted out of the buffer, data stored behind an `any` schema will be lost after compaction.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "any"
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Any data type](../pointer/any/struct.NP_Any.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## string
//! A string is a fixed or dynamically sized collection of utf-8 encoded bytes.
//! 
//! - **Bytewise Sorting**: Supported only if `size` property is set in schema.
//! - **Compaction**: If `size` property is set, compaction cannot reclaim space.  Otherwise it will reclaim space unless all updates have been identical in length.
//! - **Schema Mutations**: If the `size` property is set it's safe to make it smaller, but not larger (this may cause existing string values to truncate, though).  If the field is being used for bytewise sorting, no mutation is safe.
//! 
//!
//! 
//! ```json
//! {
//!     "type": "string"
//! }
//! // fixed size
//! {
//!     "type": "string",
//!     "size": 20
//! }
//! ```
//! 
//! More Details:
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## bytes
//! Bytes are fixed or dynimcally sized Vec<u8> collections. 
//! 
//! - **Bytewise Sorting**: Supported only if `size` property is set in schema.
//! - **Compaction**: If `size` property is set, compaction cannot reclaim space.  Otherwise it will reclaim space unless all updates have been identical in length.
//! - **Schema Mutations**: If the `size` property is set it's safe to make it smaller, but not larger (this may cause existing bytes values to truncate, though).  If the field is being used for bytewise sorting, no mutation is safe.
//! 
//! ```json
//! {
//!     "type": "bytes"
//! }
//! // fixed size
//! {
//!     "type": "bytes",
//!     "size": 20
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Bytes data type](../pointer/bytes/struct.NP_Bytes.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## int8, int16, int32, int64
//! Signed integers allow positive or negative whole numbers to be stored.  The bytes are stored in big endian format and converted to unsigned types to allow bytewise sorting.
//! 
//! ```json
//! {
//!     "type": "int8"
//! }
//! ```
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! More Details:
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## uint8, uint16, uint32, uint64
//! Unsgined integers allow only positive whole numbers to be stored.  The bytes are stored in big endian format to allow bytewise sorting.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "uint8"
//! }
//! ```
//! 
//! More Details:
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## float, double
//! Allows the storage of floating point numbers of various sizes.  Bytes are stored in big endian format.
//! 
//! - **Bytewise Sorting**: Unsupported, use decimal type.
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "float"
//! }
//! ```
//! 
//! More Details:
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## option
//! Allows efficeint storage of a selection between a known collection of ordered strings.  The selection is stored as a single u8 byte, limiting the max number of choices to 255.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: You can safely add new choices to the end of the list or update the existing choices in place.  If you need to delete a choice, just make it an empty string.  Changing the order of the choices is destructive as this type only stores the index of the choice it's set to.
//! 
//! There is one required property of this schema called `choices`.  The property should contain an array of strings that represent all possible choices of the option.
//! 
//! ```json
//! {
//!     "type": "option",
//!     "choices": ["choice 1", "choice 2", "etc"]
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Option data type](../pointer/misc/struct.NP_Option.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## bool
//! Allows efficent storage of a true or false value.  The value is stored as a single byte that is set to either 1 or 0.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "bool"
//! }
//! ```
//! 
//! More Details:
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
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
//! ```json
//! {
//!     "type": "decimal",
//!     "exp": 3
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Dec data type](../pointer/misc/struct.NP_Dec.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
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
//! | Type  | Bytes | Earth Resolution                       |
//! |-------|-------|----------------------------------------|
//! | geo4  | 4     | 1.1km resolution (city)                |
//! | geo8  | 8     | 11mm resolution (marble)               |
//! | geo16 | 16    | 110 microns resolution (grain of sand) |
//! 
//! ```json
//! {
//!     "type": "geo4"
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Geo data type](../pointer/misc/struct.NP_Geo.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## ulid
//! Allows you to store a unique ID with a timestamp.  The timestamp is stored in milliseconds since the unix epoch.
//! 
//! - **Bytewise Sorting**: Supported, orders by timestamp. Order is random if timestamp is identical between two values.
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "ulid"
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_ULID data type](../pointer/misc/struct.NP_ULID.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## uuid
//! Allows you to store a universally unique ID.
//! 
//! - **Bytewise Sorting**: Supported, but values are random
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "uuid"
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_UUID data type](../pointer/misc/struct.NP_UUID.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//! 
//! ## date
//! Allows you to store a timestamp as a u64 value.  This is just a thin wrapper around the u64 type.
//! 
//! - **Bytewise Sorting**: Supported
//! - **Compaction**: Updates are done in place, never use additional space.
//! - **Schema Mutations**: None
//! 
//! ```json
//! {
//!     "type": "date"
//! }
//! ```
//! 
//! More Details:
//! - [Using NP_Date data type](../pointer/misc/struct.NP_Date.html)
//! - [Using scalar values with pointers](../pointer/struct.NP_Ptr.html#using-scalar-types-with-pointers)
//!  
//! 
//! ## Next Step
//! 
//! Read about how to initialize a schema into a NoProto Factory.
//! 
//! [Go to NP_Factory docs](../struct.NP_Factory.html)
//! 
use crate::json_flex::NP_JSON;
use crate::pointer::any::NP_Any;
use crate::pointer::misc::NP_Date;
use crate::pointer::misc::NP_UUID;
use crate::pointer::misc::NP_ULID;
use crate::pointer::misc::NP_Geo;
use crate::pointer::misc::NP_Dec;
use crate::collection::tuple::NP_Tuple;
use crate::pointer::bytes::NP_Bytes;
use crate::collection::{list::NP_List, table::NP_Table, map::NP_Map};
use crate::pointer::{misc::NP_Option, NP_Value};
use crate::error::NP_Error;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{rc::Rc, string::ToString};


/// Stores the kind of each schema and any property details related to the schema.  Users of the library should never need to create or manipulate this data type.
#[doc(hidden)]
#[derive(Debug)]
pub enum NP_SchemaKinds {
    None,
    Scalar,
    Enum { choices: Vec<String> },
    Table { columns: Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>> },
    List { of: Rc<NP_Schema> },
    Map { value: Rc<NP_Schema> },
    Tuple { sorted: bool, values: Rc<Vec<Rc<NP_Schema>>>}
}

#[derive(Debug)]
#[doc(hidden)]
// These are just used for runtime type comparison, the type information is never stored in the buffer.
// When you cast a pointer to some type, this enum is used as comparing numbers is very efficient.
pub enum NP_TypeKeys {
    Any = 0,
    UTF8String = 1,
    Bytes = 2,
    Int8 = 3,
    Int16 = 4,
    Int32 = 5,
    Int64 = 6,
    Uint8 = 7,
    Uint16 = 8,
    Uint32 = 9,
    Uint64 = 10,
    Float = 11,
    Double = 12,
    Decimal = 13,
    Boolean = 14,
    Geo = 15,
    Uuid = 16,
    Ulid = 17,
    Date = 18,
    Enum = 19,
    Table = 20,
    Map = 21, 
    List = 22,
    Tuple = 23,
    JSON = 24
}

impl From<i64> for NP_TypeKeys {
    fn from(value: i64) -> Self {
        if value == NP_TypeKeys::Any as i64 { return NP_TypeKeys::Any }
        if value == NP_TypeKeys::UTF8String as i64 { return NP_TypeKeys::UTF8String }
        if value == NP_TypeKeys::Bytes as i64 { return NP_TypeKeys::Bytes }
        if value == NP_TypeKeys::Int8 as i64 { return NP_TypeKeys::Int8 }
        if value == NP_TypeKeys::Int16 as i64 { return NP_TypeKeys::Int16 }
        if value == NP_TypeKeys::Int32 as i64 { return NP_TypeKeys::Int32 }
        if value == NP_TypeKeys::Int64 as i64 { return NP_TypeKeys::Int64 }
        if value == NP_TypeKeys::Uint8 as i64 { return NP_TypeKeys::Uint8 }
        if value == NP_TypeKeys::Uint16 as i64 { return NP_TypeKeys::Uint16 }
        if value == NP_TypeKeys::Uint32 as i64 { return NP_TypeKeys::Uint32 }
        if value == NP_TypeKeys::Uint64 as i64 { return NP_TypeKeys::Uint64 }
        if value == NP_TypeKeys::Float as i64 { return NP_TypeKeys::Float }
        if value == NP_TypeKeys::Double as i64 { return NP_TypeKeys::Double }
        if value == NP_TypeKeys::Decimal as i64 { return NP_TypeKeys::Decimal }
        if value == NP_TypeKeys::Boolean as i64 { return NP_TypeKeys::Boolean }
        if value == NP_TypeKeys::Geo as i64 { return NP_TypeKeys::Geo }
        if value == NP_TypeKeys::Uuid as i64 { return NP_TypeKeys::Uuid }
        if value == NP_TypeKeys::Ulid as i64 { return NP_TypeKeys::Ulid }
        if value == NP_TypeKeys::Date as i64 { return NP_TypeKeys::Date }
        if value == NP_TypeKeys::Enum as i64 { return NP_TypeKeys::Enum }
        if value == NP_TypeKeys::Table as i64 { return NP_TypeKeys::Table }
        if value == NP_TypeKeys::Map as i64 { return NP_TypeKeys::Map }
        if value == NP_TypeKeys::List as i64 { return NP_TypeKeys::List }
        if value == NP_TypeKeys::Tuple as i64 { return NP_TypeKeys::Tuple }
        if value == NP_TypeKeys::JSON as i64 { return NP_TypeKeys::JSON }
        NP_TypeKeys::Any
    }
}

#[doc(hidden)]
/// Used to parse JSON into valid schema objects used internally by the library.
/// Users of the library should never need to create or manipulate this data type.
/// 
/// 
#[derive(Debug)]
pub struct NP_Schema {
    pub kind: Box<NP_SchemaKinds>,
    pub type_data: (i64, String),
    pub type_state: i64,
    pub default: Option<NP_JSON>
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Types { }

impl<'a> NP_Types {
    /// check if a given type matches the schema
    pub fn do_check<T: NP_Value + Default>(type_string: &str, json_schema: &NP_JSON)-> Result<Option<NP_Schema>, NP_Error>{
        if T::is_type(type_string) {
            Ok(Some(NP_Schema { 
                kind: Box::new(NP_SchemaKinds::Scalar),
                type_data: T::type_idx(),
                type_state: T::schema_state(type_string, json_schema)?,
                default: Some(json_schema["default"].clone())
            }))
        } else {
            Ok(None)
        }
    }

    /// Compile the JSON schema into a NP_Schema
    pub fn compile_schema(type_string: &str, json_schema: &NP_JSON)-> Result<NP_Schema, NP_Error> {

        let check = NP_Types::do_check::<NP_Any>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NP_Types::do_check::<String>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Bytes>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<f32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<f64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<bool>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Dec>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Geo>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_ULID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NP_Types::do_check::<NP_UUID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Date>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let mut err = type_string.to_owned();
        err.push_str(" is not a valid type!");
        Err(NP_Error::new(err))
    }
}

impl NP_Schema {

    /// Get a blank schema
    pub fn blank() -> NP_Schema {

        NP_Schema {
            kind: Box::new(NP_SchemaKinds::None),
            type_data: (-1, "".to_owned()),
            type_state: 0,
            default: None
        }
    }

    /// Attempt to convert JSON into a valid schema.
    pub fn from_json(json: Box<NP_JSON>) -> Result<Self, NP_Error> {
        NP_Schema::validate_model(&*json)
    }

    /// Validate the JSON as schema.
    pub fn validate_model(json_schema: &NP_JSON) -> Result<Self, NP_Error> {

        if json_schema.is_dictionary() == false {
            return Err(NP_Error::new("Object not found at root of schema!"));
        }


        if json_schema["type"].is_string() == false {
            return Err(NP_Error::new("Must declare a type for every schema!"));
        }

        let type_string = (*json_schema["type"].into_string().unwrap()).as_str();

        if type_string.len() == 0 {
            return Err(NP_Error::new("Must declare a type for every schema!"));
        }


        // validate required properties are in place for each kind
        match type_string {
            "table" => {
                
                let mut columns: Vec<Option<(u8, String, Rc<NP_Schema>)>> = vec![];

                for _x in 0..255 {
                    columns.push(None);
                }

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["columns"].is_array() == false {
                        return Err(NP_Error::new("Table kind requires 'columns' property as array!"));
                    }

                    let mut index = 0;
                    for column in borrowed_schema["columns"].into_vec().unwrap() {

                        if column[0].is_string() == false {
                            return Err(NP_Error::new("Table kind requires all columns have a name!"));
                        }

                        let column_name = column[0].into_string().unwrap();
                        

                        if column_name.len() == 0 {
                            return Err(NP_Error::new("Table kind requires all columns have a name!"));
                        }

                        if column[1].is_dictionary() == false {
                            return Err(NP_Error::new("Table kind requires all columns have a type!"));
                        }

                        let good_schema = NP_Schema::validate_model(&column[1])?;
                        
                        let this_col_obj = &column[1].into_hashmap().unwrap();

                        let use_index = match this_col_obj.get("i") {
                            Some(obj) => {
                                match obj {
                                    NP_JSON::Integer(x) => {
                                        *x as usize
                                    },
                                    _ => index as usize
                                }
                            },
                            None => index as usize
                        };


                        if use_index > 255 {
                            return Err(NP_Error::new("Table cannot have column index above 255!"));
                        }

                        match &columns[use_index] {
                            Some(_x) => {
                                return Err(NP_Error::new("Table column index numbering conflict!"));
                            },
                            None => {
                                columns[use_index] = Some((use_index as u8, column_name.to_string(), Rc::new(good_schema)));
                            }
                        };

                        index += 1;
                    }
                }

                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Table { 
                        columns: Rc::new(columns) 
                    }),
                    type_data: NP_Table::type_idx(),
                    type_state: 0,
                    default: None
                })
            },
            "list" => {

                {
                    let borrowed_schema = json_schema;
                    if borrowed_schema["of"].is_null() || borrowed_schema["of"].is_dictionary() == false {
                        return Err(NP_Error::new("List kind requires 'of' property as schema object!"));
                    }
                }

                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::List { 
                        of: Rc::new(NP_Schema::validate_model(&json_schema["of"])?) 
                    }),
                    type_data: NP_List::<NP_Any>::type_idx(),
                    type_state: 0,
                    default: None
                })
            },
            "map" => {

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["value"].is_null() || borrowed_schema["value"].is_dictionary() == false {
                        return Err(NP_Error::new("Map kind requires 'value' property as schema object!"));
                    }
                }
                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Map { 
                        value: Rc::new(NP_Schema::validate_model(&json_schema["value"])?)
                    }),
                    type_data: NP_Map::<NP_Any>::type_idx(),
                    type_state: 0,
                    default: None
                })
            },
            "tuple" => {

                let mut schemas: Vec<Rc<NP_Schema>> = vec![];

                let sorted;

                {
                    let borrowed_schema = json_schema;

                    sorted = borrowed_schema["sorted"].is_true();

                    if borrowed_schema["values"].is_null() || borrowed_schema["values"].is_array() == false  {
                        return Err(NP_Error::new("Tuple type requires 'values' property as array of schema objects!"));
                    }

                    for schema in borrowed_schema["values"].into_vec().unwrap().into_iter() {
                        let good_schema = NP_Schema::validate_model(schema)?;
                        if sorted {
                            match *good_schema.kind {
                                NP_SchemaKinds::Scalar => { // scalar string and bytes must have fixed sizes

                                    match NP_TypeKeys::from(good_schema.type_data.0) {
                                        NP_TypeKeys::UTF8String => {
                                            if good_schema.type_state == -1 { // no fixed length
                                                return Err(NP_Error::new("String must have `size` property when child of sorted tuple!"));
                                            }
                                        },
                                        NP_TypeKeys::Bytes => {
                                            if good_schema.type_state == -1 { // no fixed length
                                                return Err(NP_Error::new("Bytes must have `size` property when child of sorted tuple!"));
                                            }
                                        },
                                        NP_TypeKeys::Float => {
                                            return Err(NP_Error::new("Float is not a supported child for sorted tuples!"));
                                        },
                                        NP_TypeKeys::Double => {
                                            return Err(NP_Error::new("Double is not a supported child for sorted tuples!"));
                                        },
                                        NP_TypeKeys::Geo => {
                                            return Err(NP_Error::new("Geo is not a supported child for sorted tuples!"));
                                        },
                                        _ => {

                                        }
                                    }
                                },
                                NP_SchemaKinds::Enum { choices: _ } => {
                 
                                },
                                _ => { // table, tuple, list or map
                                    return Err(NP_Error::new("Sorted tuples cannot have a collection type (List, Tuple, Map or Table) as child!"));
                                }
                            }
                        }

                        schemas.push(Rc::new(good_schema));
                    }
                }
            
                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Tuple {
                        sorted: sorted,
                        values: Rc::new(schemas)
                    }),
                    type_data: NP_Tuple::type_idx(),
                    type_state: 0,
                    default: None
                })
            },
            "option" => { 

                let mut options: Vec<String> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["choices"].is_array() == false  {
                        return Err(NP_Error::new("Option kind requires 'choices' property as array of string choices!"));
                    }

                    for option in borrowed_schema["choices"].into_vec().unwrap().into_iter() {
                        if option.is_string() == false {
                            return Err(NP_Error::new("Option kind requires 'choices' property as array of string choices!"));
                        }
                        options.push(option.into_string().unwrap().to_string());
                    }
                }

                if options.len() > 255 {
                    return Err(NP_Error::new("Cannot have more than 255 choices for option type!"));
                }

                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Enum {
                        choices: options
                    }),
                    type_data: NP_Option::type_idx(),
                    type_state: 0,
                    default: Some(json_schema["default"].clone())
                })
            },
            _ => {
                NP_Types::compile_schema(type_string, json_schema)
            }
        }
    }
}