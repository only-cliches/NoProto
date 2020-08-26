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
//! There are 2 different address sizes, u16 and u32.  All numbers (including addresses) are always stored in big endian format and addresses are always zero based from the beginning of the buffer.  In other words, address `23` always means 23 bytes from the beginning of the buffer.
//! 
//! | Pointer Kind | u16 size (bytes) | u32 size (bytes) |
//! |--------------|------------------|------------------|
//! | Standard     | 2                | 4                |
//! | Map Item     | 6                | 12               |
//! | Table Item   | 5                | 9                |
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
//! For example, here is a buffer with u16 size that contains the string `hello`, it's schema is just `{type: "string"}`.
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
//! **Standard Pointer**<br/>
//! This is used for any scalar or collection data types.  The standard pointer is just a single u16 or u32.
//! 
//! **Map Item Pointer**<br/>
//! 
//! Used by items in a map object.  Contains the following:
//! ```text
//! | address of data | next map item pointer address | address of bytes for this key |
//! |     u16/u32     |            u16/u32            |          u16/u32              |
//! ```
//! 
//! Map collections represent a linked list of these pointers.  There should only be map item pointers for items in the map that have data.
//! 
//! The last map item pointer in a map should have a zero in the next item address for no further map items.
//! 
//! The `key` is always stored as a variable sequence of bytes provided by the client.  If you go to the address of the key you should find a length (u16/u32) followed by a sequence of bytes that represents the key.
//! 
//! **Table Item Pointer**<br/>
//! 
//! Used by items in a table object.  Contains the following:
//! ```text
//! | address of data | next table item pointer address | column index |
//! |   u16/u32       |          u16/u32                |    u8        |
//! ```
//! 
//! Tables are a linked list of these pointers.  There should only be table item pointers for columns that have data.
//! 
//! The last table item pointer should have a zero in the next item address for no further table items.
//! 
//! **List Item Pointer**<br/>
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
//! Data is stored in a specific format based on the data type in the schema.  The schema should determine how bytes at a sepcific are treated.
//! 
//! When a pointer's address "points" to a location in the buffer, you should be able to parse the bytes at the designated location following the rules for the given data type below.
//! 
//! Most data types have a known size ahead of time, some don't, and some have a size dependent on the schema.
//! 
//! 
//! **Table (Collection)**<br/>
//! 
//! The table type stores a single address (u16/u32) to the first `TableItem` pointer for this table.
//! 
//! **List (Collection)**<br/>
//! 
//! The list type stores two addresses (u16/u32), one to the first `ListItem` pointer (head) and one to the last `ListItem` pointer (tail).
//! 
//! If there is only one pointer in the list, the head and tail addresses should be identical.
//! 
//! **Map (Collection)**<br/>
//! 
//! The map type stores a single address (u16/u32) to the first `MapItem` pointer for this map.
//! 
//! **Tuple (Collection)**<br/>
//! 
//! The tuple will have as many addresses (u16/u32) as there are items in the schema.  For example, if there are 5 items in the schema there should be 5 addresses in the tuple.
//! 
//! So if a tuple is 20 items long in the schema, it should always ocuppy at least 40 bytes (u16) or 80 bytes (u32).
//! 
//! Each "address" should be treated like a standard pointer to a value in the tuple.
//! 
//! **int8, int16, int32, int64**<br/>
//! 
//! Signed integers should be converted to unsigned values, then saved in big endian format.
//! 
//! The size of the integer should determine how many bytes are used.  For example, i8 is 1 byte, i16 is 2 bytes, etc.
//! 
//! For example, an i8 of value -20 should be converted to 108, then saved as 108.
//! 
//! When it's requested by the user, it should be converted back to signed before being passed to the user.
//! 
//! **uint8, uint16, uint32, uint64**<br/>
//! 
//! Unsigned integers should be converted to big endian format, then saved to the buffer.
//! 
//! The size of the integer should determine how many bytes are used.  For example, u8 is 1 byte, u16 is 2 bytes, etc.
//! 
//! **float, double**<br/>
//! 
//! Floating point vales should be converted to big endian format, then saved to the buffer.
//! 
//! The size of the floating point value should determine how many bytes are used.  `float` is `f32` (4 bytes) and `double` is `f64` (8 bytes)
//! 
//! **option**<br/>
//! 
//! Option values are stored as a single `u8` value.  The value should represent the zero based location in the choice set.
//! 
//! For example if the schema has `choices: ["red", "blue", "yellow"]` and the user selects `yellow`, this value should `2`.
//! 
//! **bool**<br/>
//! 
//! A single `u8` byte.  `1` for `true`, `0` for `false`.
//! 
//! **decimal**<br/>
//! 
//! Stored the same as an i64 value (including converting to unsigned format described above).
//! 
//! The `i64` number should be devided by `10 ^ exp` to get the true value.  The `exp` value is provided in the schema.
//! 
//! For example, if you pull a `293` i64 value from the buffer and the `exp` value in the schema is `2`, the value is actually `293 / 100` or 2.93.
//! 
//! You should avoid converting the number to floating point values except for display purposes.  Study the source code for the `NP_Dec` type to see how to preserve the internal i64 value correctly.
//! 
//! **geo4, geo8, geo16**<br/>
//! Each geo size uses two signed integers right next to eachother in the buffer.  i16/16 for geo4, i32/i32 for geo8 and i64/i64 for geo16
//! 
//! The two signed integers are converted to unsigned values before being saved into big endian format. 
//! 
//! Depending on the size, the floating point value of each geographic coordinate is devided by a specific value before being saved as an integer.
//! 
//! | Size | Bytes      | Factor     |
//! |------|------------|------------|
//! | 4    | i16 \| i16 | 100        |
//! | 8    | i32 \| i32 | 10000000   |
//! | 16   | i64 \| i64 | 1000000000 |
//! 
//! For example, if a user provides these coordinates: 41.303921, -81.901693
//! 
//! To save into buffer:
//! =======<br/>
//! geo4: Multiply by 100 (4130.3921, -8190.1693) -> make i16 (4130, -8190) -> save/convert as unsigned in big endian format<br/>
//! =======<br/>
//! geo8: Multiply by 10000000 (413039210, -819016930) -> make i32 (413039210, -819016930) -> save/convert as unsigned in big endian format<br/>
//! ======<br/>
//! geo16: ....
//! 
//! **ulid, uuid**<br/>
//! 
//! Saved as 16 bytes following the respective formats for each data type.
//! 
//! ULIDs store the date in the first 6 bytes, then the random bytes in the last 10.
//! 
//! ***bytes, string**<br/>
//! 
//! If there is a `size` property in the schema, store the provided data and zero out the rest of the space.
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
//! **date**<br/>
//! This is stored the same as a uint64 value.