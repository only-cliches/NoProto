//! All values in buffers are accessed and modified through pointers
//! 
//! NP_Ptr are the primary abstraction to read, update or delete values in a buffer.
//! Pointers should *never* be created directly, instead the various methods provided by the library to access
//! the internals of the buffer should be used.
//! 
//! Once you have a pointer you can read it's contents if it's a scalar value with `.get()` or convert it to a collection with `.into()`.
//! When you attempt to read, update, or convert a pointer the schema is checked for that pointer location.  If the schema conflicts with the operation you're attempting it will fail.
//! As a result, you should be careful to make sure your reads and updates to the buffer line up with the schema you provided.
//! 
//! 

/// Misc types (NP_Dec, NP_Geo, etc)
pub mod misc;
/// String type
pub mod string;
/// Bytes type
pub mod bytes;
/// Any type
pub mod any;
/// Numbers types
pub mod numbers;

use crate::json_flex::NP_JSON;
use crate::memory::{NP_Size, NP_Memory};
use crate::NP_Error;
use crate::{schema::{NP_Schema, NP_TypeKeys}, collection::{map::NP_Map, table::NP_Table, list::NP_List, tuple::NP_Tuple}, utils::{overflow_error, print_path, type_error}};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{rc::Rc, vec::Vec};
use bytes::NP_Bytes;
pub use misc::{NP_Geo, NP_Dec, NP_UUID, NP_ULID, NP_Date, NP_Option};
use any::NP_Any;

// stores the different kinds of pointers and the details for each pointer
#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum NP_PtrKinds {
    None,
    // scalar / collection
    Standard  { addr: u32 }, // u32(4 bytes [4]), u16(2 bytes [2])

    // collection items
    MapItem   { addr: u32, next: u32, key: u32 },  // u32(12 bytes [4, 4, 4]), u16(6 bytes [2, 2, 2])
    TableItem { addr: u32, next: u32, i: u8    },  // u32(9  bytes [4, 4, 1]), u16(5 bytes [2, 2, 1])
    ListItem  { addr: u32, next: u32, i: u16   },  // u32(10 bytes [4, 4, 2]), u16(6 bytes [2, 2, 2])
}


impl NP_PtrKinds {

    /// Get the address of the value for this pointer
    pub fn get_value_addr(&self) -> u32 {
        match self {
            NP_PtrKinds::None                                              => { 0 },
            NP_PtrKinds::Standard  { addr } =>                      { *addr },
            NP_PtrKinds::MapItem   { addr, key: _,  next: _ } =>    { *addr },
            NP_PtrKinds::TableItem { addr, i: _,    next: _ } =>    { *addr },
            NP_PtrKinds::ListItem  { addr, i:_ ,    next: _ } =>    { *addr }
        }
    }
}

/// This trait is used to implement types as NoProto buffer types.
/// This includes all the type data, encoding and decoding methods.
pub trait NP_Value {
    /// Check if a specific string "type" in the schema matches this data type
    /// 
    fn is_type(_type_str: &str) -> bool { false }

    /// Get the type information for this type (static)
    /// 
    fn type_idx() -> (i64, String) { (-1, "null".to_owned()) }

    /// Get the type information for this type (instance)
    /// 
    fn self_type_idx(&self) -> (i64, String) { (-1, "null".to_owned()) }

    /// Called for each declaration in the schema for a given type, useful for storing configuration details about the schema
    /// 
    fn schema_state(_type_string: &str, _json_schema: &NP_JSON) -> Result<i64, NP_Error> { Ok(0) }

    /// Set the value of this scalar into the buffer
    /// 
    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        let mut message = "This type (".to_owned();
        message.push_str(Self::type_idx().1.as_str());
        message.push_str(") doesn't support .set()!");
        Err(NP_Error::new(message.as_str()))
    }

    /// Pull the data from the buffer and convert into type
    /// 
    fn into_value(_pointer: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let message = "This type  doesn't support into!".to_owned();
        Err(NP_Error::new(message.as_str()))
    }

    /// Convert this type into a JSON value (recursive for collections)
    /// 
    fn to_json(_pointer: NP_Lite_Ptr) -> NP_JSON {
         NP_JSON::Null
    }

    /// Calculate the size of this pointer and it's children (recursive for collections)
    /// 
    fn get_size(_pointer: NP_Lite_Ptr) -> Result<u32, NP_Error> {
         Err(NP_Error::new("Size not supported for this type!"))
    }

    /// Get the default value from the schema
    /// 
    fn schema_default(_schema: Rc<NP_Schema>) -> Option<Box<Self>> {
        None
    }
    
    /// Handle copying from old pointer/buffer to new pointer/buffer (recursive for collections)
    /// 
    fn do_compact(from_ptr: NP_Lite_Ptr, to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value + Default {
        if from_ptr.location == 0 {
            return Ok(());
        }

        match Self::into_value(from_ptr)? {
            Some(x) => {
                Self::set_value(to_ptr, Box::new(&*x))?;
            },
            None => { }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
#[doc(hidden)]
/// Lite pointer for manipulating non typed pointer data
pub struct NP_Lite_Ptr {
    /// pointer location in buffer 
    pub location: u32, 
    /// the kind of pointer this is (standard, list item, map item, etc).  Includes value address
    pub kind: NP_PtrKinds, 
    /// the underlying buffer this pointer is a part of
    pub memory: Rc<NP_Memory>, 
    /// schema stores the *actual* schema data for this pointer, regardless of type casting
    pub schema: Rc<NP_Schema>
}

impl NP_Lite_Ptr {

    /// New standard lite pointer  
    pub fn new_standard(location: u32, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self {

        let addr = location as usize;
        
        NP_Lite_Ptr {
            location: location,
            kind: NP_PtrKinds::Standard { addr: match &memory.size {
                NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
            }},
            memory: memory,
            schema: schema
        }
    }

    /// Convert a normal pointer into a lite one
    pub fn from<X: NP_Value + Default>(ptr: NP_Ptr<X>) -> Self {
        let addr = ptr.location as usize;
        NP_Lite_Ptr {
            location: ptr.location,
            kind: NP_PtrKinds::Standard { addr: match &ptr.memory.size {
                NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
            }},
            memory: ptr.memory,
            schema: ptr.schema
        }
    }

    /// Convert into a Lite pointer from borrowed pointer
    pub fn from_borrowed<X: NP_Value + Default>(ptr: &NP_Ptr<X>) -> Self {
        let addr = ptr.location as usize;
        NP_Lite_Ptr {
            location: ptr.location,
            kind: NP_PtrKinds::Standard { addr: match &ptr.memory.size {
                NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
            }},
            memory: Rc::clone(&ptr.memory),
            schema: Rc::clone(&ptr.schema)
        }
    }

    /// Convert a lite pointer into a normal one
    pub fn into<X: NP_Value + Default>(self) -> NP_Ptr<X> {
        NP_Ptr {
            location: self.location,
            kind: self.kind,
            memory: self.memory,
            schema: self.schema,
            value: X::default()
        }
    }

    /// used to run compaction on this pointer
    /// should not be called directly by the library user
    /// Use NP_Factory methods of `compact` and `maybe_compact`.
    pub fn compact(self, copy_to: NP_Lite_Ptr) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => {
                Ok(())
            },
            NP_TypeKeys::JSON => {
                unreachable!()
            },
            NP_TypeKeys::UTF8String => {
                String::do_compact(self, copy_to)
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::do_compact(self, copy_to)
            },
            NP_TypeKeys::Int8 => {
                i8::do_compact(self, copy_to)
            },
            NP_TypeKeys::Int16 => {
                i16::do_compact(self, copy_to)
            },
            NP_TypeKeys::Int32 => {
                i32::do_compact(self, copy_to)
            },
            NP_TypeKeys::Int64 => {
                i64::do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint8 => {
                u8::do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint16 => {
                u16::do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint32 => {
                u32::do_compact(self, copy_to)
            },
            NP_TypeKeys::Uint64 => {
                u64::do_compact(self, copy_to)
            },
            NP_TypeKeys::Float => {
                f32::do_compact(self, copy_to)
            },
            NP_TypeKeys::Double => {
                f64::do_compact(self, copy_to)
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::do_compact(self, copy_to)
            },
            NP_TypeKeys::Boolean => {
                bool::do_compact(self, copy_to)
            },
            NP_TypeKeys::Geo => {
                NP_Geo::do_compact(self, copy_to)
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::do_compact(self, copy_to)
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::do_compact(self, copy_to)
            },
            NP_TypeKeys::Date => {
                NP_Date::do_compact(self, copy_to)
            },
            NP_TypeKeys::Enum => {
                NP_Option::do_compact(self, copy_to)
            },
            NP_TypeKeys::Table => {
                NP_Table::do_compact(self, copy_to)
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::do_compact(self, copy_to)
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::do_compact(self, copy_to)
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::do_compact(self, copy_to)
            }
        }
    }
}

/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 16 or 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
/// 
/// # Using Scalar Types with Pointers
/// 
/// Scalars can easily be added or retrieved from a buffer using the `deep_set` and `deep_get` methods on the buffers.
/// ```rust
/// use no_proto::error::NP_Error;
/// use no_proto::NP_Factory;
/// use no_proto::pointer::misc::NP_Date;
/// use std::time::{SystemTime, UNIX_EPOCH};
/// 
/// // Simple schema with just a date
/// let scalar_factory = NP_Factory::new(r#"{
///     "type": "date"
/// }"#)?;
/// 
/// let mut new_buffer = scalar_factory.empty_buffer(None, None);
/// 
/// // When using scalar values, you must use the correct type with `deep_set`.
/// // For the `date` data type the correct rust type is `NP_Date`
/// let since_the_epoch = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards");
/// let time_now: NP_Date = NP_Date::new(since_the_epoch.as_millis() as u64);
/// 
/// // deep set root
/// new_buffer.deep_set("", time_now.clone());
/// 
/// // Deep get is just the reverse, must type cast correctly.
/// assert_eq!(new_buffer.deep_get::<NP_Date>("")?, Some(Box::new(time_now)));
/// 
/// // clear value at root
/// new_buffer.deep_clear("")?;
/// 
/// assert_eq!(new_buffer.deep_get::<NP_Date>("")?, None);
/// 
/// # Ok::<(), NP_Error>(()) 
/// ```
/// 
/// You can learn about which scalar types map with which rust data types on [this page](../schema/index.html#supported-data-types).
/// 
/// # Using Collection Types with Pointers
/// 
/// You can always retrieve, update and delete scalar values from within collections following the guidelines above. The process outlined here is mostly useful for iterating through collections, for example if you wanted to know what values are in a list or how large a list is.
/// 
/// ```rust
/// use no_proto::error::NP_Error;
/// use no_proto::NP_Factory;
/// use no_proto::collection::list::NP_List;
/// use std::time::{SystemTime, UNIX_EPOCH};
/// 
/// // Simple schema with just a date
/// let list_factory = NP_Factory::new(r#"{
///     "type": "list",
///     "of": {"type": "u32"}
/// }"#)?;
/// 
/// let mut new_buffer = list_factory.empty_buffer(None, None);
/// 
/// // we can use `deep_set` to set internal values of the list
/// new_buffer.deep_set("2", 200u32)?; // index 2 of list
/// new_buffer.deep_set("9", 150u32)?; // index 9 of list
/// 
/// let mut which_items: Vec<u32> = Vec::new();
/// 
/// // to access collection internals, we must open the buffer
/// new_buffer.open::<NP_List<u32>>(&mut |mut root_ptr| {
///     // root_ptr is a NP_Ptr that is generic over the root type cast in the `open` function.
///     // in this case root_ptr is NP_Ptr<NP_List<u32>>
///        
///     // convert the root pointer into a list.
///     let root_list: NP_List<u32> = root_ptr.into()?.unwrap();
///     
///     // convert the list into an iterator, then loop over it.
///     for mut item in root_list.it() {
///         // will loop 10 times... 0 to 9 since we put item at the 9th index
///         match item.select().unwrap().into().unwrap() {
///             Some(x) => { which_items.push(x) },
///             None => { which_items.push(0) }
///         }
///     }
/// 
///     Ok(())
/// })?;
/// 
/// assert_eq!(which_items, vec![0, 0, 200u32, 0, 0, 0, 0, 0, 0, 150u32]);
/// 
/// # Ok::<(), NP_Error>(()) 
/// ```
///  
/// 
/// 
#[derive(Debug)]
pub struct NP_Ptr<T: NP_Value + Default> {
    /// pointer address in buffer 
    pub location: u32, 
    /// the kind of pointer this is (standard, list item, map item, etc).  Includes value address
    pub kind: NP_PtrKinds, 
    /// the underlying buffer this pointer is a part of
    pub memory: Rc<NP_Memory>, 
    /// schema stores the *actual* schema data for this pointer, regardless of type casting
    pub schema: Rc<NP_Schema>, 
    /// a static invocation of the pointer type
    pub value: T 
}

impl<T: NP_Value + Default> NP_Ptr<T> {

    /// Retrieves the value at this pointer, only useful for scalar values (not collections).
    pub fn get(&mut self) -> Result<Option<T>, NP_Error> {

        match NP_TypeKeys::from(T::type_idx().0) {
            NP_TypeKeys::Table => { return Err(NP_Error::new("Can't get Table object, use .into()!")) },
            NP_TypeKeys::List => { return Err(NP_Error::new("Can't get List object, use .into()")) },
            NP_TypeKeys::Tuple => { return Err(NP_Error::new("Can't get Tuple object, use .into()")) },
            NP_TypeKeys::Map => { return Err(NP_Error::new("Can't get Map object, use .into()")) },
            _ => {  }
        };

        let value = T::into_value(NP_Lite_Ptr::from_borrowed(self))?;
        
        Ok(match value {
            Some (x) => {
                Some(*x)
            },
            None => {
                match T::schema_default(Rc::clone(&self.schema)) {
                    Some(x) => Some(*x),
                    None => None
                }
            }
        })
    }


    /// Sets the value for this pointer, only works for scalar types (not collection types).
    pub fn set(&mut self, value: T) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(T::type_idx().0) {
            NP_TypeKeys::JSON => { return Err(NP_Error::new("Can't set JSON Object!")) },
            NP_TypeKeys::Table => { return Err(NP_Error::new("Can't set Table object!")) },
            NP_TypeKeys::List => { return Err(NP_Error::new("Can't set List object!")) },
            NP_TypeKeys::Tuple => { return Err(NP_Error::new("Can't set Tuple object!")) },
            NP_TypeKeys::Map => { return Err(NP_Error::new("Can't set Map object!")) },
            _ => { }
        };
        
        self.kind = T::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&value))?;
        Ok(())
    }

    #[doc(hidden)]
    pub fn _new_standard_ptr(location: u32, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self {

        let addr = location as usize;
        
        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::Standard { addr: match &memory.size {
                NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
            }},
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn _new_table_item_ptr(location: u32, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self {

        let addr = location as usize;
        let b_bytes = &memory.read_bytes();

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::TableItem { 
                addr: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
                },
                next: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2])) as u32
                },
                i: match &memory.size {
                    NP_Size::U32 => b_bytes[addr + 8],
                    NP_Size::U16 => b_bytes[addr + 4]
                }
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn _new_map_item_ptr(location: u32, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self {

        let addr = location as usize;

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::MapItem { 
                addr: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
                },
                next: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2])) as u32
                },
                key: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr + 8).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr + 4).unwrap_or(&[0; 2])) as u32
                }
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn _new_list_item_ptr(location: u32, schema: Rc<NP_Schema>, memory: Rc<NP_Memory>) -> Self {

        let addr = location as usize;

        NP_Ptr {
            location: location,
            kind: NP_PtrKinds::ListItem { 
                addr: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr).unwrap_or(&[0; 2])) as u32
                },
                next: match &memory.size {
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2])) as u32
                },
                i: match &memory.size {
                    NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(addr + 8).unwrap_or(&[0; 2])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(addr + 4).unwrap_or(&[0; 2]))
                }
            },
            memory: memory,
            schema: schema,
            value: T::default()
        }
    }

    /// Check if there is any value set at this pointer
    pub fn has_value(&self) -> bool {

        if self.kind.get_value_addr() == 0 {
            return false;
        }

        return true;
    }

    /// Clear / delete the value at this pointer.  This is just dereferences, so it doesn't actually remove items from the buffer.  Also if this is called on a collection type, all children of the collection will also be cleared.
    /// 
    /// If you clear a large object it's probably a good idea to run compaction to recover the free space.
    /// 
    pub fn clear(self) -> Result<NP_Ptr<T>, NP_Error> {
        Ok(NP_Ptr {
            location: self.location,
            kind: self.memory.set_value_address(self.location, 0, &self.kind),
            memory: self.memory,
            schema: self.schema,
            value: self.value
        })
    }

    /// Destroy this pointer and convert it into the underlying data type.
    /// This is mostly useful for collections but can also be used to copy scalar values out of the buffer.
    /// 
    pub fn into(self) -> Result<Option<T>, NP_Error> {

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
        
        let result = T::into_value(NP_Lite_Ptr::from_borrowed(&self))?;

        Ok(match result {
            Some(x) => Some(*x),
            None => {
                match T::schema_default(Rc::clone(&self.schema)) {
                    Some(x) => Some(*x),
                    None => None
                }
            }
        })
    }

    /// Used to set scalar values inside the buffer, the path only works with dot notation.
    /// This does not work with collection types or `NP_JSON`.
    /// 
    /// The request path will start from the location of this pointer.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    pub fn deep_set<X: NP_Value + Default>(&mut self, path: &str, value: X) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::JSON => { Err(NP_Error::new("Can't deep set with JSON type!")) },
            NP_TypeKeys::Table => { Err(NP_Error::new("Can't deep set table type!")) },
            NP_TypeKeys::Map => { Err(NP_Error::new("Can't deep set map type!")) },
            NP_TypeKeys::List => { Err(NP_Error::new("Can't deep set list type!")) },
            NP_TypeKeys::Tuple => { Err(NP_Error::new("Can't deep set tuple type!")) },
            _ => {
                let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
                let pointer: NP_Ptr<NP_Any> = NP_Ptr::_new_standard_ptr(self.location, Rc::clone(&self.schema), Rc::clone(&self.memory));
                pointer._deep_set::<X>(vec_path, 0, value)
            }
        }
    }

    /// Clear an inner value from the buffer.  The path only works with dot notation.
    /// This can also be used to clear deeply nested collection objects.
    /// 
    /// The request path will start from the location of this pointer.
    /// 
    pub fn deep_clear(&self, path: &str) -> Result<(), NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::_new_standard_ptr(self.location, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_clear(vec_path, 0)
    }
  
    /// Retrieve an inner value from the buffer.  The path only works with dot notation.
    /// You can also use this to get JSON by casting the request type to `NP_JSON`.
    /// This can also be used to retrieve deeply nested collection objects.
    /// 
    /// The request path will start from the location of this pointer.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    pub fn deep_get<X: NP_Value + Default>(&self, path: &str) -> Result<Option<Box<X>>, NP_Error> {
        let vec_path: Vec<&str> = path.split(".").filter(|v| { v.len() > 0 }).collect();
        let pointer: NP_Ptr<NP_Any> = NP_Ptr::_new_standard_ptr(self.location, Rc::clone(&self.schema), Rc::clone(&self.memory));
        pointer._deep_get::<X>(vec_path, 0)
    }

    #[doc(hidden)]
    pub fn _deep_clear(self, path: Vec<&str>, path_index: usize) -> Result<(), NP_Error> {

        
        if path.len() == path_index {
            self.clear()?;
            return Ok(());
        }

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Table => {

                let result = NP_Table::into_value(NP_Lite_Ptr::from(self))?;
                
                match result {
                    Some(mut table) => {
                        let table_key = path[path_index];
                        let col = table.select::<NP_Any>(table_key)?;
                        col._deep_clear(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Map => {


                let result = NP_Map::<NP_Any>::into_value(NP_Lite_Ptr::from(self))?;

                match result {
                    Some(mut map) => {
                        let map_key = path[path_index];
                        let col = map.select(&map_key.as_bytes().to_vec())?;
                        col._deep_clear(path, path_index + 1)
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::List => {

                let result = NP_List::<NP_Any>::into_value(NP_Lite_Ptr::from(self))?;

                match result {
                    Some(mut list) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u16>();
                        match list_key_int {
                            Ok(x) => {
                                let col = list.select(x)?;
                                col._deep_clear(path, path_index + 1)
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

                let result = NP_Tuple::into_value(NP_Lite_Ptr::from(self))?;

                match result {
                    Some(tuple) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u8>();
                        match list_key_int {
                            Ok(x) => {
                                let col = tuple.select::<NP_Any>(x)?;
                                col._deep_clear(path, path_index + 1)
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
            _ => { // scalar type
                Err(NP_Error::new("Path error, found scalar instead of collection!".to_owned()))
            }
        }


    }

    #[doc(hidden)]
    pub fn _deep_set<X: NP_Value + Default>(self, path: Vec<&str>, path_index: usize, value: X) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::JSON => { return Err(NP_Error::new("Can't set with JSON Object!")) },
            _ => { }
        };

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Table => {

                overflow_error("deep set", &path, path_index)?;

                let result = NP_Table::into_value(NP_Lite_Ptr::from(self))?;
                
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

                overflow_error("deep set", &path, path_index)?;
                
                let result = NP_Map::<NP_Any>::into_value(NP_Lite_Ptr::from(self))?;

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

                overflow_error("deep set", &path, path_index)?;

                let result = NP_List::<NP_Any>::into_value(NP_Lite_Ptr::from(self))?;

                match result {
                    Some(mut list) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u16>();
                        match list_key_int {
                            Ok(x) => {
                                let col = list.select(x)?;
                                col._deep_set::<X>(path, path_index + 1, value)?;
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

                Ok(())
            },
            NP_TypeKeys::Tuple => {

                overflow_error("deep set", &path, path_index)?;

                let result = NP_Tuple::into_value(NP_Lite_Ptr::from(self))?;

                match result {
                    Some(tuple) => {
                        let list_key = path[path_index];
                        let list_key_int = list_key.parse::<u8>();
                        match list_key_int {
                            Ok(x) => {
                                let col = tuple.select::<NP_Any>(x)?;
                                col._deep_set::<X>(path, path_index + 1, value)?;
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

                Ok(())
            },
            _ => { // scalar type
                if path.len() != path_index { // reached scalar value but not at end of path
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

                X::set_value(NP_Lite_Ptr::from(self), Box::new(&value))?;

                Ok(())
            }
        }
    }

    #[doc(hidden)]
    pub fn _deep_get<X: NP_Value + Default>(self, path: Vec<&str>, path_index: usize) -> Result<Option<Box<X>>, NP_Error> {


        let is_json_req = match NP_TypeKeys::from(X::type_idx().0) {
            NP_TypeKeys::JSON => true,
            _ => false
        };

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Table => {

                if is_json_req == false {
                    overflow_error("deep get", &path, path_index)?;
                }

                let result = NP_Table::into_value(NP_Lite_Ptr::from_borrowed(&self))?;
                
                match result {
                    Some(mut table) => {
                        if path.len() == path_index && is_json_req {
                            // make sure the schema and type match
                            if is_json_req == false { type_error(&self.schema.type_data, &X::type_idx(), &path, path_index)?; };
                            X::into_value(NP_Lite_Ptr::from(self))
                        } else {
                            let table_key = path[path_index];
                            let col = table.select::<NP_Any>(table_key)?;
                            col._deep_get::<X>(path, path_index + 1)
                        }
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Map => {

                if is_json_req == false {
                    overflow_error("deep get", &path, path_index)?;
                }
                let result = NP_Map::<NP_Any>::into_value(NP_Lite_Ptr::from_borrowed(&self))?;

                match result {
                    Some(mut map) => {
                        if path.len() == path_index {
                            // make sure the schema and type match
                            if is_json_req == false { type_error(&self.schema.type_data, &X::type_idx(), &path, path_index)?; };
                            X::into_value(NP_Lite_Ptr::from(self))
                        } else {
                            let map_key = path[path_index];
                            let col = map.select(&map_key.as_bytes().to_vec())?;
                            col._deep_get::<X>(path, path_index + 1)
                        }
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::List => {

                if is_json_req == false {
                    overflow_error("deep get", &path, path_index)?;
                }

                let result = NP_List::<NP_Any>::into_value(NP_Lite_Ptr::from_borrowed(&self))?;

                match result {
                    Some(mut list) => {
                        if path.len() == path_index {
                            // make sure the schema and type match
                            if is_json_req == false { type_error(&self.schema.type_data, &X::type_idx(), &path, path_index)?; };
                            X::into_value(NP_Lite_Ptr::from(self))
                        } else {
                            let list_key = path[path_index];
                            let list_key_int = list_key.parse::<u16>();
                            match list_key_int {
                                Ok(x) => {
                                    let col = list.select(x)?;
                                    col._deep_get::<X>(path, path_index + 1)
                                },
                                Err(_e) => {
                                    Err(NP_Error::new("Can't query list with string, need number!".to_owned()))
                                }
                            }

                        }
                    },
                    None => {
                        unreachable!();
                    }
                }
            },
            NP_TypeKeys::Tuple => {
                
                if is_json_req == false {
                    overflow_error("deep get", &path, path_index)?;
                }

                let result = NP_Tuple::into_value(NP_Lite_Ptr::from_borrowed(&self))?;

                match result {
                    Some(tuple) => {
                        if path.len() == path_index {
                            // make sure the schema and type match
                            if is_json_req == false { type_error(&self.schema.type_data, &X::type_idx(), &path, path_index)?; };
                            X::into_value(NP_Lite_Ptr::from(self))
                        } else {
                            let list_key = path[path_index];
                            let list_key_int = list_key.parse::<u8>();
                            match list_key_int {
                                Ok(x) => {
                                    let col = tuple.select::<NP_Any>(x)?;
                                    col._deep_get::<X>(path, path_index + 1)
                                },
                                Err(_e) => {
                                    Err(NP_Error::new("Can't query tuple with string, need number!".to_owned()))
                                }
                            }

                        }
                    },
                    None => {
                        unreachable!();
                    }
                }

            },
            _ => { // scalar type

                if path.len() != path_index { // reached scalar type but not at end of path
                    let mut err = "TypeError: Attempted to deep get into collection but found scalar type (".to_owned();
                    err.push_str(self.schema.type_data.1.as_str());
                    err.push_str(") Path: ");
                    err.push_str(print_path(&path, path_index).as_str());
                    return Err(NP_Error::new(err));
                }

                // make sure the schema and type match
                if is_json_req == false { type_error(&self.schema.type_data, &X::type_idx(), &path, path_index)?; }

                match X::into_value(NP_Lite_Ptr::from_borrowed(&self))? {
                    Some(x) => {
                        Ok(Some(x))
                    },
                    None => {
                        Ok(X::schema_default(Rc::clone(&self.schema)))
                    }
                }
            }
        }

        // Ok(None)
    }
    
    /// Sets the default value for this data type into the buffer.
    /// This is NOT related to the `default` key in the schema, this is the default for the underlying Rust data type.
    pub fn set_default(&self) -> Result<(), NP_Error> {

        match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => { },
            NP_TypeKeys::JSON => { },
            NP_TypeKeys::UTF8String => {
                String::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&String::default()))?;
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Bytes::default()))?;
            },
            NP_TypeKeys::Int8 => {
                i8::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&i8::default()))?;
            },
            NP_TypeKeys::Int16 => {
                i16::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&i16::default()))?;
            },
            NP_TypeKeys::Int32 => {
                i32::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&i32::default()))?;
            },
            NP_TypeKeys::Int64 => {
                i64::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&i64::default()))?;
            },
            NP_TypeKeys::Uint8 => {
                u8::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&u8::default()))?;
            },
            NP_TypeKeys::Uint16 => {
                u16::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&u16::default()))?;
            },
            NP_TypeKeys::Uint32 => {
                u32::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&u32::default()))?;
            },
            NP_TypeKeys::Uint64 => {
                u64::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&u64::default()))?;
            },
            NP_TypeKeys::Float => {
                f32::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&f32::default()))?;
            },
            NP_TypeKeys::Double => {
                f64::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&f64::default()))?;
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Dec::default()))?;
            },
            NP_TypeKeys::Boolean => {
                bool::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&bool::default()))?;
            },
            NP_TypeKeys::Geo => {
                NP_Geo::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Geo::default()))?;
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_UUID::default()))?;
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_ULID::default()))?;
            },
            NP_TypeKeys::Date => {
                NP_Date::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Date::default()))?;
            },
            NP_TypeKeys::Enum => {
                NP_Option::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Option::default()))?;
            },
            NP_TypeKeys::Table => {
                NP_Table::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Table::default()))?;
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Map::default()))?;
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_List::default()))?;
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::set_value(NP_Lite_Ptr::from_borrowed(self), Box::new(&NP_Tuple::default()))?;
            }
        };

        Ok(())
    }

    /// Calculate the number of bytes used by this object and it's descendants.
    /// 
    pub fn calc_size(&self) -> Result<u32, NP_Error> {

        let base_size = self.memory.ptr_size(&self.kind);

        if self.location == 0 { // no value, just base size
            return Ok(base_size);
        }

        let type_size = match NP_TypeKeys::from(self.schema.type_data.0) {
            NP_TypeKeys::Any => {
                Ok(0)
            },
            NP_TypeKeys::JSON => {
                unreachable!()
            },
            NP_TypeKeys::UTF8String => {
                String::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int8 => {
                i8::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int16 => {
                i16::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int32 => {
                i32::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int64 => {
                i64::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint8 => {
                u8::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint16 => {
                u16::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint32 => {
                u32::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint64 => {
                u64::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Float => {
                f32::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Double => {
                f64::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Boolean => {
                bool::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Geo => {
                NP_Geo::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Date => {
                NP_Date::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Enum => {
                NP_Option::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Table => {
                NP_Table::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::get_size(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::get_size(NP_Lite_Ptr::from_borrowed(self))
            }
        }?;

        Ok(type_size + base_size)
    }


    /// Exports this pointer and all it's descendants into a JSON object.
    /// This will create a copy of the underlying data and return default values where there isn't data.
    pub fn json_encode(&self) -> NP_JSON {
        if self.location == 0 {
            return NP_JSON::Null;
        }

        let type_key = NP_TypeKeys::from(self.schema.type_data.0);

        match type_key {
            NP_TypeKeys::Any => {
                NP_JSON::Null
            },
            NP_TypeKeys::JSON => {
                unreachable!()
            },
            NP_TypeKeys::UTF8String => {
                String::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Bytes => {
                NP_Bytes::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int8 => {
                i8::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int16 => {
                i16::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int32 => {
                i32::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Int64 => {
                i64::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint8 => {
                u8::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint16 => {
                u16::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint32 => {
                u32::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uint64 => {
                u64::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Float => {
                f32::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Double => {
                f64::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Decimal => {
                NP_Dec::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Boolean => {
                bool::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Geo => {
                NP_Geo::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Uuid => {
                NP_UUID::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Ulid => {
                NP_ULID::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Date => {
                NP_Date::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Enum => {
                NP_Option::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Table => {
                NP_Table::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Map => {
                NP_Map::<NP_Any>::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::List => {
                NP_List::<NP_Any>::to_json(NP_Lite_Ptr::from_borrowed(self))
            },
            NP_TypeKeys::Tuple => {
                NP_Tuple::to_json(NP_Lite_Ptr::from_borrowed(self))
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
/*impl From<&NP_Ptr> for Result<String> {
    fn from(ptr: &NP_Ptr) -> Result<String> {
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

// cast Pointer => Result<i64>
impl From<&NP_Value> for Result<i64> {
    fn from(ptr: &NP_Value) -> Result<i64> {
        match ptr.value {
            NP_Value::int64 { value } => {
                Some(value)
            }
            _ => None
        }
    }
}*/