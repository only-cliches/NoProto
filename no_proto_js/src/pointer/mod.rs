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

use core::{fmt::{Debug}};

use alloc::prelude::v1::Box;
use crate::{pointer::dec::NP_Dec, schema::{NP_Parsed_Schema, NP_Schema_Addr}};
// use crate::NP_Parsed_Schema;
use crate::{json_flex::NP_JSON};
use crate::memory::{NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_TypeKeys}};
// use crate::{schema::{NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}};

use alloc::{string::String, vec::Vec, borrow::ToOwned};
use bytes::NP_Bytes;

use self::{date::NP_Date, geo::NP_Geo, option::NP_Enum, string::NP_String, ulid::{NP_ULID, _NP_ULID}, uuid::{NP_UUID, _NP_UUID}};

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct NP_Pointer_Scalar {
    pub addr_value: [u8; 2]
}

impl Default for NP_Pointer_Scalar {
    fn default() -> Self {
        Self { addr_value: [0; 2] }
    }
}

#[doc(hidden)]
#[derive(Debug)]
#[repr(C)]
pub struct NP_Pointer_List_Item {
    pub addr_value: [u8; 2],
    pub next_value: [u8; 2],
    pub index: u8
}

#[doc(hidden)]
#[derive(Debug)]
#[repr(C)]
pub struct NP_Pointer_Map_Item {
    pub addr_value: [u8; 2],
    pub next_value: [u8; 2],
    pub key_addr: [u8; 2]
}

#[doc(hidden)]
#[allow(missing_docs, unused_variables)]
pub trait NP_Pointer_Bytes {
    fn get_type(&self) -> &str                                     { "" }
    fn get_addr_value(&self) -> u16                                { 0 }
    fn set_addr_value(&mut self, addr: u16)                        {   }
    fn get_next_addr(&self) -> u16                                 { 0 }
    fn set_next_addr(&mut self, addr: u16)                         {   }
    fn set_index(&mut self, index: u8)                             {   }
    fn get_index(&self) -> u8                                      { 0 }
    fn set_key_addr(&mut self, hash: u16)                          {   }
    fn get_key_addr(&self) -> u16                                  { 0 }
    fn reset(&mut self)                                            {   }
    fn get_size(&self) -> usize                                    { 0 }
    fn get_key<'key>(&self, memory: &'key dyn NP_Memory) -> &'key str  { "" }
    fn get_key_size<'key>(&self, memory: &'key dyn NP_Memory) -> usize { 0  }
}

impl NP_Pointer_Bytes for NP_Pointer_Scalar {
    fn get_type(&self) -> &str { "Scalar" }
    #[inline(always)]
    fn get_addr_value(&self) -> u16 { u16::from_be_bytes(self.addr_value) }
    #[inline(always)]
    fn set_addr_value(&mut self, addr: u16) { self.addr_value = addr.to_be_bytes() }
    #[inline(always)]
    fn reset(&mut self) { self.addr_value = [0; 2]; }
    #[inline(always)]
    fn get_size(&self) -> usize { 2 }
}
impl NP_Pointer_Bytes for NP_Pointer_List_Item {
    fn get_type(&self) -> &str { "List Item" }
    #[inline(always)]
    fn get_addr_value(&self) -> u16 { u16::from_be_bytes(self.addr_value) }
    #[inline(always)]
    fn set_addr_value(&mut self, addr: u16) { self.addr_value = addr.to_be_bytes() }
    #[inline(always)]
    fn get_next_addr(&self) -> u16 { u16::from_be_bytes(self.next_value) }
    #[inline(always)]
    fn set_next_addr(&mut self, addr: u16) { self.next_value = addr.to_be_bytes() }
    #[inline(always)]
    fn set_index(&mut self, index: u8)  { self.index = index }
    #[inline(always)]
    fn get_index(&self) -> u8  { self.index }
    #[inline(always)]
    fn reset(&mut self) { self.addr_value = [0; 2]; self.next_value = [0; 2]; self.index = 0; }
    #[inline(always)]
    fn get_size(&self) -> usize { 5 }
}
impl NP_Pointer_Bytes for NP_Pointer_Map_Item {
    fn get_type(&self) -> &str { "Map Item" }
    #[inline(always)]
    fn get_addr_value(&self) -> u16 { u16::from_be_bytes(self.addr_value) }
    #[inline(always)]
    fn set_addr_value(&mut self, addr: u16) { self.addr_value = addr.to_be_bytes() }
    #[inline(always)]
    fn get_next_addr(&self) -> u16 { u16::from_be_bytes(self.next_value) }
    #[inline(always)]
    fn set_next_addr(&mut self, addr: u16) { self.next_value = addr.to_be_bytes() }
    #[inline(always)]
    fn set_key_addr(&mut self, addr: u16)  { self.key_addr = addr.to_be_bytes(); }
    #[inline(always)]
    fn get_key_addr(&self) -> u16  { u16::from_be_bytes(self.key_addr) }
    #[inline(always)]
    fn reset(&mut self) { self.addr_value = [0; 2]; self.next_value = [0; 2]; self.key_addr = [0;2 ]; }
    #[inline(always)]
    fn get_size(&self) -> usize { 6 }
    #[inline(always)]
    fn get_key<'key>(&self, memory: &'key dyn NP_Memory) -> &'key str {
        let key_addr = self.get_key_addr() as usize;
        if key_addr == 0 {
            return "";
        } else {
            let key_length = memory.read_bytes()[key_addr] as usize;
            let key_bytes = &memory.read_bytes()[(key_addr + 1)..(key_addr + 1 + key_length)];
            unsafe { core::str::from_utf8_unchecked(key_bytes) }
        }
    }
    #[inline(always)]
    fn get_key_size<'key>(&self, memory: &'key dyn NP_Memory) -> usize {
        let key_addr = self.get_key_addr() as usize;
        if key_addr == 0 {
            return 0;
        } else {
            return memory.read_bytes()[key_addr] as usize;
        }
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc(hidden)]
#[allow(missing_docs)]
pub struct NP_Map_Bytes {
    head: [u8; 2]
}

#[allow(missing_docs)]
impl NP_Map_Bytes {
    #[inline(always)]
    pub fn set_head(&mut self, head: u16) {
        self.head = head.to_be_bytes();
    }
    #[inline(always)]
    pub fn get_head(&self) -> u16 {
        u16::from_be_bytes(self.head)
    }
}

#[repr(C)]
#[derive(Debug)]
#[doc(hidden)]
#[allow(missing_docs)]
pub struct NP_List_Bytes {
    head: [u8; 2],
    tail: [u8; 2]
}

#[allow(missing_docs)]
impl NP_List_Bytes {
    #[inline(always)]
    pub fn set_head(&mut self, head: u16) {
        self.head = head.to_be_bytes();
    }
    #[inline(always)]
    pub fn get_head(&self) -> u16 {
        u16::from_be_bytes(self.head)
    }
    #[inline(always)]
    pub fn set_tail(&mut self, tail: u16) {
        self.tail = tail.to_be_bytes();
    }
    #[inline(always)]
    pub fn get_tail(&self) -> u16 {
        u16::from_be_bytes(self.tail)
    }
}

// holds 4 u16 addresses and a next value (10 bytes)
#[repr(C)]
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
#[allow(missing_docs)]
pub struct NP_Vtable {
    pub values: [NP_Pointer_Scalar; 4],
    next: [u8; 2]
}


#[allow(missing_docs)]
impl NP_Vtable {

    #[inline(always)]
    pub fn get_next(&self) -> u16 {
        u16::from_be_bytes(unsafe { *(&self.next as *const [u8] as *const [u8; 2]) }) 
    }

    #[inline(always)]
    pub fn set_next(&mut self, value: u16) {
        let bytes = value.to_be_bytes();
        self.next[0] = bytes[0];
        self.next[1] = bytes[1];
    }
}

/// Cursor for pointer value in buffer
/// 
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub struct NP_Cursor {
    /// The location of this cursor in the buffer
    pub buff_addr: usize,
    /// The address of the schema for this cursor
    pub schema_addr: NP_Schema_Addr,
    /// the values of the buffer pointer
    pub parent_schema_addr: usize
}

impl<'cursor> NP_Cursor {

    /// Create a new cursor
    pub fn new(buff_addr: usize, schema_addr: usize, parent_schema_addr: usize) -> Self {
        Self {
            buff_addr,
            schema_addr,
            parent_schema_addr
        }
    }
    
    /// Get the value bytes of this cursor
    #[inline(always)]
    pub fn get_value<X: NP_Memory>(&self, memory: &X) -> &'cursor mut dyn NP_Pointer_Bytes {
        // let ptr = memory.write_bytes().as_mut_ptr();
        // // if requesting root pointer or address is higher than buffer length
        // if self.buff_addr == memory.get_root() || self.buff_addr > memory.read_bytes().len() {
        //     unsafe { &mut *(ptr.add(memory.get_root()) as *mut NP_Pointer_Scalar) }
        // } else {
        //     match memory.get_schema(self.parent_schema_addr) {
        //         NP_Parsed_Schema::List { .. } => {
        //             unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_List_Item) }
        //         },
        //         NP_Parsed_Schema::Map { .. } => {
        //             unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_Map_Item) }
        //         },
        //         _ => { // parent is scalar, table or tuple
        //             unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_Scalar) }
        //         }
        //     }                   
        // }
        panic!()
    }

    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    /// 
    pub fn json_encode<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> NP_JSON {

        // match memory.get_schema(cursor.schema_addr).get_type_key() {
        //     NP_TypeKeys::None           => { NP_JSON::Null },
        //     NP_TypeKeys::Any            => { NP_JSON::Null },
        //     NP_TypeKeys::UTF8String     => { NP_String::to_json(cursor, memory) },
        //     NP_TypeKeys::Bytes          => {  NP_Bytes::to_json(cursor, memory) },
        //     NP_TypeKeys::Int8           => {        i8::to_json(cursor, memory) },
        //     NP_TypeKeys::Int16          => {       i16::to_json(cursor, memory) },
        //     NP_TypeKeys::Int32          => {       i32::to_json(cursor, memory) },
        //     NP_TypeKeys::Int64          => {       i64::to_json(cursor, memory) },
        //     NP_TypeKeys::Uint8          => {        u8::to_json(cursor, memory) },
        //     NP_TypeKeys::Uint16         => {       u16::to_json(cursor, memory) },
        //     NP_TypeKeys::Uint32         => {       u32::to_json(cursor, memory) },
        //     NP_TypeKeys::Uint64         => {       u64::to_json(cursor, memory) },
        //     NP_TypeKeys::Float          => {       f32::to_json(cursor, memory) },
        //     NP_TypeKeys::Double         => {       f64::to_json(cursor, memory) },
        //     NP_TypeKeys::Decimal        => {    NP_Dec::to_json(cursor, memory) },
        //     NP_TypeKeys::Boolean        => {      bool::to_json(cursor, memory) },
        //     NP_TypeKeys::Geo            => {    NP_Geo::to_json(cursor, memory) },
        //     NP_TypeKeys::Uuid           => {  _NP_UUID::to_json(cursor, memory) },
        //     NP_TypeKeys::Ulid           => {  _NP_ULID::to_json(cursor, memory) },
        //     NP_TypeKeys::Date           => {   NP_Date::to_json(cursor, memory) },
        //     NP_TypeKeys::Enum           => {   NP_Enum::to_json(cursor, memory) },
        //     NP_TypeKeys::Table          => {  NP_Table::to_json(cursor, memory) },
        //     NP_TypeKeys::Map            => {    NP_Map::to_json(cursor, memory) },
        //     NP_TypeKeys::List           => {   NP_List::to_json(cursor, memory) },
        //     NP_TypeKeys::Tuple          => {  NP_Tuple::to_json(cursor, memory) }
        // }
            panic!()
    }

    /// Compact from old cursor and memory into new cursor and memory
    /// 
    pub fn compact<M: NP_Memory, M2: NP_Memory>(from_cursor: NP_Cursor, from_memory: &M, to_cursor: NP_Cursor, to_memory: &M2) -> Result<NP_Cursor, NP_Error> {

        // match from_memory.get_schema(from_cursor.schema_addr).get_type_key() {
        //     NP_TypeKeys::Any           => { Ok(to_cursor) }
        //     NP_TypeKeys::UTF8String    => { NP_String::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Bytes         => {  NP_Bytes::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Int8          => {        i8::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Int16         => {       i16::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Int32         => {       i32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Int64         => {       i64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Uint8         => {        u8::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Uint16        => {       u16::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Uint32        => {       u32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Uint64        => {       u64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Float         => {       f32::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Double        => {       f64::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Decimal       => {    NP_Dec::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Boolean       => {      bool::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Geo           => {    NP_Geo::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Uuid          => {  _NP_UUID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Ulid          => {  _NP_ULID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Date          => {   NP_Date::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Enum          => {   NP_Enum::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Table         => {  NP_Table::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Map           => {    NP_Map::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::List          => {   NP_List::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     NP_TypeKeys::Tuple         => {  NP_Tuple::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
        //     _ => { Err(NP_Error::new("unreachable")) }
        // }
        panic!()
    }

    /// Set default for this value.  Not related to the schema default, this is the default value for this data type
    /// 
    pub fn set_default<M: NP_Memory>(cursor: NP_Cursor, memory: &M) -> Result<(), NP_Error> {

        // match memory.get_schema(cursor.schema_addr).get_type_key() {
        //     NP_TypeKeys::None        => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::Any         => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::Table       => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::Map         => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::List        => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::Tuple       => { return Err(NP_Error::new("unreachable")); },
        //     NP_TypeKeys::UTF8String  => {  NP_String::set_value(cursor, memory, &String::default())?; },
        //     NP_TypeKeys::Bytes       => {   NP_Bytes::set_value(cursor, memory, &NP_Bytes::default())?; },
        //     NP_TypeKeys::Int8        => {         i8::set_value(cursor, memory, i8::default())?; },
        //     NP_TypeKeys::Int16       => {        i16::set_value(cursor, memory, i16::default())?; },
        //     NP_TypeKeys::Int32       => {        i32::set_value(cursor, memory, i32::default())?; },
        //     NP_TypeKeys::Int64       => {        i64::set_value(cursor, memory, i64::default())?; },
        //     NP_TypeKeys::Uint8       => {         u8::set_value(cursor, memory, u8::default())?; },
        //     NP_TypeKeys::Uint16      => {        u16::set_value(cursor, memory, u16::default())?; },
        //     NP_TypeKeys::Uint32      => {        u32::set_value(cursor, memory, u32::default())?; },
        //     NP_TypeKeys::Uint64      => {        u64::set_value(cursor, memory, u64::default())?; },
        //     NP_TypeKeys::Float       => {        f32::set_value(cursor, memory, f32::default())?; },
        //     NP_TypeKeys::Double      => {        f64::set_value(cursor, memory, f64::default())?; },
        //     NP_TypeKeys::Decimal     => {     NP_Dec::set_value(cursor, memory, NP_Dec::default())?; },
        //     NP_TypeKeys::Boolean     => {       bool::set_value(cursor, memory, bool::default())?; },
        //     NP_TypeKeys::Geo         => {     NP_Geo::set_value(cursor, memory, NP_Geo::default())?; },
        //     NP_TypeKeys::Uuid        => {   _NP_UUID::set_value(cursor, memory, &NP_UUID::default())?; },
        //     NP_TypeKeys::Ulid        => {   _NP_ULID::set_value(cursor, memory, &NP_ULID::default())?; },
        //     NP_TypeKeys::Date        => {    NP_Date::set_value(cursor, memory, NP_Date::default())?; },
        //     NP_TypeKeys::Enum        => {    NP_Enum::set_value(cursor, memory, NP_Enum::default())?; }
        // }

        Ok(())
    }

    /// Calculate the number of bytes used by this pointer and it's descendants.
    /// 
    pub fn calc_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {
        
        let value = cursor.get_value(memory);
    
        // size of pointer
        let base_size = value.get_size();

        // pointer is in buffer but has no value set
        if value.get_addr_value() == 0 { // no value, just base size
            return Ok(base_size);
        }
        let type_size = 0;
        // // get the size of the value based on schema
        // let type_size = match memory.get_schema(cursor.schema_addr).get_type_key() {
        //     NP_TypeKeys::None         => { Ok(0) },
        //     NP_TypeKeys::Any          => { Ok(0) },
        //     NP_TypeKeys::UTF8String   => { NP_String::get_size(cursor, memory) },
        //     NP_TypeKeys::Bytes        => {  NP_Bytes::get_size(cursor, memory) },
        //     NP_TypeKeys::Int8         => {        i8::get_size(cursor, memory) },
        //     NP_TypeKeys::Int16        => {       i16::get_size(cursor, memory) },
        //     NP_TypeKeys::Int32        => {       i32::get_size(cursor, memory) },
        //     NP_TypeKeys::Int64        => {       i64::get_size(cursor, memory) },
        //     NP_TypeKeys::Uint8        => {        u8::get_size(cursor, memory) },
        //     NP_TypeKeys::Uint16       => {       u16::get_size(cursor, memory) },
        //     NP_TypeKeys::Uint32       => {       u32::get_size(cursor, memory) },
        //     NP_TypeKeys::Uint64       => {       u64::get_size(cursor, memory) },
        //     NP_TypeKeys::Float        => {       f32::get_size(cursor, memory) },
        //     NP_TypeKeys::Double       => {       f64::get_size(cursor, memory) },
        //     NP_TypeKeys::Decimal      => {    NP_Dec::get_size(cursor, memory) },
        //     NP_TypeKeys::Boolean      => {      bool::get_size(cursor, memory) },
        //     NP_TypeKeys::Geo          => {    NP_Geo::get_size(cursor, memory) },
        //     NP_TypeKeys::Uuid         => {  _NP_UUID::get_size(cursor, memory) },
        //     NP_TypeKeys::Ulid         => {  _NP_ULID::get_size(cursor, memory) },
        //     NP_TypeKeys::Date         => {   NP_Date::get_size(cursor, memory) },
        //     NP_TypeKeys::Enum         => {   NP_Enum::get_size(cursor, memory) },
        //     NP_TypeKeys::Table        => {  NP_Table::get_size(cursor, memory) },
        //     NP_TypeKeys::Map          => {    NP_Map::get_size(cursor, memory) },
        //     NP_TypeKeys::List         => {   NP_List::get_size(cursor, memory) },
        //     NP_TypeKeys::Tuple        => {  NP_Tuple::get_size(cursor, memory) }
        // }?;

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
    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>);

    /// Set the value of this scalar into the buffer
    /// 
    fn set_value<'set, M: NP_Memory>(_cursor: NP_Cursor, _memory: &'set M, _value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {
        let message = "This type doesn't support set_value!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {
        let message = "This type doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &'value M) -> NP_JSON;

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size<M: NP_Memory>(cursor: &'value NP_Cursor, memory: &'value M) -> Result<usize, NP_Error>;
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact<M: NP_Memory, M2: NP_Memory>(from_cursor: NP_Cursor, from_memory: &'value M, to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        match Self::into_value(&from_cursor, from_memory)? {
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