//! Top level abstraction for buffer objects (read only)
//! 
//! 
//! 

use crate::NP_Size_Data;
use crate::{NP_Memory_Writable, buffer::NP_Buffer};
use crate::{memory::NP_Memory_ReadOnly, utils::opt_err};
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
pub const DEFAULT_ROOT_PTR_ADDR: usize = 1;
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
pub struct NP_Buffer_RO<'buffer> {
    /// Schema data used by this buffer
    memory: NP_Memory_ReadOnly<'buffer>,
    cursor: NP_Cursor
}

impl<'buffer> Clone for NP_Buffer_RO<'buffer> {
    fn clone(&self) -> Self {
        Self {
            memory: self.memory.clone(),
            cursor: self.cursor.clone()
        }
    }
}

impl<'buffer> NP_Buffer_RO<'buffer> {

    #[doc(hidden)]
    pub fn _new(memory: NP_Memory_ReadOnly<'buffer>) -> Self { // make new buffer

        NP_Buffer_RO {
            cursor: NP_Cursor::new(memory.root, 0, 0),
            memory: memory
        }
    }

    /// Copy this read only buffer into a writable one
    /// 
    pub fn get_writable(&self) -> NP_Buffer {
        NP_Buffer::_new(NP_Memory_Writable::existing(self.read_bytes().to_vec(), self.memory.get_schemas(), self.memory.get_root()))
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
    /// let new_buffer = factory.open_buffer_ro(new_buffer.read_bytes());
    /// 
    /// assert_eq!("{\"age\":30,\"name\":\"Jeb Kermin\"}", new_buffer.json_encode(&[])?.stringify());
    /// assert_eq!("\"Jeb Kermin\"", new_buffer.json_encode(&["name"])?.stringify());
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn json_encode(&self, path: &[&str]) -> Result<NP_JSON, NP_Error> {

        let value_cursor = self.select(self.cursor.clone(), path)?;

        if let Some(x) = value_cursor {
            Ok(NP_Cursor::json_encode(&x, &self.memory))
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
    /// let new_buffer = factory.open_buffer_ro(new_buffer.read_bytes());
    /// // close buffer and get bytes
    /// let bytes: Vec<u8> = new_buffer.close();
    /// assert_eq!([0, 0, 3, 0, 5, 104, 101, 108, 108, 111].to_vec(), bytes);
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
    /// // the buffer with it's vtables take up 20 bytes!
    /// assert_eq!(new_buffer.read_bytes().len(), 20usize);
    /// 
    /// // close buffer and get sortable bytes
    /// let bytes: Vec<u8> = factory.open_buffer_ro(new_buffer.read_bytes()).close_sortable()?;
    /// // with close_sortable() we only get the bytes we care about!
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
                    let root_offset = DEFAULT_ROOT_PTR_ADDR + 2 + (vtables * 10);

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
    /// If you attempt to move into a path that doesn't exist, this method will return `false`. 
    /// 
    pub fn move_cursor(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        let value_cursor = self.select(self.cursor.clone(), path)?;

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
    /// factory.open_buffer_ro(new_buffer.read_bytes()).get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    /// factory.open_buffer_ro(new_buffer.read_bytes()).get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    /// factory.open_buffer_ro(new_buffer.read_bytes()).get_iter(&["tags"])?.unwrap().into_iter().for_each(|item| {
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
    /// factory.open_buffer_ro(new_buffer.read_bytes()).get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    /// factory.open_buffer_ro(new_buffer.read_bytes()).get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    pub fn get_iter<'iter>(&'iter self, path: &'iter [&str]) -> Result<Option<NP_Generic_Iterator<'iter>>, NP_Error> {

        let value = self.select(self.cursor.clone(), path)?;

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
    /// assert_eq!(factory.open_buffer_ro(new_buffer.read_bytes()).length(&[])?, Some(5));
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
    /// assert_eq!(factory.open_buffer_ro(new_buffer.read_bytes()).length(&[])?, Some(10));
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
    /// assert_eq!(factory.open_buffer_ro(new_buffer.read_bytes()).length(&[])?, Some(2));
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
    /// assert_eq!(factory.open_buffer_ro(new_buffer.read_bytes()).length(&[])?, Some(2));
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
    /// assert_eq!(factory.open_buffer_ro(new_buffer.read_bytes()).length(&[])?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn length(&self, path: &[&str]) -> Result<Option<usize>, NP_Error> {
        let value_cursor = self.select(self.cursor.clone(), path)?;

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
    /// let ro_buffer = factory.open_buffer_ro(new_buffer.read_bytes());
    /// let message = ro_buffer.get::<&str>(&["3", "alpha", "9"])?;
    /// 
    /// assert_eq!(message, Some("who would build a schema like this"));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get<'get, X: 'get>(&'get self, path: &[&str]) -> Result<Option<X>, NP_Error> where X: NP_Value<'get> + NP_Scalar {
        let value_cursor = self.select(self.cursor.clone(), path)?;

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
                        match X::schema_default(&self.memory.get_schema(x.schema_addr)) {
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
    /// 
    /// For read only buffers the compaction method compacts into a writable buffer.
    /// 
    /// The closure is provided an argument that contains the original size of the buffer, how many bytes could be saved by compaction, and how large the new buffer would be after compaction.  The closure should return `true` to perform compaction, `false` otherwise.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// **WARNING** Your cursor location will be reset to the root.
    /// 
    /// 
    pub fn maybe_compact<F>(&mut self, new_capacity: Option<usize>, mut callback: F) -> Result<NP_Buffer, NP_Error> where F: FnMut(NP_Size_Data) -> bool {

        let bytes_data = self.calc_bytes()?;

        if callback(bytes_data) {
            return self.compact(new_capacity);
        }

        return Ok(NP_Buffer::_new(NP_Memory_Writable::new(new_capacity, self.memory.schema, self.memory.get_root())));
    }

    /// Compacts a buffer to remove an unused bytes or free space after a mutation.
    /// This is a pretty expensive operation (requires full copy of the whole buffer) so should be done sparingly.
    /// 
    /// For read only buffers the compaction method compacts into a writable buffer.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// **WARNING** Your cursor location will be reset to the root.
    /// 
    /// 
    pub fn compact<'compact>(&mut self, new_capacity: Option<usize>) -> Result<NP_Buffer, NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let old_root = NP_Cursor::new(self.memory.root, 0, 0);

        let new_bytes = NP_Memory_Writable::new(Some(capacity), self.memory.schema, self.memory.root);
        let new_root  = NP_Cursor::new(self.memory.root, 0, 0);

        NP_Cursor::compact(old_root, &self.memory, new_root, &new_bytes)?;

        self.cursor = NP_Cursor::new(self.memory.root, 0, 0);

        // self.memory = new_bytes;

        Ok(NP_Buffer::_new(new_bytes))
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
    ///     current_buffer: 10,
    ///     after_compaction: 10,
    ///     wasted_bytes: 0
    /// }, factory.open_buffer_ro(new_buffer.read_bytes()).calc_bytes()?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn calc_bytes<'bytes>(&self) -> Result<NP_Size_Data, NP_Error> {

        let root = NP_Cursor::new(self.memory.root, 0, 0);
        let real_bytes = NP_Cursor::calc_size(&root, &self.memory)? + self.memory.root;
        let total_size = self.memory.read_bytes().len() - self.memory.root + 1;
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


    fn select(&self, cursor: NP_Cursor, path: &[&str]) -> Result<Option<NP_Cursor>, NP_Error> {

        let mut loop_cursor = cursor;

        let mut path_index = 0usize;

        let mut loop_count = 0u16;
        
        loop {

            loop_count += 1;
            
            if path.len() == path_index {
                return Ok(Some(loop_cursor));
            }

            if loop_count > 256 {
                return Err(NP_Error::new("Select overflow"))
            }

            // now select into collections
            match &self.memory.get_schema(loop_cursor.schema_addr) {
                NP_Parsed_Schema::Table { columns, .. } => {
                    if let Some(next) = NP_Table::select(loop_cursor, columns, path[path_index], false, &self.memory)? {
                        loop_cursor = next;
                        path_index += 1;
                    } else {
                        return Ok(None);
                    }
                },
                NP_Parsed_Schema::Tuple { values, .. } => {
                    match path[path_index].parse::<usize>() {
                        Ok(x) => {
                            if let Some(next) = NP_Tuple::select(loop_cursor, values, x, false, &self.memory)? {
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
                            if let Some(next) = NP_List::select(loop_cursor, x, false, &self.memory)? {
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
                NP_Parsed_Schema::Map {  .. } => {
                    if let Some(next) = NP_Map::select(loop_cursor, path[path_index], false, &self.memory)? {
                        loop_cursor = next;
                        path_index += 1;
                    } else {
                        return Ok(None);
                    }

                },
                _ => { // we've reached a scalar value but not at the end of the path
                    return Ok(None);
                }
            }
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
    memory: &'item NP_Memory_ReadOnly<'item>
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
    pub fn get<X>(&'item self) -> Result<Option<X>, NP_Error> where X: NP_Value<'item> + NP_Scalar {
        if let Some(cursor) = self.cursor {
            match X::into_value(&cursor, self.memory)? {
                Some(x) => {
                    Ok(Some(x))
                },
                None => {
                    match X::schema_default(&self.memory.get_schema(cursor.schema_addr)) {
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
        match &memory.get_schema(cursor.schema_addr) {
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
    value: NP_Iterator_Collection<'it>,
    memory: &'it NP_Memory_ReadOnly<'it>,
    index: usize
}

#[allow(missing_docs)]
impl<'it> NP_Generic_Iterator<'it> {
    pub fn new(cursor: NP_Cursor, memory: &'it NP_Memory_ReadOnly<'it>) -> Result<Self, NP_Error> {
        Ok(Self { 
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
                    Some(NP_Item { memory: self.memory, key: next_item.0, col: next_item.0, index: self.index - 1, cursor: Some(next_item.1) })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::List(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: "", col: "", index: next_item.0, cursor: next_item.1 })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Table(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: next_item.1, col: next_item.1, index: next_item.0, cursor: next_item.2 })
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Tuple(x) => {
                if let Some(next_item) = x.step_iter(self.memory) {
                    Some(NP_Item { memory: self.memory, key: "", col: "", index: next_item.0, cursor: next_item.1 })
                } else {
                    None
                }
            },
            _ => { None }
        }
    }
}