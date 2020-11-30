//! All values in buffers are accessed and modified through pointers
//! 
//! NP_Ptr are the primary abstraction to read, update or delete values in a buffer.
//! Pointers should *never* be created directly, instead the various methods provided by the library to access
//! the internals of the buffer should be used.
//! 
//! Once you have a pointer you can read it's contents if it's a scalar value with `.get()` or convert it to a collection with `.deref()`.
//! When you attempt to read, update, or convert a pointer the schema is checked for that pointer location.  If the schema conflicts with the operation you're attempting it will fail.
//! As a result, you should be careful to make sure your reads and updates to the buffer line up with the schema you provided.
//! 
//! 

/// Any type
pub mod any;
pub mod string;
pub mod bytes;
pub mod numbers;
pub mod bool;
pub mod geo;
pub mod dec;
pub mod ulid;
pub mod uuid;
pub mod option;
pub mod date;

use std::hint::unreachable_unchecked;

use crate::{pointer::dec::NP_Dec, schema::NP_Schema_Addr, memory::NP_Size};
use crate::NP_Parsed_Schema;
use crate::{json_flex::NP_JSON};
use crate::memory::{NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}};

use alloc::{string::String, vec::Vec, borrow::ToOwned};
use bytes::NP_Bytes;

use self::{date::NP_Date, geo::NP_Geo, option::NP_Enum, string::NP_String, ulid::{NP_ULID, _NP_ULID}, uuid::{NP_UUID, _NP_UUID}};


#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NP_Cursor_Value {
    None,
    Standard { value_addr: usize },   // u32(4 bytes [4]), u16(2 bytes [2])

    // collection items
    MapItem { value_addr: usize, next: usize, key_addr: usize },  // [addr | next | key] u32(12 bytes  [4, 4, 4]),  u16(6 bytes [2, 2, 2]), 
    TableItem { value_addr: usize, next: usize, index: usize },   // [addr | next | i: u8]  u32(9  bytes  [4, 4, 1]),  u16(5 bytes [2, 2, 1]),   
    ListItem { value_addr: usize, next: usize, index: usize },   // [addr | next | i: u16] u32(10 bytes  [4, 4, 2]),  u16(6 bytes [2, 2, 2]),
    TupleItem  { value_addr: usize, index: usize}                // [addr]u32(4 bytes  [4]),  u16(2 bytes [2])           
}

impl<'kind> NP_Cursor_Value {
    /// Get the value address of this cursor
    pub fn get_value_address(&self) -> usize {
        match self {
            NP_Cursor_Value::None => 0,
            NP_Cursor_Value::Standard { value_addr } => { *value_addr },
            NP_Cursor_Value::TableItem { value_addr, .. } => { *value_addr },
            NP_Cursor_Value::ListItem { value_addr, .. } =>  { *value_addr },
            NP_Cursor_Value::TupleItem { value_addr, .. } => { *value_addr },
            NP_Cursor_Value::MapItem { value_addr, .. } => { *value_addr }
        }
    }
    /// Update the value address (doesn't touch the buffer)
    pub fn update_value_address(&self, new_value: usize) -> Self {
        match self {
            NP_Cursor_Value::None => NP_Cursor_Value::None,
            NP_Cursor_Value::Standard { value_addr: _ } => { NP_Cursor_Value::Standard { value_addr: new_value } },
            NP_Cursor_Value::TableItem { value_addr: _, next , index  } => { NP_Cursor_Value::TableItem { value_addr: new_value, next: *next, index: *index } },
            NP_Cursor_Value::ListItem { value_addr: _, next , index } =>  { NP_Cursor_Value::ListItem { value_addr: new_value, next: *next, index: *index } },
            NP_Cursor_Value::TupleItem { value_addr: _, index } => { NP_Cursor_Value::TupleItem { value_addr: new_value, index: *index } },
            NP_Cursor_Value::MapItem { value_addr: _, next , key_addr  } => { NP_Cursor_Value::MapItem { value_addr: new_value, next: *next, key_addr: *key_addr} }
        }
    }
}

impl<'kind> Default for NP_Cursor_Value {
    fn default() -> Self { NP_Cursor_Value::None }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NP_Cursor_Parent {
    None,
    List { head: usize, tail: usize, addr: usize, schema_addr: usize},
    Map { head: usize, length: usize, addr: usize, schema_addr: usize},
    Table { head: usize, addr: usize, schema_addr: usize},
    Tuple { addr: usize, schema_addr: usize}
}

impl Default for NP_Cursor_Parent {
    fn default() -> Self { NP_Cursor_Parent::None }
}

/// Cursor for pointer value in buffer
/// 
#[derive(Debug, Clone, Copy, Default)]
pub struct NP_Cursor {
    /// The location of this cursor in the buffer
    pub buff_addr: usize,
    /// The address of the schema for this cursor
    pub schema_addr: NP_Schema_Addr,
    /// the values of the buffer pointer
    pub value: NP_Cursor_Value,
    /// Information about the parent cursor
    pub parent: NP_Cursor_Parent,
    /// The previous cursor
    pub prev_cursor: Option<usize>,
    /// The next cursor 
    pub next_cursor: Option<usize>
}


impl<'cursor> NP_Cursor {

    /// Clone a cursor without the previous or next data
    pub fn clone_lite(&self) -> Self {
        Self {
            buff_addr: self.buff_addr,
            schema_addr: self.schema_addr,
            value: self.value,
            parent: self.parent,
            prev_cursor: None,
            next_cursor: None
        }
    }

    /// Create a new cursor and read details from buffer
    /// 
    pub fn new(buff_addr: usize, schema_addr: NP_Schema_Addr, memory: &NP_Memory<'cursor>, parent: NP_Cursor_Parent) -> Self {
    
        Self {
            buff_addr,
            schema_addr,
            value: if buff_addr > 0 {
                match parent {
                    NP_Cursor_Parent::None => NP_Cursor_Value::Standard { value_addr: memory.read_address(buff_addr) },
                    NP_Cursor_Parent::List { .. } => {
                        NP_Cursor_Value::ListItem { 
                            value_addr:  memory.read_address(buff_addr),
                            next:  memory.read_address_offset(buff_addr, 1),
                            index: if buff_addr == 0 { 0 } else { match &memory.size {
                                NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(buff_addr + 8).unwrap_or(&[0; 2])) as usize,
                                NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(buff_addr + 4).unwrap_or(&[0; 2])) as usize,
                                NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(buff_addr + 2).unwrap_or(0)]) as usize
                            }}
                        }
                    },
                    NP_Cursor_Parent::Table { schema_addr: t_schema_addr, .. } => {
                        let index = if buff_addr == 0 { 0 } else { match &memory.size {
                            NP_Size::U32 => memory.read_bytes()[buff_addr + 8] as usize,
                            NP_Size::U16 => memory.read_bytes()[buff_addr + 4] as usize,
                            NP_Size::U8 => memory.read_bytes()[buff_addr + 2] as usize
                        }};
                        NP_Cursor_Value::TableItem { 
                            value_addr:  memory.read_address(buff_addr),
                            next:  memory.read_address_offset(buff_addr, 1),
                            index: index
                        }
                    },
                    NP_Cursor_Parent::Map { .. } => {
                        let key_addr = memory.read_address_offset(buff_addr, 2);
                        NP_Cursor_Value::MapItem { 
                            value_addr: memory.read_address(buff_addr),
                            next: memory.read_address_offset(buff_addr,  1),
                            key_addr: key_addr
                        }
                    },
                    NP_Cursor_Parent::Tuple { addr, .. } => {
                        let addr_size = memory.addr_size_bytes();
                        let index = (buff_addr - addr) / addr_size;
                        NP_Cursor_Value::TupleItem { value_addr: memory.read_address(buff_addr), index: index}
                    }
                }
            } else {
                match parent {
                    NP_Cursor_Parent::None => NP_Cursor_Value::Standard { value_addr: 0 },
                    NP_Cursor_Parent::List { .. } => {
                        NP_Cursor_Value::ListItem { 
                            value_addr:  0,
                            next:  0,
                            index: 0
                        }
                    },
                    NP_Cursor_Parent::Table { .. } => {
                        NP_Cursor_Value::TableItem { 
                            value_addr:  0,
                            next:  0,
                            index: 0
                        }
                    },
                    NP_Cursor_Parent::Map { .. } => {
                        NP_Cursor_Value::MapItem { 
                            value_addr: 0,
                            next: 0,
                            key_addr: 0
                        }
                    },
                    NP_Cursor_Parent::Tuple { .. } => {
                        NP_Cursor_Value::TupleItem { 
                            value_addr: 0, 
                            index: 0
                        }
                    }
                }
            },
            parent: parent,
            prev_cursor: None,
            next_cursor: None
        }
    }

    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    /// 
    pub fn json_encode(cursor: &NP_Cursor, memory: &NP_Memory<'cursor>) -> NP_JSON {

        match memory.schema[cursor.schema_addr].get_type_key() {
            NP_TypeKeys::None           => { NP_JSON::Null },
            NP_TypeKeys::Any            => { NP_JSON::Null },
            NP_TypeKeys::UTF8String     => {    NP_String::to_json(cursor, memory) },
            NP_TypeKeys::Bytes          => {  NP_Bytes::to_json(cursor, memory) },
            NP_TypeKeys::Int8           => {        i8::to_json(cursor, memory) },
            NP_TypeKeys::Int16          => {       i16::to_json(cursor, memory) },
            NP_TypeKeys::Int32          => {       i32::to_json(cursor, memory) },
            NP_TypeKeys::Int64          => {       i64::to_json(cursor, memory) },
            NP_TypeKeys::Uint8          => {        u8::to_json(cursor, memory) },
            NP_TypeKeys::Uint16         => {       u16::to_json(cursor, memory) },
            NP_TypeKeys::Uint32         => {       u32::to_json(cursor, memory) },
            NP_TypeKeys::Uint64         => {       u64::to_json(cursor, memory) },
            NP_TypeKeys::Float          => {       f32::to_json(cursor, memory) },
            NP_TypeKeys::Double         => {       f64::to_json(cursor, memory) },
            NP_TypeKeys::Decimal        => {    NP_Dec::to_json(cursor, memory) },
            NP_TypeKeys::Boolean        => {      bool::to_json(cursor, memory) },
            NP_TypeKeys::Geo            => {    NP_Geo::to_json(cursor, memory) },
            NP_TypeKeys::Uuid           => {   _NP_UUID::to_json(cursor, memory) },
            NP_TypeKeys::Ulid           => {   _NP_ULID::to_json(cursor, memory) },
            NP_TypeKeys::Date           => {   NP_Date::to_json(cursor, memory) },
            NP_TypeKeys::Enum           => { NP_Enum::to_json(cursor, memory) },
            NP_TypeKeys::Table          => {  NP_Table::to_json(cursor, memory) },
            NP_TypeKeys::Map            => {    NP_Map::to_json(cursor, memory) },
            NP_TypeKeys::List           => {   NP_List::to_json(cursor, memory) },
            NP_TypeKeys::Tuple          => {  NP_Tuple::to_json(cursor, memory) }
        }

    }

    /// Compact from old cursor and memory into new cursor and memory
    /// 
    pub fn compact(from_cursor: &NP_Cursor, from_memory: &NP_Memory<'cursor>, to_cursor: NP_Cursor, to_memory: &NP_Memory<'cursor>) -> Result<NP_Cursor, NP_Error> {

        match from_memory.schema[from_cursor.schema_addr].get_type_key() {
            NP_TypeKeys::Any           => { Ok(to_cursor) }
            NP_TypeKeys::UTF8String    => {       NP_String::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Bytes         => {  NP_Bytes::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int8          => {        i8::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int16         => {       i16::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int32         => {       i32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int64         => {       i64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint8         => {        u8::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint16        => {       u16::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint32        => {       u32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint64        => {       u64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Float         => {       f32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Double        => {       f64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Decimal       => {    NP_Dec::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Boolean       => {      bool::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Geo           => {    NP_Geo::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uuid          => {   _NP_UUID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Ulid          => {   _NP_ULID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Date          => {   NP_Date::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Enum          => { NP_Enum::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Table         => {  NP_Table::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Map           => {    NP_Map::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::List          => {   NP_List::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Tuple         => {  NP_Tuple::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
            _ => { panic!() }
        }
    }

    /// Set default for this value.  Not related to the schema default, this is the default value for this data type
    /// 
    pub fn set_default(cursor: NP_Cursor, memory: &NP_Memory<'cursor>) -> Result<(), NP_Error> {

        match memory.schema[cursor.schema_addr].get_type_key() {
            NP_TypeKeys::None        => { panic!() },
            NP_TypeKeys::Any         => { panic!() },
            NP_TypeKeys::Table       => { panic!() },
            NP_TypeKeys::Map         => { panic!() },
            NP_TypeKeys::List        => { panic!() },
            NP_TypeKeys::Tuple       => { panic!() },
            NP_TypeKeys::UTF8String  => {  NP_String::set_value(cursor, memory, &String::default())?; },
            NP_TypeKeys::Bytes       => {   NP_Bytes::set_value(cursor, memory, &NP_Bytes::default())?; },
            NP_TypeKeys::Int8        => {         i8::set_value(cursor, memory, i8::default())?; },
            NP_TypeKeys::Int16       => {        i16::set_value(cursor, memory, i16::default())?; },
            NP_TypeKeys::Int32       => {        i32::set_value(cursor, memory, i32::default())?; },
            NP_TypeKeys::Int64       => {        i64::set_value(cursor, memory, i64::default())?; },
            NP_TypeKeys::Uint8       => {         u8::set_value(cursor, memory, u8::default())?; },
            NP_TypeKeys::Uint16      => {        u16::set_value(cursor, memory, u16::default())?; },
            NP_TypeKeys::Uint32      => {        u32::set_value(cursor, memory, u32::default())?; },
            NP_TypeKeys::Uint64      => {        u64::set_value(cursor, memory, u64::default())?; },
            NP_TypeKeys::Float       => {        f32::set_value(cursor, memory, f32::default())?; },
            NP_TypeKeys::Double      => {        f64::set_value(cursor, memory, f64::default())?; },
            NP_TypeKeys::Decimal     => {     NP_Dec::set_value(cursor, memory, NP_Dec::default())?; },
            NP_TypeKeys::Boolean     => {       bool::set_value(cursor, memory, bool::default())?; },
            NP_TypeKeys::Geo         => {     NP_Geo::set_value(cursor, memory, NP_Geo::default())?; },
            NP_TypeKeys::Uuid        => {    _NP_UUID::set_value(cursor, memory, &NP_UUID::default())?; },
            NP_TypeKeys::Ulid        => {    _NP_ULID::set_value(cursor, memory, &NP_ULID::default())?; },
            NP_TypeKeys::Date        => {    NP_Date::set_value(cursor, memory, NP_Date::default())?; },
            NP_TypeKeys::Enum        => {  NP_Enum::set_value(cursor, memory, NP_Enum::default())?; }
        }

        Ok(())
    }

    /// Calculate the number of bytes used by this pointer and it's descendants.
    /// 
    pub fn calc_size(cursor: NP_Cursor, memory: &NP_Memory<'cursor>) -> Result<usize, NP_Error> {

        // no pointer, no size
        if cursor.buff_addr == 0 {
            return Ok(0);
        }

        // size of pointer
        let base_size = memory.ptr_size(&cursor);

        // pointer is in buffer but has no value set
        if cursor.value.get_value_address() == 0 { // no value, just base size
            return Ok(base_size);
        }

        // get the size of the value based on schema
        let type_size = match memory.schema[cursor.schema_addr].get_type_key() {
            NP_TypeKeys::None         => { Ok(0) },
            NP_TypeKeys::Any          => { Ok(0) },
            NP_TypeKeys::UTF8String   => {       NP_String::get_size(cursor, memory) },
            NP_TypeKeys::Bytes        => {  NP_Bytes::get_size(cursor, memory) },
            NP_TypeKeys::Int8         => {        i8::get_size(cursor, memory) },
            NP_TypeKeys::Int16        => {       i16::get_size(cursor, memory) },
            NP_TypeKeys::Int32        => {       i32::get_size(cursor, memory) },
            NP_TypeKeys::Int64        => {       i64::get_size(cursor, memory) },
            NP_TypeKeys::Uint8        => {        u8::get_size(cursor, memory) },
            NP_TypeKeys::Uint16       => {       u16::get_size(cursor, memory) },
            NP_TypeKeys::Uint32       => {       u32::get_size(cursor, memory) },
            NP_TypeKeys::Uint64       => {       u64::get_size(cursor, memory) },
            NP_TypeKeys::Float        => {       f32::get_size(cursor, memory) },
            NP_TypeKeys::Double       => {       f64::get_size(cursor, memory) },
            NP_TypeKeys::Decimal      => {    NP_Dec::get_size(cursor, memory) },
            NP_TypeKeys::Boolean      => {      bool::get_size(cursor, memory) },
            NP_TypeKeys::Geo          => {    NP_Geo::get_size(cursor, memory) },
            NP_TypeKeys::Uuid         => {   _NP_UUID::get_size(cursor, memory) },
            NP_TypeKeys::Ulid         => {   _NP_ULID::get_size(cursor, memory) },
            NP_TypeKeys::Date         => {   NP_Date::get_size(cursor, memory) },
            NP_TypeKeys::Enum         => { NP_Enum::get_size(cursor, memory) },
            NP_TypeKeys::Table        => {  NP_Table::get_size(cursor, memory) },
            NP_TypeKeys::Map          => {    NP_Map::get_size(cursor, memory) },
            NP_TypeKeys::List         => {   NP_List::get_size(cursor, memory) },
            NP_TypeKeys::Tuple        => {  NP_Tuple::get_size(cursor, memory) }
        }?;

        Ok(type_size + base_size)
    }
}


/// This trait is used to restrict which types can be set/get in the buffer
pub trait NP_Scalar {}

/// This trait is used to implement types as NoProto buffer types.
/// This includes all the type data, encoding and decoding methods.
#[doc(hidden)]
pub trait NP_Value<'value> {

    /// Get the type information for this type (static)
    /// 
    fn type_idx() -> (&'value str, NP_TypeKeys);

    /// Get the type information for this type (instance)
    /// 
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys);

    /// Convert the schema byte array for this type into JSON
    /// 
    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error>;

    /// Get the default schema value for this type
    /// 
    fn schema_default(_schema: &'value NP_Parsed_Schema) -> Option<Self> where Self: Sized;

    /// Parse JSON schema into schema
    ///
    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error>;

    /// Parse bytes into schema
    /// 
    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>);

    /// Set the value of this scalar into the buffer
    /// 
    fn set_value(_cursor: NP_Cursor, _memory: &NP_Memory<'value>, _value: Self) -> Result<NP_Cursor, NP_Error> where Self: Sized {
        let message = "This type doesn't support set_value!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value(_cursor: NP_Cursor, _memory: &'value NP_Memory<'value>) -> Result<Option<Self>, NP_Error> where Self: Sized {
        let message = "This type doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json(_cursor: &NP_Cursor, _memory: &'value NP_Memory<'value>) -> NP_JSON;

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size(_cursor: NP_Cursor, memory: &NP_Memory<'value>) -> Result<usize, NP_Error>;
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact(from_cursor: &NP_Cursor, from_memory: &'value NP_Memory<'value>, to_cursor: NP_Cursor, to_memory: &'value NP_Memory<'value>) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        match Self::into_value(from_cursor.clone(), from_memory)? {
            Some(x) => {
                return Self::set_value(to_cursor, to_memory, x);
            },
            None => { }
        }

        Ok(to_cursor)
    }
}



/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
*/