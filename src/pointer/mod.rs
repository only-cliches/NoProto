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

use crate::{collection::NP_Collection, pointer::dec::NP_Dec, schema::NP_Schema_Addr};
use crate::NP_Parsed_Schema;
use crate::{json_flex::NP_JSON};
use crate::memory::{NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}, utils::{print_path}};

use alloc::{boxed::Box, string::String, vec::Vec, borrow::ToOwned};
use bytes::NP_Bytes;

use self::{date::NP_Date, geo::NP_Geo, option::NP_Option, ulid::NP_ULID, uuid::NP_UUID};

#[derive(Debug, Clone, Copy)]
pub struct NP_Cursor_Addr {
    pub address: usize,
    pub is_virtual: bool
}

impl<'cursor> NP_Cursor_Addr {

    pub fn get_schema_data(&self, memory: &'cursor NP_Memory<'cursor>) -> &'cursor NP_Parsed_Schema<'cursor> {
        &memory.schema[self.get_data(memory).unwrap().schema]
    }

    pub fn get_schema_data_owned(&self, memory: NP_Memory<'cursor>) -> &'cursor NP_Parsed_Schema<'cursor> {
        &memory.schema[self.get_data(&memory).unwrap().schema]
    }

    pub fn get_data<'data>(&'data self, memory: &'data NP_Memory<'data>) -> Result<&mut NP_Cursor<'data>, NP_Error> {
        if self.is_virtual {
            Ok(unsafe { &mut *memory.virtual_cursor.get() })
        } else {
            let cache = unsafe { &mut *memory.cursor_cache.get() };
            if let Some(c) = cache.get_mut(self.address) {
                Ok(c)
            } else {
                Err(NP_Error::new("Attempted to get cached cursor that didn't exist!"))
            }
        }
    }

    pub fn get_data_owned(&self, memory: NP_Memory<'cursor>) -> Result<&mut NP_Cursor<'cursor>, NP_Error> {
        if self.is_virtual {
            Ok(unsafe { &mut *memory.virtual_cursor.get() })
        } else {
            let cache = unsafe { &mut *memory.cursor_cache.get() };
            if let Some(c) = cache.get_mut(self.address) {
                Ok(c)
            } else {
                Err(NP_Error::new("Attempted to get cached cursor that didn't exist!"))
            }
        }
    }

    /// Delete value at this pointer
    pub fn clear_here<'clear>(&self, memory: NP_Memory<'clear>) -> bool {

        if self.is_virtual == false {
            let cursor = self.get_data(&memory).unwrap();
            if cursor.address_value != 0 {
                cursor.address_value = 0;
                memory.set_value_address(cursor.address, 0);
                true
            } else {
                false
            }
        } else {
            false     
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NP_Cursor<'cursor> {
    pub address: usize,
    pub address_value: usize,
    pub schema: NP_Schema_Addr,
    pub parent_addr: usize,
    pub kind: NP_Cursor_Kinds,

    pub item_next_addr: Option<usize>,
    pub item_prev_addr: Option<usize>,
    pub item_index: Option<usize>,
    pub item_key_addr: Option<usize>,
    pub item_key: Option<&'cursor str>,

    pub coll_head: Option<usize>,
    pub coll_tail: Option<usize>,
    pub coll_length: Option<usize>
}

impl<'cursor> Default for NP_Cursor<'cursor> {
    fn default() -> Self {
        NP_Cursor {
            address: 0,
            address_value: 0,
            schema: 0,
            parent_addr: 0,
            kind: NP_Cursor_Kinds::None,
            item_next_addr: None,
            item_prev_addr: None,
            item_index: None,
            item_key_addr: None,
            item_key: None,
            coll_head: None,
            coll_tail: None,
            coll_length: None
        }
    }
}

impl<'cursor> NP_Cursor<'cursor> {


    pub fn new(address: usize, parent: usize, schema_addr: NP_Schema_Addr, memory: &NP_Memory) -> Self {
        NP_Cursor {
            address: address,
            address_value: memory.read_address(address),
            schema: schema_addr,
            parent_addr: parent,
            kind: NP_Cursor_Kinds::Standard,
            item_next_addr: None,
            item_prev_addr: None,
            item_index: None,
            item_key_addr: None,
            item_key: None,
            coll_head: None,
            coll_tail: None,
            coll_length: None
        }
    }

    pub fn schema_data(&self, memory: &'cursor NP_Memory<'cursor>) -> &'cursor NP_Parsed_Schema<'cursor> {
        &memory.schema[self.schema]
    }

    /// Deep get a value
    /// 
    pub fn select(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, path: &'cursor [&str], path_index: usize) -> Result<Option<NP_Cursor_Addr>, NP_Error> {

        if path.len() == path_index {
            return Ok(Some(cursor_addr));
        }

        match cursor_addr.get_schema_data(&memory).get_type_key() {
            NP_TypeKeys::Table => {
                let new_cursor = NP_Table::select_to_ptr(cursor_addr, &memory, &path[path_index], None)?;
                match new_cursor {
                    Some(x ) => NP_Cursor::select(x, memory, path, path_index + 1),
                    None => Ok(None)
                }
                
            },
            NP_TypeKeys::Map => {
                let new_cursor = NP_Map::select_to_ptr(cursor_addr, &memory, &path[path_index], false)?;
                NP_Cursor::select(new_cursor, memory, path, path_index + 1)
            },
            NP_TypeKeys::List => {
            
                let list_key = &path[path_index];
                let list_key_int = list_key.parse::<u16>();
                match list_key_int {
                    Ok(x) => {
                        let new_cursor = NP_List::select_to_ptr(cursor_addr, &memory, x)?;
                        NP_Cursor::select(new_cursor, memory, path, path_index + 1)
                    },
                    Err(_e) => {
                        let mut err = String::from("Can't query list with string, need number! Path: \n");
                        err.push_str(print_path(&path, path_index).as_str());
                        Err(NP_Error::new(err))
                    }
                }
           
            },
            NP_TypeKeys::Tuple => {

                let list_key = &path[path_index];
                let list_key_int = list_key.parse::<u8>();
                match list_key_int {
                    Ok(x) => {
                        match NP_Tuple::select_to_ptr(cursor_addr, &memory, x as usize)? {
                            Some(y) => NP_Cursor::select(y, memory, path, path_index + 1),
                            None => Ok(None)
                        }
                    },
                    Err(_e) => {
                        let mut err = String::from("Can't query tuple with string, need number! Path: \n");
                        err.push_str(print_path(&path, path_index).as_str());
                        Err(NP_Error::new(err))
                    }
                }
                 
            },
            _ => { 
                // we're not at the end of the select path but we've reached a scalar value
                // so the select has failed to find anything
                return Ok(Some(cursor_addr));
            }
        }
    }

    pub fn select_with_commit<'sel>(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, path: &'sel [&str], path_index: usize) -> Result<Option<NP_Cursor_Addr>, NP_Error> {

        if path.len() == path_index {
            return Ok(Some(cursor_addr));
        }

        match cursor_addr.get_schema_data(&memory).get_type_key() {
            NP_TypeKeys::Table => {
                let mut new_cursor = NP_Table::select_to_ptr(cursor_addr, &memory, &path[path_index], None)?;
                match new_cursor {
                    Some(x) => {
                        let new_cursor = NP_Table::commit_pointer(&x, &memory)?;
                        NP_Cursor::select_with_commit(new_cursor, memory, path, path_index + 1)
                    },
                    None => Ok(None)
                }
            },
            NP_TypeKeys::Map => {
                let mut new_cursor = NP_Map::select_to_ptr(cursor_addr, &memory, &path[path_index], false)?;
                new_cursor = NP_Map::commit_pointer(&new_cursor, &memory)?;
                NP_Cursor::select_with_commit(new_cursor, memory, path, path_index + 1)
            },
            NP_TypeKeys::List => {

                let list_key_int = (&path[path_index]).parse::<u16>();
                match list_key_int {
                    Ok(x) => {
                        let new_cursor = NP_List::select_to_ptr(cursor_addr, &memory, x)?;
                        let new_cursor = NP_List::commit_pointer(&new_cursor, &memory)?;
                        NP_Cursor::select_with_commit(new_cursor, memory, path, path_index + 1)

                    },
                    Err(_e) => {
                        let mut err = String::from("Can't query list with string, need number! Path: \n");
                        err.push_str(print_path(&path, path_index).as_str());
                        Err(NP_Error::new(err))
                    }
                }
            },
            NP_TypeKeys::Tuple => {

                let list_key = &path[path_index];
                let list_key_int = list_key.parse::<u8>();
                match list_key_int {
                    Ok(x) => {
                        match NP_Tuple::select_to_ptr(cursor_addr, &memory, x as usize)? {
                            Some(y) => NP_Cursor::select_with_commit(y, memory, path, path_index + 1),
                            None => Ok(None)
                        }
                    },
                    Err(_e) => {
                        let mut err = String::from("Can't query tuple with string, need number! Path: \n");
                        err.push_str(print_path(&path, path_index).as_str());
                        Err(NP_Error::new(err))
                    }
                }

            },
            _ => { // scalar type
                
                Ok(Some(cursor_addr))
            }
        }
    }

    /// Get value at this address
    pub fn get_here<'get, T: 'get>(cursor_addr: NP_Cursor_Addr, memory: &'get NP_Memory<'get>) -> Result<Option<&'get *const T>, NP_Error> where T: Default + NP_Value<'get> {
        
        let schema_data = cursor_addr.get_schema_data(&memory);

        if schema_data.get_type_data().1 != T::type_idx().1 {
            return Err(NP_Error::new("typecast error!"))
        }
        match T::into_value(cursor_addr, &memory)? {
            Some(x) => {
                Ok(Some(x))
            },
            None => {
                Ok(T::schema_default(schema_data))
            }
        }
    }



    pub fn get_json<'json>(cursor_addr: NP_Cursor_Addr, memory: &'json NP_Memory<'json>, path: &'json [&str]) -> NP_JSON {
        
        match NP_Cursor::select(cursor_addr, memory.clone(), path, 0) {
            Ok(new_addr) => {
                if let Some(x) = new_addr {
                    NP_Cursor::json_encode(x, memory.clone())
                } else {
                    NP_JSON::Null
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    /// 
    pub fn json_encode<'json>(cursor_addr: NP_Cursor_Addr, memory: NP_Memory<'json>) -> NP_JSON {

        let cursor = cursor_addr.get_data(&memory);

        match cursor {
            Ok(data) => {
                if data.address_value == 0 {
                    return NP_JSON::Null;
                }

                match data.schema_data(&memory).get_type_key() {
                    NP_TypeKeys::None           => { NP_JSON::Null },
                    NP_TypeKeys::Any            => { NP_JSON::Null },
                    NP_TypeKeys::UTF8String     => {    String::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Bytes          => {  NP_Bytes::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Int8           => {        i8::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Int16          => {       i16::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Int32          => {       i32::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Int64          => {       i64::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Uint8          => {        u8::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Uint16         => {       u16::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Uint32         => {       u32::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Uint64         => {       u64::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Float          => {       f32::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Double         => {       f64::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Decimal        => {    NP_Dec::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Boolean        => {      bool::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Geo            => {    NP_Geo::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Uuid           => {   NP_UUID::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Ulid           => {   NP_ULID::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Date           => {   NP_Date::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Enum           => { NP_Option::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Table          => {  NP_Table::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Map            => {    NP_Map::to_json(cursor_addr, memory) },
                    NP_TypeKeys::List           => {   NP_List::to_json(cursor_addr, memory) },
                    NP_TypeKeys::Tuple          => {  NP_Tuple::to_json(cursor_addr, memory) }
                }

            },
            Err(_e) => return NP_JSON::Null
        }

    }

    pub fn compact<'comp>(from_cursor: NP_Cursor_Addr, from_memory: &'comp NP_Memory<'comp>, to_cursor: NP_Cursor_Addr, to_memory: &'comp NP_Memory<'comp>) -> Result<NP_Cursor_Addr, NP_Error> {

        let cursor = from_cursor.get_data(&from_memory);

        match cursor {
            Ok(data) => {
                if data.address_value == 0 {
                    return Ok(to_cursor)
                }

                match data.schema_data(&from_memory).get_type_key() {
                    NP_TypeKeys::Any           => { Ok(to_cursor) }
                    NP_TypeKeys::UTF8String    => {    String::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
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
                    NP_TypeKeys::Uuid          => {   NP_UUID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Ulid          => {   NP_ULID::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Date          => {   NP_Date::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Enum          => { NP_Option::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Table         => {  NP_Table::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Map           => {    NP_Map::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::List          => {   NP_List::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    NP_TypeKeys::Tuple         => {  NP_Tuple::do_compact(from_cursor, from_memory, to_cursor, to_memory) }
                    _ => { panic!() }
                }

            },
            Err(_e) => return Ok(to_cursor)
        }
    }

    pub fn set_default<'default>(cursor_addr: NP_Cursor_Addr, memory: &'default NP_Memory<'default>) -> Result<(), NP_Error> {

        let cursor = cursor_addr.get_data(&memory);

        match cursor {
            Ok(data) => {
                if data.address_value == 0 {
                    return Ok(())
                }

                match data.schema_data(&memory).get_type_key() {
                    NP_TypeKeys::None        => { },
                    NP_TypeKeys::Any         => { },
                    NP_TypeKeys::Table       => { },
                    NP_TypeKeys::Map         => { },
                    NP_TypeKeys::List        => { },
                    NP_TypeKeys::Tuple       => { },
                    NP_TypeKeys::UTF8String  => {     String::set_value(cursor_addr, memory, String::default())?; },
                    NP_TypeKeys::Bytes       => {   NP_Bytes::set_value(cursor_addr, memory, NP_Bytes::default())?; },
                    NP_TypeKeys::Int8        => {         i8::set_value(cursor_addr, memory, i8::default())?; },
                    NP_TypeKeys::Int16       => {        i16::set_value(cursor_addr, memory, i16::default())?; },
                    NP_TypeKeys::Int32       => {        i32::set_value(cursor_addr, memory, i32::default())?; },
                    NP_TypeKeys::Int64       => {        i64::set_value(cursor_addr, memory, i64::default())?; },
                    NP_TypeKeys::Uint8       => {         u8::set_value(cursor_addr, memory, u8::default())?; },
                    NP_TypeKeys::Uint16      => {        u16::set_value(cursor_addr, memory, u16::default())?; },
                    NP_TypeKeys::Uint32      => {        u32::set_value(cursor_addr, memory, u32::default())?; },
                    NP_TypeKeys::Uint64      => {        u64::set_value(cursor_addr, memory, u64::default())?; },
                    NP_TypeKeys::Float       => {        f32::set_value(cursor_addr, memory, f32::default())?; },
                    NP_TypeKeys::Double      => {        f64::set_value(cursor_addr, memory, f64::default())?; },
                    NP_TypeKeys::Decimal     => {     NP_Dec::set_value(cursor_addr, memory, NP_Dec::default())?; },
                    NP_TypeKeys::Boolean     => {       bool::set_value(cursor_addr, memory, bool::default())?; },
                    NP_TypeKeys::Geo         => {     NP_Geo::set_value(cursor_addr, memory, NP_Geo::default())?; },
                    NP_TypeKeys::Uuid        => {    NP_UUID::set_value(cursor_addr, memory, NP_UUID::default())?; },
                    NP_TypeKeys::Ulid        => {    NP_ULID::set_value(cursor_addr, memory, NP_ULID::default())?; },
                    NP_TypeKeys::Date        => {    NP_Date::set_value(cursor_addr, memory, NP_Date::default())?; },
                    NP_TypeKeys::Enum        => {  NP_Option::set_value(cursor_addr, memory, NP_Option::default())?; }
                }

            },
            Err(_e) => { }
        }
        Ok(())
    }

    /// Calculate the number of bytes used by this pointer and it's descendants.
    /// 
    pub fn calc_size<'size>(cursor_addr: NP_Cursor_Addr, memory: &'size NP_Memory<'size>) -> Result<usize, NP_Error> {

        let cursor = cursor_addr.get_data(&memory)?;

        // no pointer, no size
        if cursor.address == 0 {
            return Ok(0);
        }

        // size of pointer
        let base_size = memory.ptr_size(&cursor);

        // pointer is in buffer but has no value set
        if cursor.address_value == 0 { // no value, just base size
            return Ok(base_size);
        }

        // get the size of the value based on schema
        let type_size = match cursor.schema_data(&memory).get_type_key() {
            NP_TypeKeys::None         => { Ok(0) },
            NP_TypeKeys::Any          => { Ok(0) },
            NP_TypeKeys::UTF8String   => {    String::get_size(cursor_addr, memory) },
            NP_TypeKeys::Bytes        => {  NP_Bytes::get_size(cursor_addr, memory) },
            NP_TypeKeys::Int8         => {        i8::get_size(cursor_addr, memory) },
            NP_TypeKeys::Int16        => {       i16::get_size(cursor_addr, memory) },
            NP_TypeKeys::Int32        => {       i32::get_size(cursor_addr, memory) },
            NP_TypeKeys::Int64        => {       i64::get_size(cursor_addr, memory) },
            NP_TypeKeys::Uint8        => {        u8::get_size(cursor_addr, memory) },
            NP_TypeKeys::Uint16       => {       u16::get_size(cursor_addr, memory) },
            NP_TypeKeys::Uint32       => {       u32::get_size(cursor_addr, memory) },
            NP_TypeKeys::Uint64       => {       u64::get_size(cursor_addr, memory) },
            NP_TypeKeys::Float        => {       f32::get_size(cursor_addr, memory) },
            NP_TypeKeys::Double       => {       f64::get_size(cursor_addr, memory) },
            NP_TypeKeys::Decimal      => {    NP_Dec::get_size(cursor_addr, memory) },
            NP_TypeKeys::Boolean      => {      bool::get_size(cursor_addr, memory) },
            NP_TypeKeys::Geo          => {    NP_Geo::get_size(cursor_addr, memory) },
            NP_TypeKeys::Uuid         => {   NP_UUID::get_size(cursor_addr, memory) },
            NP_TypeKeys::Ulid         => {   NP_ULID::get_size(cursor_addr, memory) },
            NP_TypeKeys::Date         => {   NP_Date::get_size(cursor_addr, memory) },
            NP_TypeKeys::Enum         => { NP_Option::get_size(cursor_addr, memory) },
            NP_TypeKeys::Table        => {  NP_Table::get_size(cursor_addr, memory) },
            NP_TypeKeys::Map          => {    NP_Map::get_size(cursor_addr, memory) },
            NP_TypeKeys::List         => {   NP_List::get_size(cursor_addr, memory) },
            NP_TypeKeys::Tuple        => {  NP_Tuple::get_size(cursor_addr, memory) }
        }?;

        Ok(type_size + base_size)
    }
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
    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error>;

    /// Get the default schema value for this type
    /// 
    fn schema_default<'default>(_schema: &'default NP_Parsed_Schema) -> Option<&'default *const Self>;

    /// Parse JSON schema into schema
    ///
    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error>;

    /// Parse bytes into schema
    /// 
    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>>;

    /// Set the value of this scalar into the buffer
    /// 
    fn set_value<'set>(_cursor: NP_Cursor_Addr, _memory: &'set NP_Memory<'set>, _value: Self) -> Result<NP_Cursor_Addr, NP_Error> where Self: core::marker::Sized {
        let message = "This type doesn't support set_value!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value<'into>(_cursor: NP_Cursor_Addr, _memory: &'into NP_Memory<'into>) -> Result<Option<&'into *const Self>, NP_Error> {
        let message = "This type doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json<'json>(value: &'json *const Self) -> NP_JSON;

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size<'size>(_cursor: NP_Cursor_Addr, _memory: &'size NP_Memory<'size>) -> Result<usize, NP_Error>;
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact<'compact>(from_cursor: NP_Cursor_Addr, from_memory: &'compact NP_Memory<'compact>, to_cursor: NP_Cursor_Addr, to_memory: &'compact NP_Memory<'compact>) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'compact + Clone {

        match Self::into_value(from_cursor, from_memory)? {
            Some(x) => {
                return Self::set_value(to_cursor, to_memory, *x.clone());
            },
            None => { }
        }

        Ok(to_cursor)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NP_Cursor_Kinds {
    None,
    Standard,   // u32(4 bytes [4]), u16(2 bytes [2])

    Map,        // u32(4 bytes [4]), u16(2 bytes [2])
    Table,      // u32(4 bytes [4]), u16(2 bytes [2])
    List,       // u32(4 bytes [4]), u16(2 bytes [2])
    Tuple,      // u32(4 bytes [4]), u16(2 bytes [2])

    // collection items
    MapItem,    // [addr | next | key] u32(12 bytes  [4, 4, 4]),  u16(6 bytes [2, 2, 2]), 
    TableItem,  // [addr | next | i: u8]  u32(9  bytes  [4, 4, 1]),  u16(5 bytes [2, 2, 1]),   
    ListItem,   // [addr | next | i: u16] u32(10 bytes  [4, 4, 2]),  u16(6 bytes [2, 2, 2]),
    TupleItem   // [addr]u32(4 bytes  [4]),  u16(2 bytes [2])           
}



/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
*/