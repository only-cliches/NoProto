//! Top level abstraction for buffer objects

use crate::{NP_Size_Data, schema::NP_TypeKeys};
use crate::{memory::NP_Memory_Writable, utils::opt_err};
use crate::collection::tuple::NP_Tuple;

use crate::{pointer::{NP_Scalar}};
use crate::{collection::map::NP_Map};
use crate::{pointer::NP_Value};
use crate::pointer::NP_Cursor;
use crate::{schema::NP_Parsed_Schema, collection::table::NP_Table};
use alloc::vec::Vec;
use crate::{collection::{list::NP_List}};
use crate::error::NP_Error;
use crate::memory::{NP_Memory};
use crate::{json_flex::NP_JSON};
use crate::alloc::borrow::ToOwned;

/// The address location of the root pointer.
#[doc(hidden)]
pub const DEFAULT_ROOT_PTR_ADDR: usize = 2;
/// Maximum size of list collections
#[doc(hidden)]
pub const LIST_MAX_SIZE: usize = core::u16::MAX as usize;
#[doc(hidden)]
pub const VTABLE_SIZE: usize = 4;
#[doc(hidden)]
pub const VTABLE_BYTES: usize = 10;


/// Buffers contain the bytes of each object and allow you to perform reads, updates, deletes and compaction.
/// 
/// 
#[derive(Debug)]
pub struct NP_Buffer<'buffer> {
    /// Schema data used by this buffer
    memory: NP_Memory_Writable<'buffer>,
    cursor: NP_Cursor
}

impl<'buffer> Clone for NP_Buffer<'buffer> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            cursor: self.cursor.clone()
        }
    }
}

impl<'buffer> NP_Buffer<'buffer> {

    #[doc(hidden)]
    pub fn _new(memory: NP_Memory_Writable<'buffer>) -> Self { // make new buffer

        // is the root a sortable tuple?  if so, create its children and vtables
        match memory.get_schema(0) {
            NP_Parsed_Schema::Tuple { sortable, values, .. } => {
                if *sortable {
                    NP_Tuple::select(NP_Cursor::new(memory.root, 0, 0), values,  0, true, false, &memory).unwrap_or(None);
                }
            },
            _ => {}
        };

        NP_Buffer {
            cursor: NP_Cursor::new(memory.root, 0, 0),
            memory: memory
        }
    }

    /// Copy an object at the provided path and all it's children into JSON.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "uint8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// new_buffer.set(&["name"], "Jeb Kermin");
    /// new_buffer.set(&["age"], 30u8);
    /// 
    /// assert_eq!("{\"age\":30,\"name\":\"Jeb Kermin\"}", new_buffer.json_encode(&[])?.stringify());
    /// assert_eq!("\"Jeb Kermin\"", new_buffer.json_encode(&["name"])?.stringify());
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn json_encode(&self, path: &[&str]) -> Result<NP_JSON, NP_Error> {

        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), false, false, path)?;

        if let Some(x) = value_cursor {
            Ok(NP_Cursor::json_encode(0, &x, &self.memory))
        } else {
            Ok(NP_JSON::Null)
        }

    }

    /// Moves the underlying bytes out of the buffer, consuming the buffer in the process.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&[], "hello")?;
    /// // close buffer and get bytes
    /// let bytes: Vec<u8> = new_buffer.close();
    /// assert_eq!([0, 0, 0, 4, 0, 5, 104, 101, 108, 108, 111].to_vec(), bytes);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn close(self) -> Vec<u8> {
        self.memory.dump()
    }

    /// If the buffer is sortable, this provides only the sortable elements of the buffer.
    /// There is typically 10 bytes or more in front of the buffer that are identical between all the sortable buffers for a given schema.
    /// 
    /// This calculates how many leading identical bytes there are and returns only the bytes following them.  This allows your sortable buffers to be only as large as they need to be.
    /// 
    /// This operation fails if the buffer is not sortable.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "sorted": true,
    ///    "values": [
    ///         {"type": "u8"},
    ///         {"type": "string", "size": 6}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&["0"], 55u8)?;
    /// new_buffer.set(&["1"], "hello")?;
    /// 
    /// // the buffer with it's vtables take up 21 bytes!
    /// assert_eq!(new_buffer.read_bytes().len(), 21usize);
    /// 
    /// // close buffer and get sortable bytes
    /// let bytes: Vec<u8> = new_buffer.close_sortable()?;
    /// // with close_sortable() we only get the 8 value bytes we care about!
    /// assert_eq!([55, 104, 101, 108, 108, 111, 32].to_vec(), bytes);
    /// 
    /// // you can always re open the sortable buffers with this call
    /// let new_buffer = factory.open_sortable_buffer(bytes)?;
    /// assert_eq!(new_buffer.get(&["0"])?, Some(55u8));
    /// assert_eq!(new_buffer.get(&["1"])?, Some("hello "));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn close_sortable(self) -> Result<Vec<u8>, NP_Error> {
        match &self.memory.get_schema(0) {
            NP_Parsed_Schema::Tuple { values, sortable, .. } => {
                if *sortable == false {
                    Err(NP_Error::new("Attempted to close_sortable() on buffer that isn't sortable!"))
                } else {
                    let mut vtables = 1usize;
                    let mut length = values.len();
                    while length > 4 {
                        vtables +=1;
                        length -= 4;
                    }
                    let root_offset =  self.memory.root + 2 + (vtables * 10);

                    let closed_vec = self.memory.dump();
                    
                    Ok(closed_vec[root_offset..].to_vec())
                }
            },
            _ => Err(NP_Error::new("Attempted to close_sortable() on buffer that isn't sortable!"))
        }
    }

    /// Read the bytes of the buffer immutably.  No touching!
    /// 
    pub fn read_bytes(&self) -> &[u8] {
        self.memory.read_bytes()
    }

    /// Move buffer cursor to new location.  Cursors can only be moved into children.  If you need to move up reset the cursor to root, then move back down to the desired level.
    /// 
    /// This also creates objects/collections along the path as needed.  If you attempt to move into a path that doesn't exist, this method will return `false`.  Otherwise it will return `true` of the path requested exists or is something that can be made to exist.
    /// 
    pub fn move_cursor(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), true, false, path)?;

        let cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(false);
        };

        self.cursor = cursor;

        Ok(true)
    }

    /// Moves cursor position to root of buffer, the default.
    /// 
    pub fn cursor_to_root(&mut self) {
        self.cursor = NP_Cursor::new(self.memory.root, 0, 0);
    }

    /// Set the max value allowed for the specific data type at the given key.
    /// 
    /// String & Byte types only work if a `size` property is set in the schema.
    /// 
    /// Will return `true` if a value was found and succesfully set, `false` otherwise.
    /// 
    /// *WARNING* If you call this on a collection (Map, Tuple, List, or Table) ALL children will be overwritten/set.  The method is recursive, so this will hit *all* children, including nested children.
    /// 
    /// When this is applied to a `string` data type, only ascii values are supported.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "sorted": true,
    ///    "values": [
    ///         {"type": "string", "size": 10},
    ///         {"type": "u32"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut low_buffer = factory.empty_buffer(None);
    /// // set all types to minimum value
    /// low_buffer.set_min(&[])?;
    /// // get bytes
    /// let low_bytes: Vec<u8> = low_buffer.close();
    /// 
    /// let mut high_buffer = factory.empty_buffer(None);
    /// // set all types to max value
    /// high_buffer.set_max(&[])?;
    /// // get bytes
    /// let high_bytes: Vec<u8> = high_buffer.close();
    /// 
    /// let mut middle_buffer = factory.empty_buffer(None);
    /// middle_buffer.set(&["0"], "Light This Candle!");
    /// middle_buffer.set(&["1"], 22938u32);
    /// let middle_bytes: Vec<u8> = middle_buffer.close();
    /// 
    /// assert!(low_bytes < middle_bytes);
    /// assert!(middle_bytes < high_bytes);
    /// assert!(low_bytes < high_bytes);
    /// 
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn set_max(&mut self, path: &[&str]) -> Result<bool, NP_Error> {
        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), true, false, path)?;
        match value_cursor {
            Some(x) => {
                Ok(NP_Cursor::set_max(x, &self.memory)?)
            }
            None => Ok(false)
        }
    }

    /// Set the max value allowed for the specific data type at the given key.
    /// 
    /// String & Byte types only work if a `size` property is set in the schema.
    /// 
    /// Will return `true` if a value was found and succesfully set, `false` otherwise.
    /// 
    /// *WARNING* If you call this on a collection (Map, Tuple, List, or Table) ALL children will be overwritten/set.  The method is recursive, so this will hit *all* children, including nested children.
    /// 
    /// When this is applied to a `string` data type, only ascii values are supported.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "sorted": true,
    ///    "values": [
    ///         {"type": "string", "size": 10},
    ///         {"type": "u32"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut low_buffer = factory.empty_buffer(None);
    /// // set all types to minimum value
    /// low_buffer.set_min(&[])?;
    /// // get bytes
    /// let low_bytes: Vec<u8> = low_buffer.close();
    /// 
    /// let mut high_buffer = factory.empty_buffer(None);
    /// // set all types to max value
    /// high_buffer.set_max(&[])?;
    /// // get bytes
    /// let high_bytes: Vec<u8> = high_buffer.close();
    /// 
    /// let mut middle_buffer = factory.empty_buffer(None);
    /// middle_buffer.set(&["0"], "Light This Candle!");
    /// middle_buffer.set(&["1"], 22938u32);
    /// let middle_bytes: Vec<u8> = middle_buffer.close();
    /// 
    /// assert!(low_bytes < middle_bytes);
    /// assert!(middle_bytes < high_bytes);
    /// assert!(low_bytes < high_bytes);
    /// 
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn set_min(&mut self, path: &[&str]) -> Result<bool, NP_Error> {
        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), true, false, path)?;
        match value_cursor {
            Some(x) => {
                Ok(NP_Cursor::set_min(x, &self.memory)?)
            }
            None => Ok(false)
        }
    }

    /// Used to set scalar values inside the buffer.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///    "of": {"type": "map", "value": {
    ///         "type": "list", "of": {"type": "string"}
    ///     }}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // third item in the top level list -> key "alpha" of map at 3rd element -> 9th element of list at "alpha" key
    /// // 
    /// new_buffer.set(&["3", "alpha", "9"], "look at all this nesting madness")?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<&str>(&["3", "alpha", "9"])?;
    /// 
    /// assert_eq!(message, Some("look at all this nesting madness"));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn set<X: 'buffer>(&mut self, path: &[&str], value: X) -> Result<bool, NP_Error> where X: NP_Value<'buffer> + NP_Scalar<'buffer> {
        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), true, false, path)?;
        match value_cursor {
            Some(x) => {

                // type does not match schema
                if X::type_idx().1 != *self.memory.get_schema(x.schema_addr).get_type_key() {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") into schema of type (");
                    err.push_str(self.memory.get_schema(x.schema_addr).get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                X::set_value(x, &self.memory, value)?;
                Ok(true)
            }
            None => Ok(false)
        }
    }

    
    /// Get an iterator for a collection
    /// 
    /// 
    /// ## List Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set value at 1 index
    /// new_buffer.set(&["1"], "hello")?;
    /// // set value at 4 index
    /// new_buffer.set(&["4"], "world")?;
    /// // push value onto the end
    /// new_buffer.list_push(&[], "!")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         1 => assert_eq!(item.get::<&str>().unwrap(), Some("hello")),
    ///         2 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         3 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         4 => assert_eq!(item.get::<&str>().unwrap(), Some("world")),
    ///         5 => assert_eq!(item.get::<&str>().unwrap(), Some("!")),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Table Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "uint8"}],
    ///         ["name", {"type": "string"}],
    ///         ["job", {"type": "string"}],
    ///         ["tags", {"type": "list", "of": {"type": "string"}}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set value of age
    /// new_buffer.set(&["age"], 20u8)?;
    /// // set value of name
    /// new_buffer.set(&["name"], "Bill Kerman")?;
    /// // push value onto tags list
    /// new_buffer.list_push(&["tags"], "rocket")?;
    /// 
    /// // get iterator of root (table)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     
    ///     match item.key {
    ///         "name" => assert_eq!(item.get::<&str>().unwrap(), Some("Bill Kerman")),
    ///         "age" =>  assert_eq!(item.get::<u8>().unwrap(), Some(20)),
    ///         "job" => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         "tags" => { /* tags column is list, can't do anything with it here */ },
    ///         _ => { panic!() }
    ///     };
    /// });
    /// 
    /// // we can also loop through items of the tags list
    /// new_buffer.get_collection(&["tags"])?.unwrap().into_iter().for_each(|item| {
    ///     assert_eq!(item.index, 0);
    ///     assert_eq!(item.get::<&str>().unwrap(), Some("rocket"));
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Map Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set value of color key
    /// new_buffer.set(&["color"], "blue")?;
    /// // set value of sport key
    /// new_buffer.set(&["sport"], "soccor")?;
    /// 
    /// // get iterator of root (map)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     
    ///     match item.key {
    ///         "color" => assert_eq!(item.get::<&str>().unwrap(), Some("blue")),
    ///         "sport" => assert_eq!(item.get::<&str>().unwrap(), Some("soccor")),
    ///         _ => panic!()
    ///     }
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Tuple Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///     "values": [
    ///         {"type": "string"},
    ///         {"type": "u8"},
    ///         {"type": "bool"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set value at 0 index
    /// new_buffer.set(&["0"], "hello")?;
    /// // set value at 2 index
    /// new_buffer.set(&["2"], false)?;
    /// 
    /// // get iterator of root (tuple item)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<&str>().unwrap(), Some("hello")),
    ///         1 => assert_eq!(item.get::<u8>().unwrap(), None),
    ///         2 => assert_eq!(item.get::<bool>().unwrap(), Some(false)),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get_collection<'iter>(&'iter self, path: &'iter [&str]) -> Result<Option<NP_Generic_Iterator<'iter>>, NP_Error> {

        let value = NP_Cursor::select(&self.memory, self.cursor.clone(), false, false, path)?;

        let value = if let Some(x) = value {
            x
        } else {
            return Ok(None);
        };

        let value_data = value.get_value(&self.memory);

        // value doesn't exist
        if value_data.get_addr_value() == 0 {
            return Ok(None);
        }

        Ok(Some(NP_Generic_Iterator::new(value, &self.memory)?))
    }

    /// Push a value onto the end of a list.
    /// The path provided must resolve to a list type, and the type being pushed must match the schema
    /// 
    /// This is the most efficient way to add values to a list type.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// new_buffer.set(&["3"], "launch")?;
    /// new_buffer.list_push(&[], "this")?;
    /// new_buffer.list_push(&[], "rocket")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         1 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         2 => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         3 => assert_eq!(item.get::<&str>().unwrap(), Some("launch")),
    ///         4 => assert_eq!(item.get::<&str>().unwrap(), Some("this")),
    ///         5 => assert_eq!(item.get::<&str>().unwrap(), Some("rocket")),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// new_buffer.list_push(&[], "launch")?;
    /// new_buffer.list_push(&[], "this")?;
    /// new_buffer.list_push(&[], "rocket")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_collection(&[])?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<&str>().unwrap(), Some("launch")),
    ///         1 => assert_eq!(item.get::<&str>().unwrap(), Some("this")),
    ///         2 => assert_eq!(item.get::<&str>().unwrap(), Some("rocket")),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn list_push<X: 'buffer>(&mut self, path: &[&str], value: X) -> Result<Option<u16>, NP_Error> where X: NP_Value<'buffer> + NP_Scalar<'buffer> {

        let list_cursor = if path.len() == 0 { self.cursor.clone() } else { match NP_Cursor::select(&self.memory, self.cursor.clone(), true, false, path)? {
            Some(x) => x,
            None => return Ok(None)
        }};

        match &self.memory.get_schema(list_cursor.schema_addr) {
            NP_Parsed_Schema::List { of, .. } => {

                let of_schema = &self.memory.get_schema(*of);

                // type does not match schema
                if X::type_idx().1 != *of_schema.get_type_key() {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") into schema of type (");
                    err.push_str(of_schema.get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }
            },
            _ => return Err(NP_Error::new("Trying to push onto non list item!"))
        }

        match NP_List::push(&list_cursor, &self.memory, None)? {
            Some((index, new_item_addr)) => {
                X::set_value(new_item_addr, &self.memory, value)?;
                Ok(Some(index))
            },
            None => Ok(None)
        }
    }


    /// Get length of String, Bytes, Table, Tuple, List or Map Type
    /// 
    /// If the type found at the path provided does not support length operations, you'll get `None`.
    /// 
    /// If there is no value at the path provodid, you will get `None`.
    /// 
    /// If an item is found and it's length is zero, you can expect `Some(0)`.
    /// 
    /// ## String Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&[], "hello")?;
    /// // get length of value at root (String)
    /// assert_eq!(new_buffer.length(&[])?, Some(5));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (List) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set value at 9th index
    /// new_buffer.set(&["9"], "hello")?;
    /// // get length of value at root (List)
    /// assert_eq!(new_buffer.length(&[])?, Some(10));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Table) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "u8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // get length of value at root (Table)
    /// assert_eq!(new_buffer.length(&[])?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Map) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set values
    /// new_buffer.set(&["foo"], "bar")?;
    /// new_buffer.set(&["foo2"], "bar2")?;
    /// // get length of value at root (Map)
    /// assert_eq!(new_buffer.length(&[])?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Tuple) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "values": [
    ///         {"type": "string"}, 
    ///         {"type": "string"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // get length of value at root (Tuple)
    /// assert_eq!(new_buffer.length(&[])?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn length(&self, path: &[&str]) -> Result<Option<usize>, NP_Error> {
        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), false, false, path)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None);
        };

        let addr_value = found_cursor.get_value(&self.memory).get_addr_value();


        match &self.memory.get_schema(found_cursor.schema_addr) {
            NP_Parsed_Schema::List { of, .. } => {
                if addr_value == 0 {
                    return Ok(None);
                }

                let list_data = NP_List::get_list(addr_value as usize, &self.memory);
                let tail_addr = list_data.get_tail() as usize;
                if tail_addr == 0 {
                    Ok(Some(0))
                } else {
                    let tail_cursor = NP_Cursor::new(tail_addr, *of, found_cursor.schema_addr);
                    let cursor_data = tail_cursor.get_value(&self.memory);
                    Ok(Some(cursor_data.get_index() as usize + 1))
                }
            },
            NP_Parsed_Schema::Map { .. } => {
                if addr_value == 0 {
                    return Ok(None);
                }
                let mut count = 0usize;
                {
                    let mut map_iter = NP_Map::new_iter(&found_cursor, &self.memory);

                    // key is maybe in map
                    while let Some((_ikey, _item)) = map_iter.step_iter(&self.memory) {
                        count += 1;
                    }
                }

                Ok(Some(count))
            },
            NP_Parsed_Schema::Table { columns, ..} => {
                Ok(Some(columns.len()))
            },
            NP_Parsed_Schema::Tuple { values, .. } => {
                Ok(Some(values.len()))
            },
            NP_Parsed_Schema::Bytes {  size, ..} => {
                if *size > 0 {
                    Ok(Some(*size as usize))
                } else {
                    let length_bytes = self.memory.get_2_bytes(addr_value as usize).unwrap_or(&[0u8; 2]);
                    Ok(Some(u16::from_be_bytes(*length_bytes) as usize))
                }
            },
            NP_Parsed_Schema::UTF8String { size, .. } => {
                if *size > 0 {
                    Ok(Some(*size as usize))
                } else {
                    let length_bytes = self.memory.get_2_bytes(addr_value as usize).unwrap_or(&[0u8; 2]);
                    Ok(Some(u16::from_be_bytes(*length_bytes) as usize))
                }
            },
            _ => {
                Ok(None)
            }
        }
  
    }

    /// Clear an inner value from the buffer.
    /// This can also be used to clear deeply nested collection objects or scalar objects.
    /// 
    /// Returns `true` if it found a value to delete (and deleted it), `false` otherwise.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set index 0
    /// new_buffer.set(&["0"], "hello")?;
    /// // del index 0
    /// new_buffer.del(&["0"])?;
    /// // value is gone now!
    /// assert_eq!(None, new_buffer.get::<&str>(&["0"])?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn del(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), false, false, path)?;

        let is_sortable = match &self.memory.get_schema(0) {
            NP_Parsed_Schema::Tuple { sortable , ..} => *sortable,
            _ => false
        };
        
        match value_cursor {
            Some(x) => {
                if is_sortable {
                    match &self.memory.get_schema(x.schema_addr) {
                        NP_Parsed_Schema::Table { .. } => { return Ok(false) },
                        NP_Parsed_Schema::Tuple { .. } => { return Ok(false) },
                        NP_Parsed_Schema::List { .. } => { return Ok(false) },
                        NP_Parsed_Schema::Map { .. } => { return Ok(false) },
                        _ => NP_Cursor::set_schema_default(x, &self.memory)?
                    }
                } else {
                    // clear value address in buffer
                    x.get_value(&self.memory).set_addr_value(0);
                }

                Ok(true)
            }
            None => Ok(false)
        }
    }

    /// Retrieve the schema type at a given path.
    /// 
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::schema::NP_TypeKeys;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "values": [
    ///         {"type": "geo8"},
    ///         {"type": "dec", "exp": 2},
    ///         {"type": "string"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// 
    /// assert_eq!(new_buffer.get_schema_type(&[])?.unwrap(), NP_TypeKeys::Tuple);
    /// assert_eq!(new_buffer.get_schema_type(&["0"])?.unwrap(), NP_TypeKeys::Geo);
    /// assert_eq!(new_buffer.get_schema_type(&["1"])?.unwrap(), NP_TypeKeys::Decimal);
    /// assert_eq!(new_buffer.get_schema_type(&["2"])?.unwrap(), NP_TypeKeys::UTF8String);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get_schema_type(&self, path: &[&str]) -> Result<Option<NP_TypeKeys>, NP_Error> {

        match NP_Cursor::select(&self.memory, self.cursor.clone(), false, true, path)? {
            Some(x) => {
                Ok(Some(*self.memory.get_schema(x.schema_addr).get_type_key()))
            }
            None => Ok(None)
        }
    }

    /// Retrieve the schema default at a given path.
    /// 
    /// This is useful for `geo` and `dec` data types where there is information about the value in the schema.
    /// 
    /// For example, when you create an `NP_Geo` type to put into a `geo` field, you must know the resolution (4/8/16).  If you use this method you can get an empty `NP_Geo` type that already has the correct resolution set based on the schema.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::pointer::dec::NP_Dec;
    /// use no_proto::pointer::geo::NP_Geo;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "values": [
    ///         {"type": "geo8"},
    ///         {"type": "dec", "exp": 2}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // Get an empty NP_Geo type that has the correct resolution for the schema
    /// // 
    /// let geo_default: NP_Geo = new_buffer.get_schema_default::<NP_Geo>(&["0"])?.unwrap();
    /// assert_eq!(geo_default.size, 8); // geo is size 8 in schema
    /// 
    /// // Get an empty NP_Dec type that has the correct exp for the schema
    /// // 
    /// let dec_default: NP_Dec = new_buffer.get_schema_default::<NP_Dec>(&["1"])?.unwrap();
    /// assert_eq!(dec_default.exp, 2); // exponent is 2 in schema
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get_schema_default<'get, X: 'get>(&'get self, path: &[&str]) -> Result<Option<X>, NP_Error> where X: NP_Value<'get> + NP_Scalar<'get> {

        match NP_Cursor::select(&self.memory, self.cursor.clone(), false, true, path)? {
            Some(x) => {
                                
                // type does not match schema
                if X::type_idx().1 != *self.memory.get_schema(x.schema_addr).get_type_key() {
                    let mut err = "TypeError: Attempted to get schema for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") for schema of type (");
                    err.push_str(self.memory.get_schema(x.schema_addr).get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                Ok(X::schema_default(&self.memory.get_schema(x.schema_addr)))
            }
            None => Ok(None)
        }
    }
  
    /// Retrieve an inner value from the buffer. 
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///    "of": {"type": "map", "value": {
    ///         "type": "list", "of": {"type": "string"}
    ///     }}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // third item in the top level list -> key "alpha" of map at 3rd element -> 9th element of list at "alpha" key
    /// // 
    /// new_buffer.set(&["3", "alpha", "9"], "who would build a schema like this")?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<&str>(&["3", "alpha", "9"])?;
    /// 
    /// assert_eq!(message, Some("who would build a schema like this"));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get<'get, X: 'get>(&'get self, path: &[&str]) -> Result<Option<X>, NP_Error> where X: NP_Value<'get> + NP_Scalar<'get> {
        let value_cursor = NP_Cursor::select(&self.memory, self.cursor.clone(), false, false, path)?;

        match value_cursor {
            Some(x) => {
                                
                // type does not match schema
                if X::type_idx().1 != *self.memory.get_schema(x.schema_addr).get_type_key() {
                    let mut err = "TypeError: Attempted to get value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") for schema of type (");
                    err.push_str(self.memory.get_schema(x.schema_addr).get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                match X::into_value(&x, &self.memory)? {
                    Some(x) => {
                        Ok(Some(x))
                    },
                    None => { // no value found here, return default from schema
                        match X::default_value(0, x.schema_addr, &self.memory.get_schemas()) {
                            Some(y) => {
                                Ok(Some(y))
                            },
                            None => { // no default in schema, no value to provide
                                Ok(None)
                            }
                        }                        
                    }
                }
            }
            None => Ok(None)
        }
    }

    /// This performs a compaction if the closure provided as the second argument returns `true`.
    /// Compaction is a pretty expensive operation (requires full copy of the whole buffer) so should be done sparingly.
    /// The closure is provided an argument that contains the original size of the buffer, how many bytes could be saved by compaction, and how large the new buffer would be after compaction.  The closure should return `true` to perform compaction, `false` otherwise.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// **WARNING** Your cursor location and backup will be reset to the root.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&[], "hello")?;
    /// // using 9 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set(&[], "hello, world")?;
    /// // now using 25 bytes, with 7 bytes of wasted space
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 25,
    ///     after_compaction: 18,
    ///     wasted_bytes: 7
    /// }, new_buffer.calc_bytes()?);
    /// // compact to save space
    /// new_buffer.maybe_compact(None, |compact_data| {
    ///     // only compact if wasted bytes are greater than 5
    ///     if compact_data.wasted_bytes > 5 {
    ///         true
    ///     } else {
    ///         false
    ///     }
    /// })?;
    /// // back down to 18 bytes with no wasted bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 18,
    ///     after_compaction: 18,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn maybe_compact<F>(&mut self, new_capacity: Option<u32>, mut callback: F) -> Result<(), NP_Error> where F: FnMut(NP_Size_Data) -> bool {

        let bytes_data = self.calc_bytes()?;

        if callback(bytes_data) {
            self.compact(new_capacity)?;
        }

        return Ok(());
    }

    /// Compacts a buffer to remove an unused bytes or free space after a mutation.
    /// This is a pretty expensive operation (requires full copy of the whole buffer) so should be done sparingly.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// **WARNING** Your cursor location will be reset to the root.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// // set initial value
    /// new_buffer.set(&[], "hello")?;
    /// // using 11 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set(&[], "hello, world")?;
    /// // now using 25 bytes, with 7 bytes of wasted bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 25,
    ///     after_compaction: 18,
    ///     wasted_bytes: 7
    /// }, new_buffer.calc_bytes()?);
    /// // compact to save space
    /// new_buffer.compact(None)?;
    /// // back down to 18 bytes with no wasted bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 18,
    ///     after_compaction: 18,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn compact<'compact>(&mut self, new_capacity: Option<u32>) -> Result<(), NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let old_root = NP_Cursor::new(self.memory.root, 0, 0);

        let new_bytes = NP_Memory_Writable::new_owned(Some(capacity), self.memory.schema.get().clone(), self.memory.root);
        let new_root  = NP_Cursor::new(self.memory.root, 0, 0);

        NP_Cursor::compact(0, old_root, &self.memory, new_root, &new_bytes)?;

        self.cursor = NP_Cursor::new(self.memory.root, 0, 0);

        self.memory = new_bytes;

        Ok(())
    }

    /// Recursively measures how many bytes each element in the buffer is using.
    /// This will let you know how many bytes can be saved from a compaction.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None);
    /// new_buffer.set(&[], "hello")?;
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn calc_bytes<'bytes>(&self) -> Result<NP_Size_Data, NP_Error> {

        let root = NP_Cursor::new(self.memory.root, 0, 0);
        let real_bytes = NP_Cursor::calc_size(0, &root, &self.memory)? + self.memory.root;
        let total_size = self.memory.read_bytes().len();

        if total_size >= real_bytes {
            return Ok(NP_Size_Data {
                current_buffer: total_size,
                after_compaction: real_bytes,
                wasted_bytes: total_size - real_bytes
            });
        } else {
            return Err(NP_Error::new("Error calculating bytes!"));
        }
    }
}

/// NP Item
pub struct NP_Item<'item> {
    /// index of this value
    pub index: usize,
    /// Key at this index
    pub key: &'item str,
    /// Column at this index
    pub col: &'item str,
    /// Cursor value
    cursor: Option<NP_Cursor>,
    parent: NP_Cursor,
    memory: &'item NP_Memory_Writable<'item>
}

impl<'item> NP_Item<'item> {

    /// If this item has a value
    pub fn has_value(&self) -> bool {
        if let Some(x) = self.cursor {
            let value = x.get_value(self.memory);
            value.get_addr_value() != 0
        } else {
            false
        }
    }
    /// Get value at this pointer
    pub fn get<X>(&'item self) -> Result<Option<X>, NP_Error> where X: NP_Value<'item> + NP_Scalar<'item> {
        if let Some(cursor) = self.cursor {
            match X::into_value(&cursor, self.memory)? {
                Some(x) => {
                    Ok(Some(x))
                },
                None => {
                    match X::default_value(0, cursor.schema_addr, &self.memory.get_schemas()) {
                        Some(y) => {
                            Ok(Some(y))
                        },
                        None => {
                            Ok(None)
                        }
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Set value at this pointer
    pub fn set<X>(&'item mut self, value: X) -> Result<(), NP_Error> where X: NP_Value<'item> + NP_Scalar<'item> {
        if let Some(cursor) = self.cursor {
            X::set_value(cursor.clone(), self.memory, value)?;
        } else {
            match self.memory.get_schema(self.parent.schema_addr) {
                // maps don't let you select values that don't exist in the buffer yet
                NP_Parsed_Schema::List { .. } => {
                    let item = opt_err(opt_err(NP_List::select(self.parent.clone(), self.index, true, false, self.memory)?)?.1)?;
                    X::set_value(item, self.memory, value)?;
                }
                NP_Parsed_Schema::Table { columns, .. } => {
                    let item = opt_err(NP_Table::select(self.parent.clone(), columns, &self.key, true, false, self.memory)?)?;
                    X::set_value(item, self.memory, value)?;
                },
                NP_Parsed_Schema::Tuple { values, .. } => {
                    let item = opt_err(NP_Tuple::select(self.parent.clone(), values, self.index, true, false, self.memory)?)?;
                    X::set_value(item, self.memory, value)?;
                }
                _ => { }
            }
        }

        Ok(())
    }

    /// Clear the value at this pointer
    pub fn del(&'item mut self) -> bool {
        if let Some(cursor) = self.cursor {
            let value = cursor.get_value(self.memory);
            value.set_addr_value(0);
            true
        } else {
            false
        }
    }
}



/// Iterator Enum
#[derive(Debug)]
#[doc(hidden)]
pub enum NP_Iterator_Collection<'col> {
    /// None
    None,
    /// Map
    Map(NP_Map<'col>),
    /// List
    List(NP_List),
    /// Table
    Table(NP_Table<'col>),
    /// Tuple
    Tuple(NP_Tuple<'col>)
}

#[allow(missing_docs)]
impl<'col> NP_Iterator_Collection<'col> {
    pub fn new<M: NP_Memory>(cursor: NP_Cursor, memory: &'col M) -> Result<Self, NP_Error> {
        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Table { .. } => {
                let table = NP_Table::new_iter(&cursor, memory);
                Ok(NP_Iterator_Collection::Table(table))
            },
            NP_Parsed_Schema::List { .. } => {
                let list = NP_List::new_iter(&cursor, memory, false, 0);
                Ok(NP_Iterator_Collection::List(list))
            },
            NP_Parsed_Schema::Tuple { .. } => {
                let tuple = NP_Tuple::new_iter(&cursor, memory);
                Ok(NP_Iterator_Collection::Tuple(tuple))
            },
            NP_Parsed_Schema::Map { .. } => {
                let map = NP_Map::new_iter(&cursor, memory);
                Ok(NP_Iterator_Collection::Map(map))
            },
            _ => Err(NP_Error::new("Tried to create iterator on non collection item!"))
        }
    }
}

#[allow(missing_docs)]
pub struct NP_Generic_Iterator<'it> {
    root: NP_Cursor,
    value: NP_Iterator_Collection<'it>,
    memory: &'it NP_Memory_Writable<'it>,
    index: usize
}

#[allow(missing_docs)]
impl<'it> NP_Generic_Iterator<'it> {
    pub fn new(cursor: NP_Cursor, memory: &'it NP_Memory_Writable<'it>) -> Result<Self, NP_Error> {
        Ok(Self { 
            root: cursor.clone(),
            value: NP_Iterator_Collection::new(cursor.clone(), memory)?,
            memory: memory,
            index: 0
        })
    }
}


impl<'it> Iterator for NP_Generic_Iterator<'it> {
    type Item = NP_Item<'it>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.value {
            NP_Iterator_Collection::Map(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    self.index += 1;
                    Some(NP_Item { memory: self.memory, key: next_item.0, col: next_item.0, index: self.index - 1, cursor: Some(next_item.1), parent: self.root.clone() })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::List(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: "", col: "", index: next_item.0, cursor: next_item.1, parent: self.root.clone() })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Table(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: next_item.1, col: next_item.1, index: next_item.0, cursor: next_item.2, parent: self.root.clone() })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Tuple(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: "", col: "", index: next_item.0, cursor: next_item.1, parent: self.root.clone() })
                } else {
                    None
                }
            },
            _ => { None }
        }
    }
}