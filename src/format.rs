//! # NoProto Data Format Documentaion
//! 
//! NoProto buffers are built out of pointers and data.  
//! 
//! They are designed to hold a variable amount of data that is parsed based on a schema provided by the client.
//! 
//! 
//! ## Pointers
//! 
//! Pointers contain one or more addresses depending on the pointer type.  The addresses will point to data or other pointers.
//! 
//! There are 2 different address sizes, u16 and u32.  Addresses are always stored in little endian format and addresses are always zero based from the beginning of the buffer.  In other words, address `23` always means 23 bytes from the beginning of the buffer.
//! 
//! | Pointer Kind | u16 size (bytes) | u32 size (bytes) |
//! |--------------|------------------|------------------|
//! | Standard     | 2                | 4                |
//! | Map Item     | 6                | 12               |
//! | List Item    | 6                | 10               |
//!  
//! The first two bytes of every buffer are:
//! 1. The protocol version used for this buffer, currently always 1.  This allows future breaking changes if needed.
//! 2. The address sized used in this buffer.  0 for u32 addresses, 1 for u16 addresses.  All addresses in the buffer are the same size, deteremined by this flag.
//! 
//! The next 2 (for u16) or 4 (for u32) bytes are the root pointer, these bytes should contain the address of the root object in the buffer.
//! 
//! Most of the time these bytes will point to the data immediately following them, but it's possible to clear the root object causing these bytes to be zero, or to update the root data which would cause this address to update to something else.
//! 
//! For example, here is a buffer with u16 address size that contains the string `hello`, it's schema is just `{type: "string"}`.
//! 
//! ```text
//! [       1,        1,         0, 4,          0, 5, 104, 101, 108, 108, 111]
//! [protocol, u16 size, root pointer, string length,   h,   e,   l,   l,   o]
//! ```
//! 
//! Here is the same buffer for u32 address size:
//! ```text
//! [       1,        0,   0, 0, 0, 6,    0, 0, 0, 5, 104, 101, 108, 108, 111]
//! [protocol, u32 size, root pointer, string length,   h,   e,   l,   l,   o]
//! ```
//! 
//! It should be noted that a schema is *required* to parse a buffer, otherwise you don't know the difference between pointers, data and what data types beyond the root.
//! 
//! Let's look at the different pointer types you will encounter in a buffer.
//! 
//! ### Standard Pointer
//! This is used for any scalar or collection data types.  The standard pointer is just a single u16 or u32.
//! 
//! ### Map Item Pointer
//! 
//! Used by items in a map object.  Contains the following:
//! ```text
//! | address of data | next map item pointer address | address of bytes for this key |
//! |     u16/u32     |             u16/u32           |          u16/u32              |
//! ```
//! 
//! Map collections represent a linked list of these pointers.  There should only be map item pointers for items in the map that have data.
//! 
//! The last map item pointer in a map should have a zero in the next item address for no further map items.
//! 
//! The `key` is always stored as a variable sequence of bytes provided by the client.  If you go to the address of the key you should find a length (u16/u32) followed by a sequence of bytes that represents the key.
//! 
//! 
//! ### List Item Pointer
//! 
//! Used by items in a list object.  Contains the following:
//! ```text
//! | address of data | next list item pointer address | item index |
//! |   u16/u32       |          u16/u32               |    u16     |
//! ```
//! 
//! Unlike tables and maps, the order of the list items point to eachother should be kept so that the index is the correct sequence.
//! 
//! You can have gaps in the sequence, but the index should always be in order.  So if you have 3 item pointers with indexes 2, 8 and 20 they should point to each other in this order: 2 -> 8 -> 20.  This doesn't mean they have to be in order in the buffer, they just have to point to eachother in order.
//! 
//! There should be list item pointers only for indexes that have data in the list.
//! 
//! The last list item pointer in a list should have a zero in the next item address for no further list items.
//! 
//! 
//! ## Data
//! 
//! Data is stored in a specific format based on the data type in the schema.  The schema should determine how bytes at a sepcific address are treated.
//! 
//! When a pointer's address "points" to a location in the buffer, you should be able to parse the bytes at the designated location following the rules for the given data type below.
//! 
//! Most data types have a known size ahead of time, some don't, and some have a size dependent on the schema.
//! 
//! 
//! ### Table (Collection)
//! 
//! The table data type stores one or more vtables for the column values.  Each vtable contains:
//! - 1 leading byte that tells you how many columns are in the vtable
//! - 1 or more address (u32/u16) pointers for the table column values
//! - a trailing address(u32/u16) of the next vtable (should be zero if no more vtables)
//! 
//! The column indexes should accumulate across the vtables, and there should be at least one vtable entry for each column.
//! 
//! For example, if you have 4 columns and 2 vtables, the indexes could be arranged like this:
//! | vtable 1 | vtable 2 |
//! |  0, 1, 2 |  3, 4    |
//! 
//! Vtables can contain as few as one column entry or as many as the total columns in the schema (up to 255).
//! 
//! Typically you won't have more than 1 vtable unless the schema has been modified with additional columns.
//! 
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "table",
//!     "columns": [
//!         ["age",  {"type": "u8"}]
//!     ]
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&["age"], 20u8)?;
//!
//! assert_eq!(vec![0, 2, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 20], new_buffer.close());
//! 
//! // [    0, 2, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0,    20]
//! // [root ptr,                        vtable,  data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### List (Collection)
//! 
//! The list type stores two addresses (u16/u32), one to the first `ListItem` pointer (head) and one to the last `ListItem` pointer (tail).
//! 
//! If there is only one list item pointer in the list, the head and tail addresses should be identical.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "list",
//!     "of": {"type": "u8"}
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&["4"], 20u8)?;
//! assert_eq!(vec![0, 2, 0, 6, 0, 6, 0, 11, 0, 0, 4, 20], new_buffer.close());
//! 
//! // [    0, 2,  0, 6, 0, 6,   0, 11, 0, 0, 4,    20]
//! // [root ptr,  head, tail,    list item ptr,  data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### Map (Collection)
//! 
//! The map type stores a single address (u16/u32) to the first `MapItem` pointer for this map followed by a `u16` with the total number of values in the map.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "map",
//!     "value": {"type": "u8"}
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&["age"], 20u8)?;
//! assert_eq!(vec![0, 2, 0, 12, 0, 0, 0, 8, 3, 97, 103, 101, 20], new_buffer.close());
//! 
//! // [    0, 2,  0, 12, 0, 0, 0, 8,    3, 97, 103, 101,     20]
//! // [root ptr,       map item ptr,        a,   g,   e,   data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### Tuple (Collection)
//! 
//! The tuple will have as many addresses (u16/u32) as there are items in the schema.  For example, if there are 5 items in the schema there should be 5 addresses in the tuple.
//! 
//! So if a tuple is 20 items long in the schema, it should always ocuppy at least 40 bytes (u16) or 80 bytes (u32).
//! 
//! Each "address" should be treated like a standard pointer to a value in the tuple.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "tuple",
//!    "values": [
//!        {"type": "u8"},
//!        {"type": "string"}
//!    ]
//! }"#)?;
//! 
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&["0"], 20u8)?;
//! new_buffer.set(&["1"], "hello")?;
//! assert_eq!(vec![0, 2, 0, 12, 0, 13, 0, 0, 0, 0, 0, 0, 20, 0, 5, 104, 101, 108, 108, 111], new_buffer.close());
//! 
//! // [    0, 2, 0, 12, 0, 13, 0, 0, 0, 0, 0, 0,  20, 0, 5, 104, 101, 108, 108, 111]
//! // [root ptr,                         vtable,  u8,         h,   e,   l,   l,   o]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### int8, int16, int32, int64 (Scalar)
//! 
//! Signed integers should be converted to unsigned values, then saved in big endian format.
//! 
//! The size of the integer should determine how many bytes are used.  For example, i8 is 1 byte, i16 is 2 bytes, etc.
//! 
//! For example, an i8 of value -20 should be converted to 108, then saved as 108.
//! 
//! When it's requested by the client, it should be converted back to signed before being passed to the client.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "i32"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], -2023830i32)?;
//! assert_eq!(vec![0, 2, 127, 225, 30, 106], new_buffer.close());
//! 
//! // [    0, 2, 127, 225, 30, 106]
//! // [root ptr,              data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### uint8, uint16, uint32, uint64 (Scalar)
//! 
//! Unsigned integers should be converted to big endian format, then saved to the buffer.
//! 
//! The size of the integer should determine how many bytes are used.  For example, u8 is 1 byte, u16 is 2 bytes, etc.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "u32"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], 28378u32)?;
//! assert_eq!(vec![0, 2, 0, 0, 110, 218], new_buffer.close());
//! 
//! // [    0, 2, 0, 0, 110, 218]
//! // [root ptr,           data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### float, double (Scalar)
//! 
//! Floating point vales should be converted to big endian format, then saved to the buffer.
//! 
//! The size of the floating point value should determine how many bytes are used.  `float` is `f32` (4 bytes) and `double` is `f64` (8 bytes)
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "f32"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], 2.389988f32)?;
//! assert_eq!(vec![0, 2, 64, 24, 245, 144], new_buffer.close());
//! 
//! // [    0, 2, 64, 24, 245, 144]
//! // [root ptr,             data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### option (Scalar)
//! 
//! Option values are stored as a single `u8` value.  The value should represent the zero based location in the choice set.
//! 
//! For example if the schema has `choices: ["red", "blue", "yellow"]` and the user selects `yellow`, this value should `2`.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::option::NP_Enum;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "option",
//!    "choices": ["blue", "orange", "red"]
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Enum::new("red"))?;
//! assert_eq!(vec![0, 2, 2], new_buffer.close());
//! 
//! // [    0, 2,      2]
//! // [root ptr,   data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### bool (Scalar)
//! 
//! A single `u8` byte.  `1` for `true`, `0` for `false`.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "bool"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], true)?;
//! assert_eq!(vec![0, 2, 1], new_buffer.close());
//! 
//! // [    0, 2,      1]
//! // [root ptr,   data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### decimal (Scalar)
//! 
//! Stored the same as an i64 value (including converting to unsigned format described above).
//! 
//! The `i64` number should be devided by `10 ^ exp` to get the true value.  The `exp` value is provided in the schema.
//! 
//! For example, if you pull a `293` i64 value from the buffer and the `exp` value in the schema is `2`, the value is actually `293 / 100` or 2.93.
//! 
//! You should avoid converting the number to floating point values except for display purposes.  Study the source code for the `NP_Dec` type to see how to preserve the internal i64 value correctly.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::dec::NP_Dec;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "decimal",
//!    "exp": 2
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Dec::new(200, 0))?;
//! assert_eq!(vec![0, 2, 128, 0, 0, 0, 0, 0, 78, 32], new_buffer.close());
//! 
//! // [    0, 2, 128, 0, 0, 0, 0, 0, 78, 32]
//! // [root ptr,                       data]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### geo4, geo8, geo16 (Scalar)
//! Each geo size uses two signed integers right next to eachother in the buffer.  i16/i16 for geo4, i32/i32 for geo8 and i64/i64 for geo16
//! 
//! The two signed integers are converted to unsigned values before being saved into big endian format. 
//! 
//! Depending on the size, the floating point value of each geographic coordinate is multiplied by a specific value before being saved as an integer.
//! 
//! | Size | Bytes      | Factor     |
//! |------|------------|------------|
//! | 4    | i16 \| i16 | 100        |
//! | 8    | i32 \| i32 | 10000000   |
//! | 16   | i64 \| i64 | 1000000000 |
//! 
//! For example, if a user provides these coordinates: 41.303921, -81.901693
//! 
//! To save into buffer:<br/>
//! <br/>
//! geo4: <br/>
//! 1 - Multiply by 100: (4130.3921, -8190.1693) <br/>
//! 2 - Make i16 (4130, -8190)<br/>
//! 3 - Save/convert as unsigned in big endian format<br/>
//! <br/>
//! geo8: <br/>
//! 1 - Multiply by 10000000: (413039210, -819016930)<br/>
//! 2 - Make i32 (413039210, -819016930)<br/>
//! 3 - Save/convert as unsigned in big endian format<br/>
//! <br/>
//! geo16: ....
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::geo::NP_Geo;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "geo8"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Geo::new(8, 41.303921, -81.901693))?;
//! assert_eq!(vec![0, 2, 152, 158, 122, 106, 79, 46, 203, 30], new_buffer.close());
//! 
//! // [    0, 2, 152, 158, 122, 106, 79, 46, 203, 30]
//! // [root ptr,           latitude,       longitude]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### ulid, uuid (Scalar)
//! 
//! Saved as 16 bytes following the respective formats for each data type.
//! 
//! ULIDs store the date in the first 6 bytes, then the random bytes in the last 10.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::uuid::NP_UUID;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "uuid"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! let uuid = NP_UUID::generate(32);
//! new_buffer.set(&[], &uuid)?;
//! assert_eq!(vec![0, 2, 202, 230, 170, 176, 127, 103, 66, 13, 89, 65, 221, 4, 153, 160, 117, 252], new_buffer.close());
//! 
//! // [    0, 2, 202, 230, 170, 176, 127, 103, 66, 13, 89, 65, 221, 4, 153, 160, 117, 252]
//! // [root ptr,                              UUID                                       ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### bytes, string (Scalar)
//! 
//! If there is a `size` property in the schema, store the provided data and pad the remainder of the space with zeros.
//! 
//! If the provided data is too large, truncate it.
//! 
//! For example, if the user provideds a single byte `[22]` and the size is `3`, this should be in the buffer:
//! ```text
//! [22, 0, 0]
//! ```
//! 
//! If there is no fixed `size` in the schema, store a size (u16/u32) followed by the actual data.
//! 
//! If it's a string, the data should be utf-8 encoded when it's saved into the buffer and utf-8 decoded when it's retrieved.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], "hello, world!")?;
//! assert_eq!(vec![0, 2, 0, 13, 104, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33], new_buffer.close());
//! 
//! // [    0, 2,   0, 13, 104, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33]
//! // [root ptr,  length,   h,   e,   l,   l,   o,  ,,   ,   w,   o,   r,   l,   d,  !]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### date (Scalar)
//! This is stored the same as a uint64 value, should be unix timestamp in milliseconds.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::date::NP_Date;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "date"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Date::new(1598490738507))?;
//! assert_eq!(vec![0, 2, 0, 0, 1, 116, 45, 120, 255, 75], new_buffer.close());
//! 
//! // [    0, 2, 0, 0, 1, 116, 45, 120, 255, 75]
//! // [root ptr,           timestamp           ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! # NoProto Schema Format Documentation
//! 
//! NoProto JSON schemas are compiled into a byte array as part of the parsing process.
//! 
//! The compiled byte array is a significantly more compact and efficient way to store the schema.  It also takes almost no time to parse a byte schema, where parsing a JSON schema can be a comparitively expensive operation.
//! 
//! You can use the runtime to parse JSON schemas into byte array schemas at any time, and the JSON/byte array schemas can be used interchangebly.
//! 
//! The byte array schema store default values and all other supported schema properties.
//! 
//! Schema data is stored in a recursive format, each nested schema contains at least one byte that describes the data type.  The single data type byte is usually but not always followed by schema data specific to that data type.  The document below describes all of the data types and their specifics.
//! 
//! 
//! ### int8, int16, int32, int64, uint8, uint16, uint32, uint64, float, double (Scalar)
//! 
//! Integer values store the data type followed by wether there is a default value or not, followed optionally by the default value
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "i32",
//!    "default": 56
//! }"#)?;
//!
//! assert_eq!(vec![6, 1, 0, 0, 0, 56], factory.compile_schema());
//! 
//! // [       6,           1,      0, 0, 0, 56]
//! // [i32 type, has default,    default value]
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "i32"
//! }"#)?;
//!
//! assert_eq!(vec![6, 0], factory.compile_schema());
//! 
//! // [       6,           0]
//! // [i32 type,  no default]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### option (Scalar)
//! 
//! Option types will store the list of options and the index of the default value, if there is one.
//! 
//! The second byte is `0` if there is no default, otherwise it contains the default index + 1.
//! 
//! The third byte contains a `u8` that is the number of options available.
//! 
//! The remaining bytes go on a loop for each option, with each loop containing 1 u8 byte at the begining describing the length of the string option, followed by the string value itself.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::option::NP_Enum;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "option",
//!    "choices": ["blue", "orange", "red"],
//!    "default": "red"
//! }"#)?;
//!
//! assert_eq!(vec![20, 3, 3, 4, 98, 108, 117, 101, 6, 111, 114, 97, 110, 103, 101, 3, 114, 101, 100], factory.compile_schema());
//! 
//! // [       20,                        3,            3, 4, 98, 108, 117, 101, 6, 111, 114, 97, 110, 103, 101, 3, 114, 101, 100]
//! // [data type, 1 based index of default, # of options,     b,   l,   u,   e,      o,   r,  a,   n,   g,   e,      r,   e,   d]  
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "option",
//!    "choices": ["blue", "orange", "red"]
//! }"#)?;
//!
//! assert_eq!(vec![20, 0, 3, 4, 98, 108, 117, 101, 6, 111, 114, 97, 110, 103, 101, 3, 114, 101, 100], factory.compile_schema());
//! 
//! // [       20,          0,             3, 4, 98, 108, 117, 101, 6, 111, 114, 97, 110, 103, 101, 3, 114, 101, 100]
//! // [data type, no default,  # of options,     b,   l,   u,   e,      o,   r,  a,   n,   g,   e,      r,   e,   d]  
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! ### bool (Scalar)
//! 
//! The second byte of a bool schema is used to store the default value.
//! 
//! If there is no default value, the second byte is 0.<br/>
//! If the default is true, the second byte is 1.<br/>
//! If the default is false, the second byte is 2.<br/>
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::option::NP_Enum;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "bool",
//!    "default": true
//! }"#)?;
//!
//! assert_eq!(vec![15, 1], factory.compile_schema());
//! 
//! // [       15,               1]
//! // [data type, default is true]  
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "bool",
//!    "default": false
//! }"#)?;
//!
//! assert_eq!(vec![15, 2], factory.compile_schema());
//! 
//! // [       15,               2]
//! // [data type, default is true]  
//! 
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "bool"
//! }"#)?;
//!
//! assert_eq!(vec![15, 0], factory.compile_schema());
//! 
//! // [       15,          0]
//! // [data type, no default]  
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### decimal (Scalar)
//! 
//! Decimal stores the expontent in the second byte.
//! 
//! The third byte is 0 if there is no default value, otherwise it is 1.
//! 
//! If there is a default value, multiply the default value by (10^exp) and convert it into an i64, then save it in the bytes following the default flag byte.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::dec::NP_Dec;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "decimal",
//!    "exp": 2
//! }"#)?;
//!
//! assert_eq!(vec![14, 2, 0], factory.compile_schema());
//! 
//! // [       14,         2,                0]
//! // [data type, expontent, no default value]
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "decimal",
//!    "exp": 2,
//!    "default": 521.32
//! }"#)?;
//!
//! assert_eq!(vec![14, 2, 1, 0, 0, 0, 0, 0, 0, 203, 164], factory.compile_schema());
//! 
//! // [       14,         2,                 1, 0, 0, 0, 0, 0, 0, 203, 164]
//! // [data type, expontent, has default value,              default value]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### geo4, geo8, geo16 (Scalar)
//! 
//! Geo stores the size of the data type in the second byte.
//! The third byte is 0 if there is no default, and 1 if there is a default.
//! The remaining bytes are the default value (if there is one) parsed in the specific size designated in the second byte.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::geo::NP_Geo;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "geo8"
//! }"#)?;
//!
//! assert_eq!(vec![16, 8, 0], factory.compile_schema());
//! 
//! // [       16,                 8,                0]
//! // [data type, geo size (4/8/16), no default value]
//! 
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "geo8",
//!    "default": {"lat": 29.2, "lng": -19.2}
//! }"#)?;
//!
//! assert_eq!(vec![16, 8, 1, 145, 103, 145, 0, 116, 142, 80, 0], factory.compile_schema());
//! 
//! // [       16,                 8,                 1, 145, 103, 145, 0, 116, 142, 80, 0]
//! // [data type, geo size (4/8/16), has default value,             geo8 value (lat/lng) ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! ### ulid, uuid (Scalar)
//! 
//! UUID and ULID do not have default options, so this data type is very simple.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::uuid::NP_UUID;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "uuid"
//! }"#)?;
//!
//! assert_eq!(vec![17], factory.compile_schema());
//! 
//! // [       17]
//! // [data type]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### bytes, string (Scalar)
//! 
//! The second and third bytes are a u16 of the fixed size.  If there is no fixed size, these two bytes are zero.
//! 
//! Thhe length of the default value follows as a u16, if there is no default value the u16 is zero.  If there is a default value, it follows the length bytes.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string"
//! }"#)?;
//!
//! assert_eq!(vec![2, 0, 0, 0, 0], factory.compile_schema());
//! 
//! // [        2,             0, 0,                 0, 0]
//! // [data type, fixed size (u16),  default size (u16) ]
//!
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string",
//!    "size": 20
//! }"#)?;
//!
//! assert_eq!(vec![2, 0, 20, 0, 0], factory.compile_schema());
//! 
//! // [        2,             0, 20,                 0, 0]
//! // [data type,  fixed size (u16),  default size (u16) ]
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "string",
//!    "size": 20,
//!    "default": "hello"
//! }"#)?;
//!
//! assert_eq!(vec![2, 0, 20, 0, 6, 104, 101, 108, 108, 111], factory.compile_schema());
//! 
//! // [        2,             0, 20,                0, 6, 104, 101, 108, 108, 111]
//! // [data type,  fixed size (u16),  default size (u16),   h,   e,   l,   l,   o]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### date (Scalar)
//! 
//! The second byte is a 1 if there is a default value, 0 otherwise.
//! 
//! If there is a default value it follows the second byte.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::date::NP_Date;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "date"
//! }"#)?;
//!
//! assert_eq!(vec![19, 0], factory.compile_schema());
//! 
//! // [       19,             0]
//! // [data type, default flag ]
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "date",
//!    "default": 1604862252
//! }"#)?;
//!
//! assert_eq!(vec![19, 1, 0, 0, 0, 0, 95, 168, 65, 44], factory.compile_schema());
//! 
//! // [       19,            1, 0, 0, 0, 0, 95, 168, 65, 44]
//! // [data type, default flag,        default value       ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ## Collection Schemas
//! 
//! Collection based schemas nest schemas in a way that allows any type to be the child of any collection, including other collections.
//! 
//! ### Table (collection)
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "table",
//!     "columns": [
//!         ["age",  {"type": "u8"}],
//!         ["name", {"type": "string"}]
//!     ]
//! }"#)?;
//!
//!
//! assert_eq!(vec![21, 2, 3, 97, 103, 101, 0, 2, 8, 0, 4, 110, 97, 109, 101, 0, 5, 2, 0, 0, 0, 0], factory.compile_schema());
//! 
//! // [       21,            2, 3, 97, 103, 101,                     0, 2,           8, 0, 4, 110, 97, 109, 101,                      0, 5,   2, 0, 0, 0, 0]
//! // [data type, # of columns,     a,   g,   e, column schema size (u16),  column schema,      n,  a,   m,   e,  column schema size (u16),  column schema ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### List (Collection)
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "list",
//!     "of": {"type": "u8"}
//! }"#)?;
//!
//! assert_eq!(vec![23, 8, 0], factory.compile_schema());
//! 
//! // [       23,        8, 0]
//! // [data type, "of" schema]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### Map (Collection)
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!     "type": "map",
//!     "value": {"type": "u8"}
//! }"#)?;
//!
//! assert_eq!(vec![22, 8, 0], factory.compile_schema());
//! 
//! // [       22,         8, 0]
//! // [data type, value schema]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! ### Tuple (Collection)
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "tuple",
//!    "values": [
//!        {"type": "u8"},
//!        {"type": "string"}
//!    ]
//! }"#)?;
//!
//! assert_eq!(vec![24, 0, 2, 0, 2, 8, 0, 0, 5, 2, 0, 0, 0, 0], factory.compile_schema());
//! 
//! // [       24,      0,           2,               0, 2,   8, 0,              0, 5,  2, 0, 0, 0, 0]
//! // [data type, sorted, length (u8),  schema size (u16), schema, schema size (u16),    schema     ]
//!
//! # Ok::<(), NP_Error>(()) 
//! ```