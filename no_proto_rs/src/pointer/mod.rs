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
pub mod portal;
pub mod union;

use core::{fmt::{Debug}};

use alloc::prelude::v1::Box;
use crate::{idl::{JS_AST, JS_Schema}, pointer::dec::NP_Dec, schema::{NP_Schema_Addr}, utils::opt_err};
use crate::NP_Parsed_Schema;
use crate::{json_flex::NP_JSON};
use crate::memory::{NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_TypeKeys}, collection::{map::NP_Map, struc::NP_Struct, list::NP_List, tuple::NP_Tuple}};

use alloc::{string::String, vec::Vec, borrow::ToOwned};
use bytes::NP_Bytes;

use self::{date::NP_Date, geo::NP_Geo, option::NP_Enum, portal::NP_Portal, ulid::{NP_ULID}, union::NP_Union, uuid::{NP_UUID}};

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

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NP_Cursor_Parent {
    None,
    Tuple
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
    /// the parent schema address (so we know if we're in a collection type)
    pub parent_schema_addr: NP_Schema_Addr,
    /// used by tuple type to store scalar pointer bytes
    pub value_bytes: Option<[u8; 2]>,
    /// if parent is tuple
    pub parent_type: NP_Cursor_Parent
}

impl<'cursor> NP_Cursor {

    /// Create a new cursor
    pub fn new(buff_addr: usize, schema_addr: usize, parent_schema_addr: usize) -> Self {
        Self {
            buff_addr,
            schema_addr,
            parent_schema_addr,
            value_bytes: None,
            parent_type: NP_Cursor_Parent::None
        }
    }
    
    /// Get the value bytes of this cursor
    #[inline(always)]
    pub fn get_value<X: NP_Memory>(&self, memory: &X) -> &'cursor mut dyn NP_Pointer_Bytes {
        let ptr = memory.write_bytes().as_mut_ptr();
        // if requesting root pointer or address is higher than buffer length
        if self.buff_addr == memory.get_root() || self.buff_addr > memory.read_bytes().len() {
            unsafe { &mut *(ptr.add(memory.get_root()) as *mut NP_Pointer_Scalar) }
        } else {
            match memory.get_schema(self.parent_schema_addr) {
                NP_Parsed_Schema::List { .. } => {
                    unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_List_Item) }
                },
                NP_Parsed_Schema::Map { .. } => {
                    unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_Map_Item) }
                },
                NP_Parsed_Schema::Tuple { .. } => {
                    match &self.value_bytes {
                        Some(x) => unsafe { &mut *(x as *const u8 as *mut NP_Pointer_Scalar) },
                        None => unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_Scalar) }
                    }
                },
                _ => { // parent is scalar or struct
                    unsafe { &mut *(ptr.add(self.buff_addr) as *mut NP_Pointer_Scalar) }
                }
            }                   
        }
    }

    /// Given a starting cursor, select into the buffer at a new location
    /// 
    pub fn select<M: NP_Memory>(memory: &M, cursor: NP_Cursor, make_path: bool, schema_query: bool, path: &[&str]) -> Result<Option<NP_Cursor>, NP_Error> {

        let mut loop_cursor = cursor;
    
        let mut path_index = 0usize;
        
        let mut loop_count = 0u16;
    
        loop {
    
            loop_count += 1;
            
            if path.len() == path_index {
                return Ok(Some(loop_cursor));
            }
    
            if loop_count > 256 {
                return Err(NP_Error::RecursionLimit)
            }
    
            // now select into collections
            match memory.get_schema(loop_cursor.schema_addr) {
                NP_Parsed_Schema::Struct { fields, empty, .. } => {
                    if let Some(next) = NP_Struct::select(loop_cursor, empty, fields, path[path_index], make_path, schema_query, memory)? {
                        loop_cursor = next;
                        path_index += 1;
                    } else {
                        return Ok(None);
                    }
                },
                NP_Parsed_Schema::Tuple { values, empty, .. } => {
                    match path[path_index].parse::<usize>() {
                        Ok(x) => {
                            if let Some(next) = NP_Tuple::select(loop_cursor, empty, values, x, make_path, schema_query, memory)? {
                                loop_cursor = next;
                                path_index += 1;
                            } else {
                                return Ok(None);
                            }
                        },
                        Err(_e) => {
                            return Err(NP_Error::new("Need a number to index into tuple, string found!"))
                        }
                    }
                },
                NP_Parsed_Schema::List { .. } => {
                    match path[path_index].parse::<usize>() {
                        Ok(x) => {
                            if let Some(next) = NP_List::select(loop_cursor, x, make_path, schema_query, memory)? {
                                loop_cursor = opt_err(next.1)?;
                                path_index += 1;
                            } else {
                                return Ok(None);
                            }
                        },
                        Err(_e) => {
                            return Err(NP_Error::new("Need a number to index into list, string found!"))
                        }
                    }
                },
                NP_Parsed_Schema::Map { .. } => {
                    if let Some(next) = NP_Map::select(loop_cursor, path[path_index], make_path, schema_query, memory)? {
                        loop_cursor = next;
                        path_index += 1;
                    } else {
                        return Ok(None);
                    }
    
                },
                NP_Parsed_Schema::Union { types, .. } => {
                    if let Some(next) = NP_Union::select(loop_cursor, types, path[path_index], make_path, schema_query, memory)? {
                        loop_cursor = next;
                        path_index += 1;
                    } else {
                        return Ok(None);
                    }
                },
                NP_Parsed_Schema::Portal { schema, parent_schema, .. } => {
                    loop_cursor.schema_addr = *schema;
                    loop_cursor.parent_schema_addr = *parent_schema;
                },
                _ => { // we've reached a scalar value but not at the end of the path
                    return Ok(None);
                }
            }
        }
    }

    /// Set the max value at this cursor
    pub fn set_max<M: NP_Memory>(cursor: NP_Cursor, memory: &M) -> Result<bool, NP_Error> {

        if cursor.parent_type == NP_Cursor_Parent::Tuple {
            memory.write_bytes()[cursor.buff_addr - 1] = 1;
        }

        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Boolean    { .. } => {       bool::set_value(cursor, memory, opt_err(    bool::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::UTF8String { .. } => {     String::set_value(cursor, memory, opt_err(   String::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Bytes      { .. } => {   NP_Bytes::set_value(cursor, memory, opt_err( NP_Bytes::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int8       { .. } => {         i8::set_value(cursor, memory, opt_err(       i8::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int16      { .. } => {        i16::set_value(cursor, memory, opt_err(      i16::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int32      { .. } => {        i32::set_value(cursor, memory, opt_err(      i32::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int64      { .. } => {        i64::set_value(cursor, memory, opt_err(      i64::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint8      { .. } => {         u8::set_value(cursor, memory, opt_err(       u8::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint16     { .. } => {        u16::set_value(cursor, memory, opt_err(      u16::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint32     { .. } => {        u32::set_value(cursor, memory, opt_err(      u32::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint64     { .. } => {        u64::set_value(cursor, memory, opt_err(      u64::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Float      { .. } => {        f32::set_value(cursor, memory, opt_err(      f32::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Double     { .. } => {        f64::set_value(cursor, memory, opt_err(      f64::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Decimal    { .. } => {     NP_Dec::set_value(cursor, memory, opt_err(   NP_Dec::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Geo        { .. } => {     NP_Geo::set_value(cursor, memory, opt_err(   NP_Geo::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Date       { .. } => {    NP_Date::set_value(cursor, memory, opt_err(  NP_Date::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Enum       { .. } => {    NP_Enum::set_value(cursor, memory, opt_err(  NP_Enum::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uuid       { .. } => {    NP_UUID::set_value(cursor, memory, opt_err(  NP_UUID::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Ulid       { .. } => {    NP_ULID::set_value(cursor, memory, opt_err(  NP_ULID::np_max_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Struct      { .. } => {
                let mut struc = NP_Struct::new_iter(&cursor, memory);
                while let Some((_index, _key, item)) = struc.step_iter(memory) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_max(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::Tuple      { .. } => {
                let mut tuple = NP_Tuple::new_iter(&cursor, memory);
                while let Some((_index, item)) = tuple.step_iter(memory, false) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_max(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::List        { .. } => {
                let mut list = NP_List::new_iter(&cursor, memory, true, 0);
                while let Some((_index, item)) = list.step_iter(memory) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_max(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::Map        { .. } => {
                let mut map = NP_Map::new_iter(&cursor, memory);
                while let Some((_index, item_cursor)) = map.step_iter(memory) {
                    NP_Cursor::set_max(item_cursor.clone(), memory)?;
                }
            },
            _ => return Ok(false)
        };

        Ok(true)
    }

    /// Set the min value at this cursor
    pub fn set_min<M: NP_Memory>(cursor: NP_Cursor, memory: &M) -> Result<bool, NP_Error> {

        if cursor.parent_type == NP_Cursor_Parent::Tuple {
            memory.write_bytes()[cursor.buff_addr - 1] = 1;
        }

        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Boolean    { .. } => {       bool::set_value(cursor, memory, opt_err(    bool::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::UTF8String { .. } => {     String::set_value(cursor, memory, opt_err(   String::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Bytes      { .. } => {   NP_Bytes::set_value(cursor, memory, opt_err( NP_Bytes::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int8       { .. } => {         i8::set_value(cursor, memory, opt_err(       i8::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int16      { .. } => {        i16::set_value(cursor, memory, opt_err(      i16::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int32      { .. } => {        i32::set_value(cursor, memory, opt_err(      i32::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Int64      { .. } => {        i64::set_value(cursor, memory, opt_err(      i64::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint8      { .. } => {         u8::set_value(cursor, memory, opt_err(       u8::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint16     { .. } => {        u16::set_value(cursor, memory, opt_err(      u16::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint32     { .. } => {        u32::set_value(cursor, memory, opt_err(      u32::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uint64     { .. } => {        u64::set_value(cursor, memory, opt_err(      u64::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Float      { .. } => {        f32::set_value(cursor, memory, opt_err(      f32::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Double     { .. } => {        f64::set_value(cursor, memory, opt_err(      f64::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Decimal    { .. } => {     NP_Dec::set_value(cursor, memory, opt_err(   NP_Dec::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Geo        { .. } => {     NP_Geo::set_value(cursor, memory, opt_err(   NP_Geo::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Date       { .. } => {    NP_Date::set_value(cursor, memory, opt_err(  NP_Date::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Enum       { .. } => {    NP_Enum::set_value(cursor, memory, opt_err(  NP_Enum::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Uuid       { .. } => {    NP_UUID::set_value(cursor, memory, opt_err(  NP_UUID::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Ulid       { .. } => {    NP_ULID::set_value(cursor, memory, opt_err(  NP_ULID::np_min_value(&cursor, memory))?)?; } ,
            NP_Parsed_Schema::Struct      { .. } => {
                let mut struc = NP_Struct::new_iter(&cursor, memory);
                while let Some((_index, _key, item)) = struc.step_iter(memory) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_min(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::Tuple      { .. } => {
                let mut tuple = NP_Tuple::new_iter(&cursor, memory);
                while let Some((_index, item)) = tuple.step_iter(memory, false) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_min(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::List        { .. } => {
                let mut list = NP_List::new_iter(&cursor, memory, true, 0);
                while let Some((_index, item)) = list.step_iter(memory) {
                    if let Some(item_cursor) = item {
                        NP_Cursor::set_min(item_cursor.clone(), memory)?;
                    }
                }
            },
            NP_Parsed_Schema::Map        { .. } => {
                let mut map = NP_Map::new_iter(&cursor, memory);
                while let Some((_index, item_cursor)) = map.step_iter(memory) {
                    NP_Cursor::set_min(item_cursor.clone(), memory)?;
                }
            },
            _ => return Ok(false)
        };

        Ok(true)
    }

    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    /// 
    pub fn json_encode<M: NP_Memory>(depth: usize, cursor: &NP_Cursor, memory: &M) -> NP_JSON {

        if depth > 255 { return NP_JSON::Null }

        match memory.get_schema(cursor.schema_addr).get_type_key() {
            NP_TypeKeys::None           => { NP_JSON::Null },
            NP_TypeKeys::Any            => { NP_JSON::Null },
            NP_TypeKeys::UTF8String     => {    String::to_json(depth, cursor, memory) },
            NP_TypeKeys::Bytes          => {  NP_Bytes::to_json(depth, cursor, memory) },
            NP_TypeKeys::Int8           => {        i8::to_json(depth, cursor, memory) },
            NP_TypeKeys::Int16          => {       i16::to_json(depth, cursor, memory) },
            NP_TypeKeys::Int32          => {       i32::to_json(depth, cursor, memory) },
            NP_TypeKeys::Int64          => {       i64::to_json(depth, cursor, memory) },
            NP_TypeKeys::Uint8          => {        u8::to_json(depth, cursor, memory) },
            NP_TypeKeys::Uint16         => {       u16::to_json(depth, cursor, memory) },
            NP_TypeKeys::Uint32         => {       u32::to_json(depth, cursor, memory) },
            NP_TypeKeys::Uint64         => {       u64::to_json(depth, cursor, memory) },
            NP_TypeKeys::Float          => {       f32::to_json(depth, cursor, memory) },
            NP_TypeKeys::Double         => {       f64::to_json(depth, cursor, memory) },
            NP_TypeKeys::Decimal        => {    NP_Dec::to_json(depth, cursor, memory) },
            NP_TypeKeys::Boolean        => {      bool::to_json(depth, cursor, memory) },
            NP_TypeKeys::Geo            => {    NP_Geo::to_json(depth, cursor, memory) },
            NP_TypeKeys::Uuid           => {   NP_UUID::to_json(depth, cursor, memory) },
            NP_TypeKeys::Ulid           => {   NP_ULID::to_json(depth, cursor, memory) },
            NP_TypeKeys::Date           => {   NP_Date::to_json(depth, cursor, memory) },
            NP_TypeKeys::Enum           => {   NP_Enum::to_json(depth, cursor, memory) },
            NP_TypeKeys::Struct          => {  NP_Struct::to_json(depth, cursor, memory) },
            NP_TypeKeys::Map            => {    NP_Map::to_json(depth, cursor, memory) },
            NP_TypeKeys::List           => {   NP_List::to_json(depth, cursor, memory) },
            NP_TypeKeys::Tuple          => {  NP_Tuple::to_json(depth, cursor, memory) },
            NP_TypeKeys::Portal         => { NP_Portal::to_json(depth, cursor, memory) },
            NP_TypeKeys::Union          => {  NP_Union::to_json(depth, cursor, memory) },
        }

    }

    /// Compact from old cursor and memory into new cursor and memory
    /// 
    pub fn compact<M: NP_Memory, M2: NP_Memory>(depth: usize, from_cursor: NP_Cursor, from_memory: &M, to_cursor: NP_Cursor, to_memory: &M2) -> Result<NP_Cursor, NP_Error> {

        if depth > 255 { return Err(NP_Error::RecursionLimit)}

        match from_memory.get_schema(from_cursor.schema_addr).get_type_key() {
            NP_TypeKeys::Any           => { Ok(to_cursor) }
            NP_TypeKeys::UTF8String    => {    String::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Bytes         => {  NP_Bytes::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int8          => {        i8::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int16         => {       i16::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int32         => {       i32::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Int64         => {       i64::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint8         => {        u8::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint16        => {       u16::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint32        => {       u32::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uint64        => {       u64::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Float         => {       f32::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Double        => {       f64::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Decimal       => {    NP_Dec::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Boolean       => {      bool::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Geo           => {    NP_Geo::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Uuid          => {   NP_UUID::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Ulid          => {   NP_ULID::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Date          => {   NP_Date::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Enum          => {   NP_Enum::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Struct        => { NP_Struct::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Map           => {    NP_Map::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::List          => {   NP_List::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Tuple         => {  NP_Tuple::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Portal        => { NP_Portal::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            NP_TypeKeys::Union         => {  NP_Union::do_compact(depth, from_cursor, from_memory, to_cursor, to_memory) }
            _ => { Err(NP_Error::Unreachable) }
        }
    }


    /// Set default for this value.  Not related to the schema default, this is the default value for this data type
    /// 
    pub fn set_schema_default<M: NP_Memory>(cursor: NP_Cursor, memory: &M) -> Result<(), NP_Error> {

        let schema = memory.get_schema(cursor.schema_addr);

        match schema.get_type_key() {
            NP_TypeKeys::None        => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::Any         => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::Struct       => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::Map         => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::List        => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::Tuple       => { return Err(NP_Error::Unreachable); },
            NP_TypeKeys::Portal      => { return Err(NP_Error::new("Portal type does not have a default type")); },
            NP_TypeKeys::Union       => { return Err(NP_Error::new("Union type does not have a default type")); },
            NP_TypeKeys::UTF8String  => {     String::set_value(cursor, memory, opt_err(String::schema_default(schema))?)?; },
            NP_TypeKeys::Bytes       => {   NP_Bytes::set_value(cursor, memory, opt_err(NP_Bytes::schema_default(schema))?)?; },
            NP_TypeKeys::Int8        => {         i8::set_value(cursor, memory, opt_err(i8::schema_default(schema))?)?; },
            NP_TypeKeys::Int16       => {        i16::set_value(cursor, memory, opt_err(i16::schema_default(schema))?)?; },
            NP_TypeKeys::Int32       => {        i32::set_value(cursor, memory, opt_err(i32::schema_default(schema))?)?; },
            NP_TypeKeys::Int64       => {        i64::set_value(cursor, memory, opt_err(i64::schema_default(schema))?)?; },
            NP_TypeKeys::Uint8       => {         u8::set_value(cursor, memory, opt_err(u8::schema_default(schema))?)?; },
            NP_TypeKeys::Uint16      => {        u16::set_value(cursor, memory, opt_err(u16::schema_default(schema))?)?; },
            NP_TypeKeys::Uint32      => {        u32::set_value(cursor, memory, opt_err(u32::schema_default(schema))?)?; },
            NP_TypeKeys::Uint64      => {        u64::set_value(cursor, memory, opt_err(u64::schema_default(schema))?)?; },
            NP_TypeKeys::Float       => {        f32::set_value(cursor, memory, opt_err(f32::schema_default(schema))?)?; },
            NP_TypeKeys::Double      => {        f64::set_value(cursor, memory, opt_err(f64::schema_default(schema))?)?; },
            NP_TypeKeys::Decimal     => {     NP_Dec::set_value(cursor, memory, opt_err(NP_Dec::schema_default(schema))?)?; },
            NP_TypeKeys::Boolean     => {       bool::set_value(cursor, memory, opt_err(bool::schema_default(schema))?)?; },
            NP_TypeKeys::Geo         => {     NP_Geo::set_value(cursor, memory, opt_err(NP_Geo::schema_default(schema))?)?; },
            NP_TypeKeys::Uuid        => {    NP_UUID::set_value(cursor, memory, opt_err(NP_UUID::schema_default(schema))?)?; },
            NP_TypeKeys::Ulid        => {    NP_ULID::set_value(cursor, memory, opt_err(NP_ULID::schema_default(schema))?)?; },
            NP_TypeKeys::Date        => {    NP_Date::set_value(cursor, memory, opt_err(NP_Date::schema_default(schema))?)?; },
            NP_TypeKeys::Enum        => {    NP_Enum::set_value(cursor, memory, opt_err(NP_Enum::schema_default(schema))?)?; }
        }

        Ok(())
    }

    /// Set a JSON value into the buffer
    pub fn set_from_json<M: NP_Memory>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &M, json: &Box<NP_JSON>) -> Result<(), NP_Error> {


        if depth > 255 { return Err(NP_Error::RecursionLimit) }

        // if apply_null is true, we should delete values where we find "null" or "undefined"
        // if apply_null && **json == NP_JSON::Null {
        //     NP_Cursor::delete(cursor, memory)?;
        //     return Ok(())
        // }

        if cursor.parent_type == NP_Cursor_Parent::Tuple {
            memory.write_bytes()[cursor.buff_addr - 1] = 1;
        }

        match memory.get_schema(cursor.schema_addr).get_type_key() {
            NP_TypeKeys::None           => { Ok(()) },
            NP_TypeKeys::Any            => { Ok(()) },
            NP_TypeKeys::UTF8String     => {    String::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Bytes          => {  NP_Bytes::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Int8           => {        i8::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Int16          => {       i16::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Int32          => {       i32::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Int64          => {       i64::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Uint8          => {        u8::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Uint16         => {       u16::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Uint32         => {       u32::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Uint64         => {       u64::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Float          => {       f32::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Double         => {       f64::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Decimal        => {    NP_Dec::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Boolean        => {      bool::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Geo            => {    NP_Geo::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Uuid           => {   NP_UUID::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Ulid           => {   NP_ULID::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Date           => {   NP_Date::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Enum           => {   NP_Enum::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Struct          => {  NP_Struct::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Map            => {    NP_Map::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::List           => {   NP_List::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Tuple          => {  NP_Tuple::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Portal         => { NP_Portal::set_from_json(depth, apply_null, cursor, memory, json) },
            NP_TypeKeys::Union          => {  NP_Union::set_from_json(depth, apply_null, cursor, memory, json) },
        }
    }

    /// Delete the value at this cursor
    /// 
    /// Returns `true` if something was deleted, `false` otherwise.
    /// 
    pub fn delete<M: NP_Memory>(cursor: NP_Cursor, memory: &M) -> Result<bool, NP_Error> {
        
        if cursor.buff_addr == 0 {
            return Ok(false)
        }

        if cursor.parent_type == NP_Cursor_Parent::Tuple {
            memory.write_bytes()[cursor.buff_addr - 1] = 0;
        }

        let is_sortable = match memory.get_schema(0) {
            NP_Parsed_Schema::Tuple { sortable , ..} => *sortable,
            _ => false
        };
        
        if is_sortable {
            match memory.get_schema(cursor.schema_addr) {
                NP_Parsed_Schema::Struct { .. } => { return Ok(false) },
                NP_Parsed_Schema::Tuple { .. } => { return Ok(false) },
                NP_Parsed_Schema::List  { .. } => { return Ok(false) },
                NP_Parsed_Schema::Map   { .. } => { return Ok(false) },
                _ => NP_Cursor::set_schema_default(cursor, memory)?
            }
        } else {
            // clear value address in buffer
            cursor.get_value(memory).set_addr_value(0);
        }

        Ok(true)
    }

    /// Calculate the number of bytes used by this pointer and it's descendants.
    /// 
    pub fn calc_size<M: NP_Memory>(depth: usize, cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        if depth > 255 { return Err(NP_Error::new("Depth error!")) }
        
        let value = cursor.get_value(memory);

        let type_key = memory.get_schema(cursor.schema_addr).get_type_key();

        // size of pointer
        let base_size = if *type_key == NP_TypeKeys::Portal { 0 } else { value.get_size() };

        // pointer is in buffer but has no value set
        if value.get_addr_value() == 0 { // no value, just base size
            return Ok(base_size);
        }

        // get the size of the value based on schema
        let type_size = match type_key {
            NP_TypeKeys::None         => { Ok(0) },
            NP_TypeKeys::Any          => { Ok(0) },
            NP_TypeKeys::UTF8String   => {    String::get_size(depth, cursor, memory) },
            NP_TypeKeys::Bytes        => {  NP_Bytes::get_size(depth, cursor, memory) },
            NP_TypeKeys::Int8         => {        i8::get_size(depth, cursor, memory) },
            NP_TypeKeys::Int16        => {       i16::get_size(depth, cursor, memory) },
            NP_TypeKeys::Int32        => {       i32::get_size(depth, cursor, memory) },
            NP_TypeKeys::Int64        => {       i64::get_size(depth, cursor, memory) },
            NP_TypeKeys::Uint8        => {        u8::get_size(depth, cursor, memory) },
            NP_TypeKeys::Uint16       => {       u16::get_size(depth, cursor, memory) },
            NP_TypeKeys::Uint32       => {       u32::get_size(depth, cursor, memory) },
            NP_TypeKeys::Uint64       => {       u64::get_size(depth, cursor, memory) },
            NP_TypeKeys::Float        => {       f32::get_size(depth, cursor, memory) },
            NP_TypeKeys::Double       => {       f64::get_size(depth, cursor, memory) },
            NP_TypeKeys::Decimal      => {    NP_Dec::get_size(depth, cursor, memory) },
            NP_TypeKeys::Boolean      => {      bool::get_size(depth, cursor, memory) },
            NP_TypeKeys::Geo          => {    NP_Geo::get_size(depth, cursor, memory) },
            NP_TypeKeys::Uuid         => {   NP_UUID::get_size(depth, cursor, memory) },
            NP_TypeKeys::Ulid         => {   NP_ULID::get_size(depth, cursor, memory) },
            NP_TypeKeys::Date         => {   NP_Date::get_size(depth, cursor, memory) },
            NP_TypeKeys::Enum         => {   NP_Enum::get_size(depth, cursor, memory) },
            NP_TypeKeys::Struct        => {  NP_Struct::get_size(depth, cursor, memory) },
            NP_TypeKeys::Map          => {    NP_Map::get_size(depth, cursor, memory) },
            NP_TypeKeys::List         => {   NP_List::get_size(depth, cursor, memory) },
            NP_TypeKeys::Tuple        => {  NP_Tuple::get_size(depth, cursor, memory) },
            NP_TypeKeys::Portal       => { NP_Portal::get_size(depth, cursor, memory) },
            NP_TypeKeys::Union        => {  NP_Union::get_size(depth, cursor, memory) },
        }?;

        Ok(type_size + base_size)
    }
}


/// This trait is used to restrict which types can be set/get in the buffer
pub trait NP_Scalar<'scalar> {
    /// Get the default for the schema type
    /// Does NOT get the `default` property of the schema, but generates a default value based on the schema settings
    fn schema_default(_schema: &'scalar NP_Parsed_Schema) -> Option<Self> where Self: Sized;

    /// Get the max value for this data type
    fn np_max_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &M) -> Option<Self> where Self: Sized;

    /// Get the min value for this data type
    fn np_min_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &M) -> Option<Self> where Self: Sized;

}

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

    /// Export schema to IDL
    /// 
    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error>;

    /// Parse JSON schema into schema
    ///
    fn from_idl_to_schema(schema: Vec<NP_Parsed_Schema>, name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error>;

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

    /// Set value from JSON
    /// 
    fn set_from_json<'set, M: NP_Memory>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &'set M, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized;

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value<M: NP_Memory>(_cursor: &NP_Cursor, _memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {
        let message = "This type doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Get the default value from the schema
    /// 
    fn default_value(depth: usize, scham_addr: usize, schema: &'value Vec<NP_Parsed_Schema>) -> Option<Self> where Self: Sized;

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json<M: NP_Memory>(depth: usize, cursor: &NP_Cursor, memory: &'value M) -> NP_JSON;

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size<M: NP_Memory>(depth: usize, cursor: &'value NP_Cursor, memory: &'value M) -> Result<usize, NP_Error>;
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact<M: NP_Memory, M2: NP_Memory>(_depth: usize, from_cursor: NP_Cursor, from_memory: &'value M, to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

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