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

use core::hint::unreachable_unchecked;
use crate::{collection::NP_Collection, pointer::dec::NP_Dec};
use crate::NP_Parsed_Schema;
use crate::{json_flex::NP_JSON};
use crate::memory::{NP_Size, NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}, utils::{overflow_error, print_path}};

use alloc::{boxed::Box, string::String, vec::Vec, borrow::ToOwned};
use bytes::NP_Bytes;
use any::NP_Any;

use self::{date::NP_Date, geo::NP_Geo, option::NP_Option, ulid::NP_ULID, uuid::NP_UUID};

// stores the different kinds of pointers and the details for each pointer
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum NP_PtrKinds {
    None,
    // scalar / collection
    Standard  { addr: usize },                    // u32(4 bytes [4]), u16(2 bytes [2])

    // collection items
    MapItem   { 
        addr: usize, next: usize, key: usize      // u32(12 bytes  [4, 4, 4]),  u16(6 bytes [2, 2, 2])
    }, 
    TableItem { 
        addr: usize, next: usize, i: u8           // u32(9  bytes  [4, 4, 1]),  u16(5 bytes [2, 2, 1])
    },   
    ListItem  { 
        addr: usize, next: usize, i: u16          // u32(10 bytes  [4, 4, 2]),  u16(6 bytes [2, 2, 2]),
    },   
    TupleItem  { 
        addr: usize, i: u8                        // u32(4 bytes  [4]),  u16(2 bytes [2])
    },                
}


impl NP_PtrKinds {

    /// Get the address of the value for this pointer
    pub fn get_value_addr(&self) -> usize {
        match self {
            NP_PtrKinds::None                                        =>    { 0 },
            NP_PtrKinds::Standard  { addr }                   =>    { *addr },
            NP_PtrKinds::MapItem   { addr, key: _,  next: _ } =>    { *addr },
            NP_PtrKinds::TableItem { addr, i: _,    next: _ } =>    { *addr },
            NP_PtrKinds::ListItem  { addr, i:_ ,    next: _ } =>    { *addr },
            NP_PtrKinds::TupleItem  { addr, i:_  }            =>    { *addr }
        }
    }
}



/// This trait is used to implement types as NoProto buffer types.
/// This includes all the type data, encoding and decoding methods.
pub trait NP_Value<'value> {

    /// Get the type information for this type (static)
    /// 
    fn type_idx() -> (u8, String, NP_TypeKeys);

    /// Get the type information for this type (instance)
    /// 
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys);

    /// Convert the schema byte array for this type into JSON
    /// 
    fn schema_to_json(_schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error>;

    /// Set the value of this scalar into the buffer
    /// 
    fn set_value(_pointer: &mut NP_Ptr<'value>, _value: Box<&Self>) -> Result<(), NP_Error>  where Self: NP_Value<'value> {
        let mut message = "This type (".to_owned();
        // message.push_str();
        message.push_str(") doesn't support .set()!");
        Err(NP_Error::new(message.as_str()))
    }

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value(_pointer: NP_Ptr<'value>) -> Result<Option<Box<Self>>, NP_Error> where Self: NP_Value<'value> {
        let message = "This type  doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json(_pointer: &'value NP_Ptr<'value>) -> NP_JSON;

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size(_pointer: &'value NP_Ptr<'value>) -> Result<usize, NP_Error>;
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact(from_ptr: NP_Ptr<'value>, to_ptr: &'value mut NP_Ptr<'value>) -> Result<(), NP_Error> where Self: NP_Value<'value> {

        match Self::into_value(from_ptr)? {
            Some(x) => {
                Self::set_value(to_ptr, Box::new(&*x))?;
            },
            None => { }
        }

        Ok(())
    }

    /// Get the default schema value for this type
    /// 
    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>>;

    /// Parse JSON schema into schema
    ///
    fn from_json_to_schema(_json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error>;

    /// Parse bytes into schema
    /// 
    fn from_bytes_to_schema(_address: usize, _bytes: &Vec<u8>) -> NP_Parsed_Schema;
}

/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 16 or 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
///  
/// 
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Ptr<'ptr> {
    /// the kind of pointer this is (standard, list item, map item, etc).  Includes value address
    pub kind: NP_PtrKinds, 
    /// schema stores the *actual* schema data for this pointer, regardless of type casting
    pub schema: &'ptr Box<NP_Parsed_Schema>, 
    /// pointer address in buffer 
    pub address: usize,
    /// the underlying buffer this pointer is a part of
    pub memory: &'ptr NP_Memory,
    /// If this is a collection pointer, data about it's parent is here
    pub parent: NP_Ptr_Collection<'ptr>,
    /// If this is a collection pointer, more data about the location of this pointer
    pub helper: NP_Iterator_Helper<'ptr>
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum NP_Ptr_Collection<'coll> {
    None,
    List { address: usize, head: usize, tail: usize },
    Map { address: usize, head: usize, length: u16 },
    Table { address: usize, head: usize, schema: &'coll Box<NP_Parsed_Schema> },
    Tuple { address: usize, length: usize, schema: &'coll Box<NP_Parsed_Schema> }
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum NP_Iterator_Helper<'it> {
    None,
    List  { index: u16, prev_addr: usize, next_addr: usize, next_index: u16 },
    Table { index: u8, column: &'it str, prev_addr: usize, skip_step: bool },
    Map   { key_addr: usize , prev_addr: usize, key: Option<String> },
    Tuple { index: u8 }
}

impl<'it> NP_Iterator_Helper <'it> {
    /// Clone iterator helper
    pub fn clone(&self) -> Self {
        match self {
            NP_Iterator_Helper::None => NP_Iterator_Helper::None,
            NP_Iterator_Helper::List { index, prev_addr, next_index, next_addr} => {
                NP_Iterator_Helper::List { index: *index, prev_addr: *prev_addr, next_index: *next_index, next_addr: *next_addr}
            },
            NP_Iterator_Helper::Table { index, column, prev_addr, skip_step} => {
                NP_Iterator_Helper::Table { index: *index, column: *column, prev_addr: *prev_addr, skip_step: *skip_step}
            },
            NP_Iterator_Helper::Map { key_addr, prev_addr, key} => {
                NP_Iterator_Helper::Map { key_addr: *key_addr, prev_addr: *prev_addr, key: key.clone()}
            },
            NP_Iterator_Helper::Tuple { index } => {
                NP_Iterator_Helper::Tuple { index: *index }
            }
        }
    }
}

impl<'ptr> NP_Ptr<'ptr> {

    /// Retrieves the value at this pointer, only useful for scalar values (not collections).
    pub fn get_here<T>(&self) -> Result<Option<T>, NP_Error> where T: Default + NP_Value<'ptr> {
        
        Ok(match T::into_value(self.clone())? {
            Some (x) => {
                Some(*x)
            },
            None => {
                match T::schema_default(&self.schema) {
                    Some(x) => Some(*x),
                    None => None
                }
            }
        })
    }  

    /// Clone this pointer
    pub fn clone(&self) -> Self {
        NP_Ptr {
            kind: self.kind,
            schema: self.schema,
            address: self.address,
            memory: self.memory,
            parent: self.parent.clone(),
            helper: self.helper.clone()
        }
    }

    /// Sets the value for this pointer, only works for scalar types (not collection types).
    pub fn set_here<T>(&'ptr mut self, value: T) -> Result<(), NP_Error> where T: NP_Value<'ptr> {
        T::set_value(self, Box::new(&value))
    }

    /// Create new standard pointer
    pub fn _new_standard_ptr(address: usize, schema: &'ptr Box<NP_Parsed_Schema>, memory: &'ptr NP_Memory) -> Self {

        NP_Ptr {
            address: address,
            kind: NP_PtrKinds::Standard { addr: memory.read_address(address) },
            memory: memory,
            schema: schema,
            parent: NP_Ptr_Collection::None,
            helper: NP_Iterator_Helper::None
        }
    }

    /// Create new collection item pointer
    pub fn _new_collection_item_ptr(address: usize, schema: &'ptr Box<NP_Parsed_Schema>, memory: &'ptr NP_Memory, parent: NP_Ptr_Collection<'ptr>, helper: NP_Iterator_Helper<'ptr>) -> Self {
        let b_bytes = &memory.read_bytes();

        NP_Ptr {
            address: address,
            kind: match parent {
                NP_Ptr_Collection::Table { address: _, head: _, schema: _} => {
                    NP_PtrKinds::TableItem { 
                        addr:  memory.read_address(address),
                        next:  memory.read_address_offset(address, 4, 2, 1),
                        i: if address == 0 { 0 } else { match &memory.size {
                            NP_Size::U32 => b_bytes[address + 8],
                            NP_Size::U16 => b_bytes[address + 4],
                            NP_Size::U8 => b_bytes[address + 2]
                        }},
                    }
                },
                NP_Ptr_Collection::List { address: _, head: _, tail: _} => {
                    NP_PtrKinds::ListItem { 
                        addr:  memory.read_address(address),
                        next:  memory.read_address_offset(address,  4, 2, 1),
                        i: if address == 0 { 0 } else { match &memory.size {
                            NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(address + 8).unwrap_or(&[0; 2])),
                            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(address + 4).unwrap_or(&[0; 2])),
                            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(address + 2).unwrap_or(0)]) as u16
                        }}
                    }
                },
                NP_Ptr_Collection::Map { address: _, head: _, length: _} => {
                    NP_PtrKinds::MapItem { 
                        addr:  memory.read_address(address),
                        next:  memory.read_address_offset(address,  4, 2, 1),
                        key:   memory.read_address_offset(address, 8, 4, 2)
                    }
                },
                _ => { panic!() }
            },
            memory: memory,
            schema: schema,
            parent,
            helper
        }
    }


    /// Check if there is any value set at this pointer
    pub fn has_value(&self) -> bool {

        if self.address == 0 || self.kind.get_value_addr() == 0 {
            return false;
        }

        return true;
    }


    /// Clear / delete the value at this pointer.  This is just clears the value address, so it doesn't actually remove items from the buffer.  Also if this is called on a collection type, all children of the collection will also be cleared.
    /// 
    /// If you clear a large object it's probably a good idea to run compaction to recover the free space.
    /// 
    pub fn clear_here(&self) -> bool {
        if self.address != 0 {
            self.memory.set_value_address(self.address, 0, &self.kind);
            true
        } else {
            false
        }
    }

    /// Deep delete a value
    pub fn _deep_delete(self, path: Vec<&str>, path_index: usize) -> Result<bool, NP_Error> {

        if let Some(x) = self._deep_get(path, path_index)? {
            Ok(x.clear_here())
        } else {
            Ok(false)
        }
    }

    /// Create a path to a pointer and provide the pointer
    /// 
    #[allow(unused_mut)]
    pub fn _deep_set(mut self, path: Vec<String>, path_index: usize) -> Result<NP_Ptr<'ptr>, NP_Error> {

        if path.len() == path_index {
            return Ok(self);
        }

        let type_data = self.schema.into_type_data();

        match type_data.2 {
            NP_TypeKeys::Table => {

                overflow_error("deep set", &path, path_index)?;

                let result = NP_Table::into_value(self)?;
                
                match result {
                    Some(table) => {
                        let table_key = &path[path_index];
                        let mut col = table.select_mv(table_key, None);
                        if col.has_value() == false {
                            col = NP_Table::commit_pointer(col)?;
                        }
                        return col._deep_set(path, path_index + 1);
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Map => {

                overflow_error("deep set", &path, path_index)?;

                match NP_Map::into_value(self)? {
                    Some(map) => {
                        let map_key = String::from(&path[path_index]);
                        let mut col = map.select_mv(map_key, false);
                        if col.has_value() == false {
                            col = NP_Map::commit_pointer(col)?;
                        }
                        return col._deep_set(path, path_index + 1);
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::List => {

                overflow_error("deep set", &path, path_index)?;

                let temp_list = NP_List::into_value(self)?;

                match temp_list {
                    Some(list)=> {
                        let list_key = &path[path_index];
                        let list_key_int = list_key.parse::<u16>();
                        match list_key_int {
                            Ok(x) => {
                                let mut col = list.select_mv(x);
                                if col.has_value() == false {
                                    col = NP_List::commit_pointer(col)?;
                                }
                                return col._deep_set(path, path_index + 1);
                            },
                            Err(_e) => {
                                return Err(NP_Error::new("Can't query list with string, need number!".to_owned()))
                            }
                        }
                    
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Tuple => {

                overflow_error("deep set", &path, path_index)?;

                let temp_tuple = NP_Tuple::into_value(self)?;

                match temp_tuple {
                    Some(tuple) => {
                        let list_key = &path[path_index];
                        let list_key_int = list_key.parse::<u8>();
                        match list_key_int {
                            Ok(x) => {
                                let col = tuple.select_mv(x)?;
                                return col._deep_set(path, path_index + 1);
                            },
                            Err(_e) => {
                                return Err(NP_Error::new("Can't query tuple with string, need number!".to_owned()))
                            }
                        }

                    },
                    None => {
                        unreachable!();
                    }
                }

            },
            _ => { // scalar type

       
                if path.len() != path_index { // reached scalar value but not at end of path
                    let mut err = "TypeError: Attempted to deep set into collection but found scalar type (".to_owned();
                    err.push_str(type_data.1.as_str());
                    err.push_str(")\n Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }
                
                return Ok(self)
            }
        }
    }

    /// Deep set a value
    #[allow(unused_mut)]
    pub fn _deep_set_value<X>(mut self, path: Vec<String>, path_index: usize, value: X) -> Result<(), NP_Error> where X: NP_Value<'ptr> + Default {

        let mut pointer_value = self._deep_set(path, path_index)?;

        let type_data = pointer_value.schema.into_type_data();

        // if schema is ANY then allow any type to set this value
        // otherwise make sure the schema and type match
        if type_data.0 != NP_Any::type_idx().0 && type_data.0 != X::type_idx().0 {
            let mut err = "TypeError: Attempted to set value for type (".to_owned();
            err.push_str(X::type_idx().1.as_str());
            err.push_str(") into schema of type (");
            err.push_str(type_data.1.as_str());
            err.push_str(")\n");
            return Err(NP_Error::new(err));
        }

        X::set_value(&mut pointer_value, Box::new(&value))?;

        Ok(())

    }

    /// deep get with type
    pub fn _deep_get_type<X: NP_Value<'ptr> + Default>(self, path: Vec<&str>, path_index: usize) -> Result<Option<Box<X>>, NP_Error> {
        let ptr = self._deep_get(path, path_index)?;

        if let Some(x) = ptr {
            if x.schema.into_type_data().0 != X::type_idx().0 {
                let mut err = "TypeError: Attempted to set value for type (".to_owned();
                err.push_str(X::type_idx().1.as_str());
                err.push_str(") into schema of type (");
                err.push_str(x.schema.into_type_data().1.as_str());
                err.push_str(")\n");
                return Err(NP_Error::new(err));
            }
            if x.has_value() {
                X::into_value(x)
            } else {
                Ok(X::schema_default(x.schema))
            }
        } else {
            Ok(None)
        }
    }

    /// Deep get a value
    pub fn _deep_get(self, path: Vec<&str>, path_index: usize) -> Result<Option<NP_Ptr<'ptr>>, NP_Error> {

        if path.len() == path_index {
            return Ok(Some(self));
        }

        let type_data = self.schema.into_type_data();

        match type_data.2 {
            NP_TypeKeys::Table => {

                let result = NP_Table::into_value(self)?;
                
                match result {
                    Some(table) => {
                        let table_key = &path[path_index];
                        let col = table.clone().select_mv(table_key, None);
                        if col.has_value() == false { 
                            match &**table.get_schema() {
                                NP_Parsed_Schema::Table { sortable: _, i: _, columns} => {
                                    for schem in columns {
                                        if schem.1 == *table_key {
                                            let ptr = NP_Ptr::_new_standard_ptr(0, &schem.2, col.memory);
                                            return ptr._deep_get(path, path_index + 1);
                                        }
                                    }
                                },
                                _ => { unsafe { unreachable_unchecked() } }
                            }
                        }
                        col._deep_get(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Map => {

                let result = NP_Map::into_value(self)?;

                match result {
                    Some(map) => {
                        let map_key = String::from(path[path_index]);
                        let col = map.clone().select_mv(map_key, false);
                        if col.has_value() == false {
                            match &**map.get_schema() {
                                NP_Parsed_Schema::Map { sortable: _, i: _, value} => {
                                    let ptr = NP_Ptr::_new_standard_ptr(0, value, col.memory);
                                    return ptr._deep_get(path, path_index + 1);
                                },
                                _ => { unsafe { unreachable_unchecked() } }
                            }
                        }
                        col._deep_get(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::List => {


                let result = NP_List::into_value(self)?;

                match result {
                    Some(list) => {
            
                        let list_key = &path[path_index];
                        let list_key_int = list_key.parse::<u16>();
                        match list_key_int {
                            Ok(x) => {
                                let col = list.clone().select_mv(x);
                                if col.has_value() == false {
                                    match &**list.get_schema() {
                                        NP_Parsed_Schema::List { sortable: _, i: _, of} => {
                                            let ptr = NP_Ptr::_new_standard_ptr(0, of, col.memory);
                                            return ptr._deep_get(path, path_index + 1);
                                        },
                                        _ => { unsafe { unreachable_unchecked() } }
                                    }
                                }
                                col._deep_get(path, path_index + 1)
                            },
                            Err(_e) => {
                                Err(NP_Error::new("Can't query list with string, need number!".to_owned()))
                            }
                        }
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Tuple => {

                let result = NP_Tuple::into_value(self.clone())?;

                match result {
                    Some(tuple) => {
                        let list_key = &path[path_index];
                        let list_key_int = list_key.parse::<u8>();
                        match list_key_int {
                            Ok(x) => {
                                let col = tuple.select(x)?;
                                if col.has_value() == false { 
                                    match &**tuple.get_schema() {
                                        NP_Parsed_Schema::Tuple { sortable: _, i: _, values} => {
                                            let ptr = NP_Ptr::_new_standard_ptr(0, &values[x as usize], col.memory);
                                            return ptr._deep_get(path, path_index + 1);
                                        },
                                        _ => { unsafe { unreachable_unchecked() } }
                                    }
                                }
                                col._deep_get(path, path_index + 1)
                            },
                            Err(_e) => {
                                Err(NP_Error::new("Can't query tuple with string, need number!".to_owned()))
                            }
                        }
                    },
                    None => {
                        unreachable!();
                    }
                }

            },
            _ => { return Ok(None); }
        }

        // Ok(None)
    }
    
    /// Sets the default value for this data type into the buffer.
    /// This is NOT related to the `default` key in the schema, this is the default for the underlying Rust data type.
    pub fn set_default(&'ptr mut self) -> Result<(), NP_Error> {

        match self.schema.into_type_data().2 {
            NP_TypeKeys::None => { },
            NP_TypeKeys::Any => { },
            NP_TypeKeys::UTF8String => {
                String::set_value(self, Box::new(&String::default()))?;
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::set_value(self, Box::new(&NP_Bytes::default()))?;
            },
            NP_TypeKeys::Int8 => {
                i8::set_value(self, Box::new(&i8::default()))?;
            },
            NP_TypeKeys::Int16 => {
                i16::set_value(self, Box::new(&i16::default()))?;
            },
            NP_TypeKeys::Int32 => {
                i32::set_value(self, Box::new(&i32::default()))?;
            },
            NP_TypeKeys::Int64 => {
                i64::set_value(self, Box::new(&i64::default()))?;
            },
            NP_TypeKeys::Uint8 => {
                u8::set_value(self, Box::new(&u8::default()))?;
            },
            NP_TypeKeys::Uint16 => {
                u16::set_value(self, Box::new(&u16::default()))?;
            },
            NP_TypeKeys::Uint32 => {
                u32::set_value(self, Box::new(&u32::default()))?;
            },
            NP_TypeKeys::Uint64 => {
                u64::set_value(self, Box::new(&u64::default()))?;
            },
            NP_TypeKeys::Float => {
                f32::set_value(self, Box::new(&f32::default()))?;
            },
            NP_TypeKeys::Double => {
                f64::set_value(self, Box::new(&f64::default()))?;
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::set_value(self, Box::new(&NP_Dec::default()))?;
            },
            NP_TypeKeys::Boolean => {
                bool::set_value(self, Box::new(&bool::default()))?;
            },
            NP_TypeKeys::Geo => {
                NP_Geo::set_value(self, Box::new(&NP_Geo::default()))?;
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::set_value(self, Box::new(&NP_UUID::default()))?;
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::set_value(self, Box::new(&NP_ULID::default()))?;
            },
            NP_TypeKeys::Date => {
                NP_Date::set_value(self, Box::new(&NP_Date::default()))?;
            },
            NP_TypeKeys::Enum => {
                NP_Option::set_value(self, Box::new(&NP_Option::default()))?;
            },
            NP_TypeKeys::Table => {
                // NP_Table::set_value(self, Box::new(&NP_Table::default()))?;
            },
            NP_TypeKeys::Map => {
                // NP_Map::set_value(self, Box::new(&NP_Map::default()))?;
            },
            NP_TypeKeys::List => {
                // NP_List::set_value(self, Box::new(&NP_List::default()))?;
            },
            NP_TypeKeys::Tuple => {
                // NP_Tuple::set_value(self, Box::new(&NP_Tuple::default()))?;
            }
        };

        Ok(())
    }

    /// Calculate the number of bytes used by this pointer and it's descendants.
    /// 
    pub fn calc_size(&self) -> Result<usize, NP_Error> {

        if self.address == 0 {
            return Ok(0);
        }

        let base_size = self.memory.ptr_size(&self.kind);

        if self.kind.get_value_addr() == 0 { // no value, just base size
            return Ok(base_size);
        }

        let type_size = match self.schema.into_type_data().2 {
            NP_TypeKeys::None => {
                Ok(0)
            },
            NP_TypeKeys::Any => {
                Ok(0)
            },
            NP_TypeKeys::UTF8String => {
                String::get_size(self)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::get_size(self)
            },
            NP_TypeKeys::Int8 => {
                i8::get_size(self)
            },
            NP_TypeKeys::Int16 => {
                i16::get_size(self)
            },
            NP_TypeKeys::Int32 => {
                i32::get_size(self)
            },
            NP_TypeKeys::Int64 => {
                i64::get_size(self)
            },
            NP_TypeKeys::Uint8 => {
                u8::get_size(self)
            },
            NP_TypeKeys::Uint16 => {
                u16::get_size(self)
            },
            NP_TypeKeys::Uint32 => {
                u32::get_size(self)
            },
            NP_TypeKeys::Uint64 => {
                u64::get_size(self)
            },
            NP_TypeKeys::Float => {
                f32::get_size(self)
            },
            NP_TypeKeys::Double => {
                f64::get_size(self)
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::get_size(self)
            },
            NP_TypeKeys::Boolean => {
                bool::get_size(self)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::get_size(self)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::get_size(self)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::get_size(self)
            },
            NP_TypeKeys::Date => {
                NP_Date::get_size(self)
            },
            NP_TypeKeys::Enum => {
                NP_Option::get_size(self)
            },
            NP_TypeKeys::Table => {
                NP_Table::get_size(self)
            },
            NP_TypeKeys::Map => {
                NP_Map::get_size(self)
            },
            NP_TypeKeys::List => {
                NP_List::get_size(self)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::get_size(self)
            }
        }?;

        Ok(type_size + base_size)
    }


    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    /// 
    pub fn json_encode(&self) -> NP_JSON {

        if self.kind.get_value_addr() == 0 {
            return NP_JSON::Null;
        }

        let type_key = self.schema.into_type_data().2;

        match type_key {
            NP_TypeKeys::None => { 
                NP_JSON::Null 
            }
            NP_TypeKeys::Any => {
                NP_JSON::Null
            },
            NP_TypeKeys::UTF8String => {
                String::to_json(self)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::to_json(self)
            },
            NP_TypeKeys::Int8 => {
                i8::to_json(self)
            },
            NP_TypeKeys::Int16 => {
                i16::to_json(self)
            },
            NP_TypeKeys::Int32 => {
                i32::to_json(self)
            },
            NP_TypeKeys::Int64 => {
                i64::to_json(self)
            },
            NP_TypeKeys::Uint8 => {
                u8::to_json(self)
            },
            NP_TypeKeys::Uint16 => {
                u16::to_json(self)
            },
            NP_TypeKeys::Uint32 => {
                u32::to_json(self)
            },
            NP_TypeKeys::Uint64 => {
                u64::to_json(self)
            },
            NP_TypeKeys::Float => {
                f32::to_json(self)
            },
            NP_TypeKeys::Double => {
                f64::to_json(self)
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::to_json(self)
            },
            NP_TypeKeys::Boolean => {
                bool::to_json(self)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::to_json(self)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::to_json(self)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::to_json(self)
            },
            NP_TypeKeys::Date => {
                NP_Date::to_json(self)
            },
            NP_TypeKeys::Enum => {
                NP_Option::to_json(self)
            },
            NP_TypeKeys::Table => {
                NP_Table::to_json(self)
            },
            NP_TypeKeys::Map => {
                NP_Map::to_json(self)
            },
            NP_TypeKeys::List => {
                NP_List::to_json(self)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::to_json(self)
            }
        }
    }

    #[doc(hidden)]
    pub fn compact(self, copy_to: &'ptr mut NP_Ptr<'ptr>) -> Result<(), NP_Error> {

        if self.address == 0 || self.kind.get_value_addr() == 0 {
            return Ok(());
        }

        match **self.schema {
            NP_Parsed_Schema::Any        { sortable: _, i:_ }                        => { Ok(()) }
            NP_Parsed_Schema::UTF8String { sortable: _, i:_, size:_, default:_ }     => { String::do_compact(self, copy_to) }
            NP_Parsed_Schema::Bytes      { sortable: _, i:_, size:_, default:_ }     => { NP_Bytes::do_compact(self, copy_to) }
            NP_Parsed_Schema::Int8       { sortable: _, i:_, default: _ }            => { i8::do_compact(self, copy_to) }
            NP_Parsed_Schema::Int16      { sortable: _, i:_ , default: _ }           => { i16::do_compact(self, copy_to)}
            NP_Parsed_Schema::Int32      { sortable: _, i:_ , default: _ }           => { i32::do_compact(self, copy_to) }
            NP_Parsed_Schema::Int64      { sortable: _, i:_ , default: _ }           => { i64::do_compact(self, copy_to) }
            NP_Parsed_Schema::Uint8      { sortable: _, i:_ , default: _ }           => { u8::do_compact(self, copy_to) }
            NP_Parsed_Schema::Uint16     { sortable: _, i:_ , default: _ }           => { u16::do_compact(self, copy_to) }
            NP_Parsed_Schema::Uint32     { sortable: _, i:_ , default: _ }           => { u32::do_compact(self, copy_to) }
            NP_Parsed_Schema::Uint64     { sortable: _, i:_ , default: _ }           => { u64::do_compact(self, copy_to) }
            NP_Parsed_Schema::Float      { sortable: _, i:_ , default: _ }           => { f32::do_compact(self, copy_to) }
            NP_Parsed_Schema::Double     { sortable: _, i:_ , default: _ }           => { f64::do_compact(self, copy_to) }
            NP_Parsed_Schema::Decimal    { sortable: _, i:_, exp:_, default:_ }      => { NP_Dec::do_compact(self, copy_to) }
            NP_Parsed_Schema::Boolean    { sortable: _, i:_, default:_ }             => { bool::do_compact(self, copy_to) }
            NP_Parsed_Schema::Geo        { sortable: _, i:_, default:_, size:_ }     => { NP_Geo::do_compact(self, copy_to) }
            NP_Parsed_Schema::Uuid       { sortable: _, i:_ }                        => { NP_UUID::do_compact(self, copy_to) }
            NP_Parsed_Schema::Ulid       { sortable: _, i:_ }                        => { NP_ULID::do_compact(self, copy_to) }
            NP_Parsed_Schema::Date       { sortable: _, i:_, default:_ }             => { NP_Date::do_compact(self, copy_to) }
            NP_Parsed_Schema::Enum       { sortable: _, i:_, default:_, choices: _ } => { NP_Option::do_compact(self, copy_to) }
            NP_Parsed_Schema::Table      { sortable: _, i:_, columns:_ }             => { NP_Table::do_compact(self, copy_to) }
            NP_Parsed_Schema::Map        { sortable: _, i:_, value:_ }               => { NP_Map::do_compact(self, copy_to) }
            NP_Parsed_Schema::List       { sortable: _, i:_, of:_ }                  => { NP_List::do_compact(self, copy_to) }
            NP_Parsed_Schema::Tuple      { sortable: _, i:_, values:_ }              => { NP_Tuple::do_compact(self, copy_to) }
            _ => { panic!() }
        }
    }
}


/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
*/