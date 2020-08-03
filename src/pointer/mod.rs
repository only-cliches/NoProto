//! All values in buffers are accessed and modified through pointers
//! 
//! NP_Ptr are the primary abstraction to read, update or delete values in a buffer.
//! Pointers should *never* be created directly, instead the various methods provided by the library to access
//! the internals of the buffer should be used.
//! 
//! Once you have a pointer you can read it's contents if it's a scalar value with '.get()` or convert it to a collection with `.into()`.
//! When you attempt to read, update, or convert a pointer the schema is checked for that pointer location.  If the schema conflicts with the operation you're attempting it will fail.
//! As a result, you should be careful to make sure your reads and updates to the buffer line up with the schema you provided.
//! 
//! 

pub mod misc;
pub mod string;
pub mod bytes;
pub mod any;
pub mod numbers;

use crate::json_flex::JFObject;
use crate::memory::NP_Memory;
use crate::NP_Error;
use crate::{schema::{NP_Schema, NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}, utils::print_path};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use bytes::NP_Bytes;
use misc::{NP_Geo, NP_Dec, NP_UUID, NP_ULID, NP_Date, NP_Option};
use any::NP_Any;

// stores the different kinds of pointers and the details for each pointer
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum NP_PtrKinds {
    None,
    // scalar / collection
    Standard  { value: u32 }, // 4 bytes [4]

    // collection items
    MapItem   { value: u32, next: u32, key: u32 },  // 12 bytes [4, 4, 4]
    TableItem { value: u32, next: u32, i: u8    },  // 9  bytes [4, 4, 1]
    ListItem  { value: u32, next: u32, i: u16   },  // 10 bytes [4, 4, 2]
}


impl NP_PtrKinds {

    /// Get the address of the value for this pointer
    pub fn get_value(&self) -> u32 {
        match self {
            NP_PtrKinds::None                                                => { 0 },
            NP_PtrKinds::Standard  { value } =>                      { *value },
            NP_PtrKinds::MapItem   { value, key: _,  next: _ } =>    { *value },
            NP_PtrKinds::TableItem { value, i: _,    next: _ } =>    { *value },
            NP_PtrKinds::ListItem  { value, i:_ ,    next: _ } =>    { *value }
        }
    }

    /// Get the size of this pointer based it's kind
    pub fn get_size(&self) -> u32 {
        match self {
            NP_PtrKinds::None                                     =>    { 0u32 },
            NP_PtrKinds::Standard  { value: _ }                   =>    { 4u32 },
            NP_PtrKinds::MapItem   { value: _, key: _,  next: _ } =>    { 12u32 },
            NP_PtrKinds::TableItem { value: _, i:_ ,    next: _ } =>    { 9u32 },
            NP_PtrKinds::ListItem  { value: _, i:_ ,    next: _ } =>    { 10u32 }
        }
    }
}

pub trait NP_Value {
    /// Check if a specific string "type" in the schema matches this data type
    /// 
    fn is_type(_type_str: &str) -> bool { false }

    /// Get the static type information for this type (static)
    /// 
    fn type_idx() -> (i64, String) { (-1, "null".to_owned()) }

    /// Get the static type information for this type (instance)
    /// 
    fn self_type_idx(&self) -> (i64, String) { (-1, "null".to_owned()) }

    /// Called for each declaration in the schema for a given type, useful for storing configuration details about the schema
    /// 
    fn schema_state(_type_string: &str, _json_schema: &JFObject) -> core::result::Result<i64, NP_Error> { Ok(0) }

    /// Set the pointer to a given scalar value, does not work for collections
    /// 
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        let mut message = "This type (".to_owned();
        message.push_str(Self::type_idx().1.as_str());
        message.push_str(") doesn't support .set()!");
        Err(NP_Error::new(message.as_str()))
    }

    fn v_buffer_into(_address: u32, _kind: NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let message = "This type  doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }
}

pub trait NP_ValueInto<'a> {

    /// Convert pointer into underlying value
    /// 
    fn buffer_into(_address: u32, _kind: NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let message = "This type  doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this pointer into a JSON value (recursive for collections)
    /// 
    fn buffer_to_json(_address: u32, _kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> JFObject {
         JFObject::Null
    }

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn buffer_get_size(_address: u32, _kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
         Err(NP_Error::new("Size not supported for this type!"))
    }
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn buffer_do_compact<X: NP_Value + Default + NP_ValueInto<'a>>(from_ptr: &NP_Ptr<'a, X>, to_ptr: NP_Ptr<'a, NP_Any>) -> Result<(u32, NP_PtrKinds, &'a NP_Schema), NP_Error> where Self: NP_Value + Default {
        if from_ptr.location == 0 {
            return Ok((0, from_ptr.kind, from_ptr.schema));
        }

        match Self::buffer_into(from_ptr.location, from_ptr.kind, from_ptr.schema, from_ptr.memory)? {
            Some(x) => {
                Self::buffer_set(to_ptr.location, &to_ptr.kind, to_ptr.schema, to_ptr.memory, Box::new(&*x))?;
                return Ok((to_ptr.location, to_ptr.kind, to_ptr.schema));
            },
            None => { }
        }

        Ok((0, from_ptr.kind, from_ptr.schema))
    }
}

/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
/// 
/// # Using Scalar Types with Pointers
/// 
/// # Using Collection Types with Pointers
/// 
#[derive(Debug)]
pub struct NP_Ptr<'a, T: NP_Value + Default + NP_ValueInto<'a>> {
    pub location: u32, // pointer address in buffer
    pub kind: NP_PtrKinds, // the kind of pointer this is (standard, list item, map item, etc).  Includes value address
    pub memory: &'a NP_Memory, // the underlying buffer this pointer is a part of
    pub schema: &'a NP_Schema, // schema stores the *actual* schema data for this pointer, regardless of type casting
    pub value: T // a static invocation of the pointer type
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_Ptr<'a, T> {

    pub fn get(&mut self) -> core::result::Result<Option<T>, NP_Error> {

        let value = T::buffer_into(self.location, self.kind, self.schema, &self.memory)?;
        
        Ok(match value {
            Some (x) => {
                Some(*x)
            },
            None => None
        })
    }

    pub fn set(&mut self, value: T) -> core::result::Result<(), NP_Error> {
        self.kind = T::buffer_set(self.location, &self.kind, self.schema, &self.memory, Box::new(&value))?;
        Ok(())
    }

    #[doc(hidden)]
    pub fn new_standard_ptr(location: u32, schema: &'a NP_Schema, memory: &'a NP_Memory) -> Self {

        let addr = location as usize;
        let value: [u8; 4] = *memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
        
        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::Standard { value: u32::from_be_bytes(value) },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn new_table_item_ptr(location: u32, schema: &'a NP_Schema, memory: &'a NP_Memory) -> Self {

        let addr = location as usize;
        let b_bytes = &memory.read_bytes();

        let value: [u8; 4] = *memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
        let next: [u8; 4] = *memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);
        let index: u8 = b_bytes[addr + 8];

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::TableItem { 
                value: u32::from_be_bytes(value),
                next: u32::from_be_bytes(next),
                i: index
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn new_map_item_ptr(location: u32, schema: &'a NP_Schema, memory: &'a NP_Memory) -> Self {

        let addr = location as usize;
        let value: [u8; 4] = *memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
        let next: [u8; 4] = *memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);
        let key: [u8; 4] = *memory.get_4_bytes(addr + 8).unwrap_or(&[0; 4]);

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::MapItem { 
                value: u32::from_be_bytes(value),
                next: u32::from_be_bytes(next),
                key: u32::from_be_bytes(key)
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn new_list_item_ptr(location: u32, schema: &'a NP_Schema, memory: &'a NP_Memory) -> Self {

        let addr = location as usize;
        let value: [u8; 4] = *memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
        let next: [u8; 4] = *memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);
        let index: [u8; 2] = *memory.get_2_bytes(addr + 8).unwrap_or(&[0; 2]);

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::ListItem { 
                value: u32::from_be_bytes(value),
                next: u32::from_be_bytes(next),
                i: u16::from_be_bytes(index)
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    pub fn has_value(&self) -> bool {

        if self.kind.get_value() == 0 {
            return false;
        }

        return true;
    }

    pub fn clear(self) -> core::result::Result<NP_Ptr<'a, T>, NP_Error> {
        Ok(NP_Ptr {
            location: self.location,
            kind: self.memory.set_value_address(self.location, 0, &self.kind),
            memory: self.memory,
            schema: self.schema,
            value: self.value
        })
    }

    pub fn into(self) -> core::result::Result<Option<T>, NP_Error> {

        // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && self.schema.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using ANY casting, check type
            if self.schema.type_data.0 != T::type_idx().0 {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(T::type_idx().1.as_str());
                err.push_str(") into schema of type (");
                err.push_str(self.schema.type_data.1.as_str());
                err.push_str(")");
                return Err(NP_Error::new(err));
            }
        }
        
        let result = T::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

        Ok(match result {
            Some(x) => Some(*x),
            None => None
        })
    }
/*
    #[doc(hidden)]
    /// Used to create a copy of a pointer for compaction
    /// Should never be called or used by a library end user.
    pub fn _clone_ptr(schema: &'a NP_Schema, kind: NP_PtrKinds, memory: &'a NP_Memory) -> Result<NP_Ptr<'a, T>, NP_Error> {

        let pointer_size = kind.get_size();

        let mut new_pointer_bytes = Vec::with_capacity(pointer_size as usize);

        for _x in 0..pointer_size {
            new_pointer_bytes.push(0);
        }

        let location = memory.malloc(new_pointer_bytes)?;
        
        Ok(match kind {
            NP_PtrKinds::None                                     => {
                unreachable!()
            },
            NP_PtrKinds::Standard  { value: _ }                   => {
                NP_Ptr::new_standard_ptr(location, schema, memory)
            },
            NP_PtrKinds::MapItem   { value: _, key: _,  next: _ } => {
                NP_Ptr::new_map_item_ptr(location, schema, memory)
            },
            NP_PtrKinds::TableItem { value: _, i:_ ,    next: _ } => {
                NP_Ptr::new_table_item_ptr(location, schema, memory)
            },
            NP_PtrKinds::ListItem  { value: _, i:_ ,    next: _ } => {
                NP_Ptr::new_list_item_ptr(location, schema, memory)
            }
        })
    }
*/
    #[doc(hidden)]
    pub fn _deep_set<X: NP_Value + Default>(self, path: Vec<&str>, path_index: usize, value: X) -> Result<(), NP_Error> {

        if path.len() > path_index + 1 {
            let mut err = "Error in deep set, ran off the end of the path. \n Path:".to_owned();
            err.push_str(print_path(&path, path_index).as_str());
            return Err(NP_Error::new(err));
        }

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Table => {

                let result = NP_Table::buffer_into(self.location, self.kind, self.schema, &self.memory)?;
                
                match result {
                    Some(mut table) => {
                        let table_key = path[path_index];
                        let col = table.select::<NP_Any>(table_key)?;
                        col._deep_set::<X>(path, path_index + 1, value)?;
                    },
                    None => {
                        unreachable!();
                    }
                }

                Ok(())
            },
            NP_TypeKeys::Map => {

                let result = NP_Map::<NP_Any>::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(mut map) => {
                        let map_key = path[path_index];
                        let col = map.select(&map_key.as_bytes().to_vec())?;
                        col._deep_set::<X>(path, path_index + 1, value)?;
                    },
                    None => {
                        unreachable!();
                    }
                }
                
                Ok(())
            },
            NP_TypeKeys::List => {
                let result = NP_List::<NP_Any>::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(mut list) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u16>()?;
                        let col = list.select(list_key_int)?;
                        col._deep_set::<X>(path, path_index + 1, value)?;
                    },
                    None => {
                        unreachable!();
                    }
                }
                Ok(())
            },
            NP_TypeKeys::Tuple => {
                let result = NP_Tuple::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(tuple) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u8>()?;
                        let col = tuple.select::<NP_Any>(list_key_int)?;
                        col._deep_set::<X>(path, path_index + 1, value)?;
                    },
                    None => {
                        unreachable!();
                    }
                }
                Ok(())
            },
            _ => { // scalar type
                if path.len() - 1 != path_index { // reached scalar value but not at end of path
                    let mut err = "TypeError: Attempted to deep set into collection but found scalar type (".to_owned();
                    err.push_str(self.schema.type_data.1.as_str());
                    err.push_str(")\n Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }    

                // if schema is ANY then allow any type to set this value
                // otherwise make sure the schema and type match
                if self.schema.type_data.0 != NP_Any::type_idx().0 && self.schema.type_data.0 != X::type_idx().0 {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().1.as_str());
                    err.push_str(") into schema of type (");
                    err.push_str(self.schema.type_data.1.as_str());
                    err.push_str(")\n Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }

                X::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&value))?;

                Ok(())
            }
        }
    }

    #[doc(hidden)]
    pub fn _deep_get<X: NP_Value + Default>(self, path: Vec<&str>, path_index: usize) -> Result<Option<Box<X>>, NP_Error> {

        if path.len() > path_index + 1 {
            let mut err = "Error in deep get, ran off the end of the path. \n Path:".to_owned();
            err.push_str(print_path(&path, path_index).as_str());
            return Err(NP_Error::new(err));
        }

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Table => {

                let result = NP_Table::buffer_into(self.location, self.kind, self.schema, &self.memory)?;
                
                match result {
                    Some(mut table) => {
                        let table_key = path[path_index];
                        let col = table.select::<NP_Any>(table_key)?;
                        col._deep_get::<X>(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Map => {

                let result = NP_Map::<NP_Any>::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(mut map) => {
                        let map_key = path[path_index];
                        let col = map.select(&map_key.as_bytes().to_vec())?;
                        col._deep_get::<X>(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::List => {
                let result = NP_List::<NP_Any>::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(mut list) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u16>()?;
                        let col = list.select(list_key_int)?;
                        col._deep_get::<X>(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Tuple => {
                let result = NP_Tuple::buffer_into(self.location, self.kind, self.schema, &self.memory)?;

                match result {
                    Some(tuple) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u8>()?;
                        let col = tuple.select::<NP_Any>(list_key_int)?;
                        col._deep_get::<X>(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }

            },
            _ => { // scalar type
                if path.len() - 1 != path_index { // reached scalar value but not at end of path
                    let mut err = "TypeError: Attempted to deep get into collection but found scalar type (".to_owned();
                    err.push_str(self.schema.type_data.1.as_str());
                    err.push_str(")\n Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }    

                // if schema is ANY then allow any type to set this value
                // otherwise make sure the schema and type match
                if self.schema.type_data.0 != X::type_idx().0 {
                    let mut err = "TypeError: Attempted to get value for type (".to_owned();
                    err.push_str(X::type_idx().1.as_str());
                    err.push_str(") from schema of type (");
                    err.push_str(self.schema.type_data.1.as_str());
                    err.push_str(")\n Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }

                X::v_buffer_into(self.location, self.kind, self.schema, self.memory)
            }
        }

        // Ok(None)
    }
    
    pub fn set_default(&self) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => {
                
            },
            NP_TypeKeys::UTF8String => {
                String::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&String::default()))?;
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Bytes::default()))?;
            },
            NP_TypeKeys::Int8 => {
                i8::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&i8::default()))?;
            },
            NP_TypeKeys::Int16 => {
                i16::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&i16::default()))?;
            },
            NP_TypeKeys::Int32 => {
                i32::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&i32::default()))?;
            },
            NP_TypeKeys::Int64 => {
                i64::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&i64::default()))?;
            },
            NP_TypeKeys::Uint8 => {
                u8::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&u8::default()))?;
            },
            NP_TypeKeys::Uint16 => {
                u16::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&u16::default()))?;
            },
            NP_TypeKeys::Uint32 => {
                u32::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&u32::default()))?;
            },
            NP_TypeKeys::Uint64 => {
                u64::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&u64::default()))?;
            },
            NP_TypeKeys::Float => {
                f32::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&f32::default()))?;
            },
            NP_TypeKeys::Double => {
                f64::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&f64::default()))?;
            },
            NP_TypeKeys::Dec64 => {
                NP_Dec::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Dec::default()))?;
            },
            NP_TypeKeys::Boolean => {
                bool::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&bool::default()))?;
            },
            NP_TypeKeys::Geo => {
                NP_Geo::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Geo::default()))?;
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_UUID::default()))?;
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_ULID::default()))?;
            },
            NP_TypeKeys::Date => {
                NP_Date::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Date::default()))?;
            },
            NP_TypeKeys::Enum => {
                NP_Option::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Option::default()))?;
            },
            NP_TypeKeys::Table => {
                NP_Table::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Table::default()))?;
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Map::default()))?;
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_List::default()))?;
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::buffer_set(self.location, &self.kind, self.schema, self.memory, Box::new(&NP_Tuple::default()))?;
            }
        };

        Ok(())
    }

    #[doc(hidden)]
    /// used to run compaction on this pointer
    /// should not be called directly by the library user
    /// Use NP_Factory methods of `compact` and `maybe_compact`.
    pub fn _compact(&self, copy_to: NP_Ptr<'a, NP_Any>) -> Result<(u32, NP_PtrKinds, &'a NP_Schema), NP_Error> {

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => {
                Ok((0, self.kind.clone(), self.schema))
            },
            NP_TypeKeys::UTF8String => {
                String::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Int8 => {
                i8::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Int16 => {
                i16::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Int32 => {
                i32::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Int64 => {
                i64::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint8 => {
                u8::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint16 => {
                u16::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint32 => {
                u32::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint64 => {
                u64::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Float => {
                f32::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Double => {
                f64::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Dec64 => {
                NP_Dec::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Boolean => {
                bool::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Date => {
                NP_Date::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Enum => {
                NP_Option::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Table => {
                NP_Table::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::buffer_do_compact(self, copy_to)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::buffer_do_compact(self, copy_to)
            }
        }
    }


    pub fn calc_size(&self) -> Result<u32, NP_Error> {

        let base_size = self.kind.get_size();

        if self.location == 0 { // no value, just base size
            return Ok(base_size);
        }

        let type_size = match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => {
                Ok(0)
            },
            NP_TypeKeys::UTF8String => {
                String::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int8 => {
                i8::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int16 => {
                i16::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int32 => {
                i32::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int64 => {
                i64::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint8 => {
                u8::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint16 => {
                u16::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint32 => {
                u32::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint64 => {
                u64::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Float => {
                f32::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Double => {
                f64::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Dec64 => {
                NP_Dec::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Boolean => {
                bool::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Date => {
                NP_Date::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Enum => {
                NP_Option::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Table => {
                NP_Table::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::buffer_get_size(self.location, &self.kind, self.schema, self.memory)
            }
        }?;

        Ok(type_size + base_size)
    }


    pub fn json_encode(&self) -> JFObject {
        if self.location == 0 {
            return JFObject::Null;
        }

        let type_key = NP_TypeKeys::from(self.schema.type_data.0);

        match type_key {
            NP_TypeKeys::Any => {
                JFObject::Null
            },
            NP_TypeKeys::UTF8String => {
                String::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int8 => {
                i8::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int16 => {
                i16::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int32 => {
                i32::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Int64 => {
                i64::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint8 => {
                u8::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint16 => {
                u16::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint32 => {
                u32::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uint64 => {
                u64::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Float => {
                f32::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Double => {
                f64::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Dec64 => {
                NP_Dec::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Boolean => {
                bool::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Date => {
                NP_Date::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Enum => {
                NP_Option::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Table => {
                NP_Table::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::buffer_to_json(self.location, &self.kind, self.schema, self.memory)
            }
        }
    }

}


/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
pub enum NP_DataType {
    none,
    /*table {
        head: u32
    },
    map {
        head: u32
    },
    list {
        head: u32,
        tail: u32,
        size: u16
    },
    tuple {
        head: u32
    },*/
    utf8_string { size: u32, value: String },
    bytes { size: u32, value: Vec<u8> },
    int8 { value: i8 },
    int16 { value: i16 },
    int32 { value: i32 },
    int64 { value: i64 }, 
    uint8 { value: u8 },
    uint16 { value: u16 },
    uint32 { value: u32 },
    uint64 { value: u64 },
    float { value: f32 }, // -3.4E+38 to +3.4E+38
    double { value: f64 }, // -1.7E+308 to +1.7E+308
    option { value: u8 }, // enum
    dec32 { value: i32, expo: i8},
    dec64 { value: i64, exp: i8},
    boolean { value: bool },
    geo_16 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_8 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_4 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: String }, // 16 bytes 21,267,647,932,558,653,966,460,912,964,485,513,216 possibilities (255^15 * 16) or over 2 quadrillion times more possibilites than stars in the universe
    time_id { id: String, time: u64 }, // 8 + 8 bytes
    date { value: u64 } // 8 bytes  
}*/

// Pointer -> String
/*impl From<&NP_Ptr> for core::result::Result<String> {
    fn from(ptr: &NP_Ptr) -> core::result::Result<String> {
        ptr.to_string()
    }
}*/

/*
// cast i64 => Pointer
impl From<i64> for NP_Value {
    fn from(num: i64) -> Self {
        NP_Value {
            loaded: false,
            address: 0,
            value: NP_Value::int64 { value: num },
            // model: None
        }
    }
}

// cast Pointer => core::result::Result<i64>
impl From<&NP_Value> for core::result::Result<i64> {
    fn from(ptr: &NP_Value) -> core::result::Result<i64> {
        match ptr.value {
            NP_Value::int64 { value } => {
                Some(value)
            }
            _ => None
        }
    }
}*/