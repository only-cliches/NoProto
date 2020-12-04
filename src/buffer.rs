//! Top level abstraction for buffer objects

use core::hint::unreachable_unchecked;

use crate::pointer::{NP_Cursor_Addr, NP_Cursor_Data, NP_Cursor_Value, NP_Scalar};
use crate::{collection::map::NP_Map, utils::print_path};
use crate::{schema::NP_TypeKeys, pointer::NP_Value};
use crate::pointer::NP_Cursor;
use alloc::string::String;
use crate::{collection::{tuple::NP_Tuple}};
use crate::{schema::NP_Parsed_Schema, collection::table::NP_Table};
use alloc::vec::Vec;
use crate::{collection::{list::NP_List}};
use crate::error::NP_Error;
use crate::memory::{NP_Memory};
use crate::{json_flex::NP_JSON};
use crate::alloc::borrow::ToOwned;
use alloc::string::ToString;

/// The address location of the root pointer.
#[doc(hidden)]
pub const ROOT_PTR_ADDR: usize = 2;
/// Maximum size of list collections
#[doc(hidden)]
pub const LIST_MAX_SIZE: usize = core::u16::MAX as usize;

/// Buffers contain the bytes of each object and allow you to perform reads, updates, deletes and compaction.
/// 
/// 
#[derive(Debug)]
pub struct NP_Buffer<'buffer> {
    /// Schema data used by this buffer
    memory: NP_Memory<'buffer>,
    cursor_addr: NP_Cursor_Addr
}

/// When calling `maybe_compact` on a buffer, this struct is provided to help make a choice on wether to compact or not.
#[derive(Debug, Eq, PartialEq)]
pub struct NP_Size_Data {
    /// The size of the existing buffer
    pub current_buffer: usize,
    /// The estimated size of buffer after compaction
    pub after_compaction: usize,
    /// How many known wasted bytes in existing buffer
    pub wasted_bytes: usize
}

impl<'buffer> NP_Buffer<'buffer> {

    #[doc(hidden)]
    pub fn _new(memory: NP_Memory<'buffer>) -> Result<Self, NP_Error> { // make new buffer

        // Parse root
        NP_Cursor::parse(ROOT_PTR_ADDR, 0, 0, &memory)?;

        Ok(NP_Buffer {
            cursor_addr: NP_Cursor_Addr::Real(ROOT_PTR_ADDR),
            memory: memory
        })
    }


    /// Copy an object at the provided path and all it's children into JSON.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "uint8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// new_buffer.set(&["name"], "Jeb Kermin");
    /// new_buffer.set(&["age"], 30u8);
    /// 
    /// assert_eq!("{\"name\":\"Jeb Kermin\",\"age\":30}", new_buffer.json_encode(&[])?.stringify());
    /// assert_eq!("\"Jeb Kermin\"", new_buffer.json_encode(&["name"])?.stringify());
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn json_encode(&self, path: &'buffer [&str]) -> Result<NP_JSON, NP_Error> {

        let value_cursor = self.select(self.cursor.clone(), false, path, 0)?;

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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // set initial value
    /// new_buffer.set(&[], "hello")?;
    /// // close buffer and get bytes
    /// let bytes: Vec<u8> = new_buffer.close();
    /// assert_eq!([1, 1, 0, 4, 0, 5, 104, 101, 108, 108, 111].to_vec(), bytes);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn close(self) -> Vec<u8> {
        self.memory.dump()
    }

    /// Read the bytes of the buffer immutably.  No touching!
    /// 
    pub fn read_bytes(&self) -> &Vec<u8> {
        self.memory.read_bytes()
    }

    /// Move buffer cursor to new location.  Cursors can only be moved into children.  If you need to move up reset the cursor to root, then move back down to the desired level.
    /// 
    pub fn move_cursor(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        let value_cursor = self.select(self.cursor.clone(), true, path, 0)?;

        let cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(false);
        };

        self.cursor = cursor;

        Ok(true)
    }

    /// Reset cursor position to root of buffer
    /// 
    pub fn cursor_to_root(&mut self) {
        self.cursor_addr = NP_Cursor_Addr::Real(ROOT_PTR_ADDR);
    }

    /// Used to set scalar values inside the buffer, the path only works with dot notation.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///    "of": {"type": "map", "value": {
    ///         "type": "list", "of": {"type": "string"}
    ///     }}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    pub fn set<X>(&mut self, path: &[&str], value: X) -> Result<bool, NP_Error> where X: NP_Value<'buffer> + NP_Scalar {

        let value_cursor = self.select(self.cursor.clone(), true, path, 0)?;
        match value_cursor {
            Some(x) => {
                if path.len() == 0 {
                    self.cursor = X::set_value(x, &self.memory, value)?;
                } else {
                    X::set_value(x, &self.memory, value)?;
                }
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // set value at 1 index
    /// new_buffer.set(&["1"], "hello")?;
    /// // set value at 4 index
    /// new_buffer.set(&["4"], "world")?;
    /// // push value onto the end
    /// new_buffer.list_push(&[], "!")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    /// use no_proto::buffer::NP_Size_Data;
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
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // set value of age
    /// new_buffer.set(&["age"], 20u8)?;
    /// // set value of name
    /// new_buffer.set(&["name"], "Bill Kerman")?;
    /// // push value onto tags list
    /// new_buffer.list_push(&["tags"], "rocket")?;
    /// 
    /// // get iterator of root (table)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
    ///     
    ///     match item.get_key() {
    ///         "name" => assert_eq!(item.get::<&str>().unwrap(), Some("Bill Kerman")),
    ///         "age" =>  assert_eq!(item.get::<u8>().unwrap(), Some(20)),
    ///         "job" => assert_eq!(item.get::<&str>().unwrap(), None),
    ///         "tags" => { /* tags column is list, can't do anything with it here */ },
    ///         _ => { panic!() }
    ///     };
    /// });
    /// 
    /// // we can also loop through items of the tags list
    /// new_buffer.get_iter(&["tags"])?.unwrap().into_iter().for_each(|item| {
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // set value of color key
    /// new_buffer.set(&["color"], "blue")?;
    /// // set value of sport key
    /// new_buffer.set(&["sport"], "soccor")?;
    /// 
    /// // get iterator of root (map)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
    ///     println!("hello");
    ///     match item.get_key() {
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
    /// use no_proto::buffer::NP_Size_Data;
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
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // set value at 0 index
    /// new_buffer.set(&["0"], "hello")?;
    /// // set value at 2 index
    /// new_buffer.set(&["2"], false)?;
    /// 
    /// // get iterator of root (tuple item)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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

        let value = self.select(self.cursor.clone(), false, path, 0)?;

        let mut value = if let Some(x) = value {
            x
        } else {
            return Ok(None);
        };

        let value_addr = if value.buff_addr != 0 { self.memory.read_address(value.buff_addr) } else { 0 };
        value.value = value.value.update_value_address(value_addr);

        // value doesn't exist
        if value.value.get_value_address() == 0 {
            return Ok(None);
        }

        match self.memory.schema[value.schema_addr] {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Table(NP_Table::new(value, &self.memory))
                }))
            },
            NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Map(NP_Map::new(value, &self.memory))
                }))
            },
            NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::List(NP_List::new(value, &self.memory, false))
                }))
            },
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Tuple(NP_Tuple::new(value, &self.memory))
                }))
            },
            _ => {
                Err(NP_Error::new("Attempted to ierate on non collection!"))
            }
        }
        
    
    }



    /// Allows quick and efficient inserting into lists, maps and tables.
    /// CAREFUL.  Unlike the `set` method, **this one does not check for existing records with the provided key**.  This works for `Lists`, `Maps` and `Tables`.
    /// 
    /// You get a very fast insert into the buffer at the desired key, but **you must gaurantee that you're not inserting a key that's already been inserted.**.
    /// 
    /// This method will let you insert duplicate keys all day long, then when you go to compact the buffer the **duplicates will not be deleted**.
    /// 
    /// This is best used to quickly fill data into a new buffer.
    /// 
    /// 
    pub fn fast_insert<X>(&mut self, key: &str, value: X) -> Result<bool, NP_Error> where X: NP_Value<'buffer> + NP_Scalar {
        let collection_cursor = self.cursor.clone();

        match self.memory.schema[collection_cursor.schema_addr] {
            NP_Parsed_Schema::List { of, .. } => { // list push

                let of_schema = &self.memory.schema[of];

                // type does not match schema
                if X::type_idx().1 != *of_schema.get_type_key() {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") into schema of type (");
                    err.push_str(of_schema.get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                let (_new_index, new_cursor) = NP_List::push(collection_cursor, &self.memory, None)?;

                X::set_value(new_cursor, &self.memory, value)?;

                Ok(true)
            },
            NP_Parsed_Schema::Map { .. } => {
                let new_cursor = NP_Map::select_into(collection_cursor, &self.memory, key, true, true)?;
                X::set_value(new_cursor, &self.memory, value)?;
                Ok(true)
            },
            NP_Parsed_Schema::Table { .. } => {
                let new_cursor = NP_Table::select_into(collection_cursor, &self.memory, key, true, true)?;
                match new_cursor {
                    Some(x) => { 
                        X::set_value(x, &self.memory, value)?;
                        Ok(true)
                    },
                    None => Ok(false)
                }
            },
            _ => Ok(false)
        }
    }

    /// Push a value onto the end of a list.
    /// The path provided must resolve to a list type, and the type being pushed must match the schema
    /// 
    /// This is the most efficient way to add values to a list type.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// new_buffer.set(&["3"], "launch")?;
    /// new_buffer.list_push(&[], "this")?;
    /// new_buffer.list_push(&[], "rocket")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// new_buffer.list_push(&[], "launch")?;
    /// new_buffer.list_push(&[], "this")?;
    /// new_buffer.list_push(&[], "rocket")?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
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
    pub fn list_push<X>(&mut self, path: &[&str], value: X) -> Result<Option<u16>, NP_Error> where X: NP_Value<'buffer> + NP_Scalar {

        let list_cursor = if path.len() == 0 { self.cursor.clone() } else { match self.select(self.cursor.clone(), true, path, 0)? {
            Some(x) => x,
            None => return Ok(None)
        }};

        match self.memory.schema[list_cursor.schema_addr] {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {

                let of_schema = &self.memory.schema[of];

                // type does not match schema
                if X::type_idx().1 != *of_schema.get_type_key() {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") into schema of type (");
                    err.push_str(of_schema.get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                let (new_index, new_cursor) = NP_List::push(list_cursor, &self.memory, None)?;

                X::set_value(new_cursor, &self.memory, value)?;

                Ok(Some(new_index as u16))
            },
            _ => Ok(None)
        }
    }

    /// Get the schema info at a specific path, works for an type
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::schema::NP_TypeKeys;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // get schema of root
    /// let type_key = new_buffer.get_type(&[])?.unwrap();
    /// 
    /// let is_string = match type_key {
    ///     NP_TypeKeys::UTF8String  => {
    ///         true
    ///     },
    ///     _ => false
    /// };
    /// 
    /// assert!(is_string);
    /// 
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get_type(&self, path: &[&str]) -> Result<Option<&'buffer NP_TypeKeys>, NP_Error> {
        let value_cursor = self.select(self.cursor.clone(), false, path, 0)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None)
        };

        Ok(Some(self.memory.schema[found_cursor.schema_addr].get_type_key()))
    }

    /// Get length of String, Bytes, Table, Tuple, List or Map Type
    /// 
    /// If the type found at the path provided does not support length operations, you'll get `None`.
    /// 
    /// If the length of the item is zero, you can expect `Some(0)`.
    /// 
    /// ## String Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "u8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "values": [
    ///         {"type": "string"}, 
    ///         {"type": "string"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
    /// // get length of value at root (Tuple)
    /// assert_eq!(new_buffer.length(&[])?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn length(&self, path: &[&str]) -> Result<Option<usize>, NP_Error> {
        let value_cursor = self.select(self.cursor.clone(), false, path, 0)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None)
        };

        if found_cursor.buff_addr == 0 {
            return Ok(None);
        }

        match &self.memory.schema[found_cursor.schema_addr] {
            NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                Ok(Some(NP_List::new(found_cursor.clone(), &self.memory, false).into_iter().count()))
            },
            NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                Ok(Some(NP_Map::new(found_cursor.clone(), &self.memory).into_iter().count()))
            },
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                Ok(Some(columns.len()))
            },
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values } => {
                Ok(Some(values.len()))
            },
            NP_Parsed_Schema::Bytes { i: _, sortable: _, default: _, size} => {
                if *size > 0 {
                    Ok(Some(*size as usize))
                } else {
                    Ok(Some(self.memory.read_address(found_cursor.value.get_value_address())))
                }
            },
            NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
                if *size > 0 {
                    Ok(Some(*size as usize))
                } else {
                    Ok(Some(self.memory.read_address(found_cursor.value.get_value_address())))
                }
            },
            _ => {
                Ok(None)
            }
        }
  
    }

    /// Clear an inner value from the buffer.  The path only works with dot notation.
    /// This can also be used to clear deeply nested collection objects or scalar objects.
    /// 
    /// Returns `true` if it deleted a value, `false` otherwise.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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

        let value_cursor = self.select(self.cursor.clone(), false, path, 0)?;
        match value_cursor {
            Some(x) => {
                // clear value address in buffer
                self.memory.write_address(x.buff_addr, 0);
                if path.len() == 0 {
                    self.cursor.value = self.cursor.value.update_value_address(0);
                }

                Ok(true)
            }
            None => Ok(false)
        }
    }
  
    /// Retrieve an inner value from the buffer.  The path only works with dot notation.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///    "of": {"type": "map", "value": {
    ///         "type": "list", "of": {"type": "string"}
    ///     }}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    pub fn get<'get, X: 'get>(&'get self, path: &'get [&str]) -> Result<Option<X>, NP_Error> where X: NP_Value<'get> + NP_Scalar {
        let value_cursor = self.select(self.cursor.clone(), false, path, 0)?;

        match value_cursor {
            Some(x) => {
                match X::into_value(x, &self.memory)? {
                    Some(x) => {
                        Ok(Some(x))
                    },
                    None => {
                        match X::schema_default(&self.memory.schema[x.schema_addr]) {
                            Some(y) => {
                                Ok(Some(y))
                            },
                            None => {
                                Ok(None)
                            }
                        }
                    }
                }
            }
            None => Ok(None)
        }
    }

    /// This performs a compaction if the closure provided as the third argument returns `true`.
    /// Compaction is a pretty expensive operation (requires full copy of the whole buffer) so should be done sparingly.
    /// The closure is provided an argument that contains the original size of the buffer, how many bytes could be saved by compaction, and how large the new buffer would be after compaction.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// The second argument, new_size, can be used to change the size of the address space in the new buffer.  Default behavior is to copy the address size of the old buffer.  Be careful, if you're going from a larg address space down to a smaller one the data might not fit in the new buffer.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// // now using 25 bytes, with 7 bytes of wasted space
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 25,
    ///     after_compaction: 18,
    ///     wasted_bytes: 7
    /// }, new_buffer.calc_bytes()?);
    /// // compact to save space
    /// new_buffer.maybe_compact(None, None, |compact_data| {
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
    pub fn maybe_compact<F>(&mut self, new_capacity: Option<u32>, new_size: Option<NP_Size>, mut callback: F) -> Result<(), NP_Error> where F: FnMut(NP_Size_Data) -> bool {

        let bytes_data = self.calc_bytes()?;

        if callback(bytes_data) {
            self.compact(new_capacity, new_size)?;
        }

        return Ok(());
    }

    /// Compacts a buffer to remove an unused bytes or free space after a mutation.
    /// This is a pretty expensive operation (requires full copy of the whole buffer) so should be done sparingly.
    /// 
    /// The first argument, new_capacity, is the capacity of the underlying Vec<u8> that we'll be copying the data into.  The default is the size of the old buffer.
    /// 
    /// The second argument, new_size, can be used to change the size of the address space in the new buffer.  Default behavior is to copy the address size of the old buffer.  Be careful, if you're going from a larg address space down to a smaller one the data might not fit in the new buffer.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    /// new_buffer.compact(None, None)?;
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
    pub fn compact(&mut self, new_capacity: Option<u32>, new_size: Option<NP_Size>) -> Result<(), NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let size = match new_size {
            None => self.memory.size,
            Some(x) => { x }
        };

        let old_root = NP_Cursor::new(ROOT_PTR_ADDR, 0, &self.memory, NP_Cursor_Parent::None);

        let new_bytes = NP_Memory::new(Some(capacity), size, self.memory.schema);
        let new_root = NP_Cursor::new(ROOT_PTR_ADDR, 0, &new_bytes, NP_Cursor_Parent::None);

        self.cursor = NP_Cursor::compact(&old_root, &self.memory, new_root, &new_bytes)?;

        self.memory = new_bytes;

        Ok(())
    }

    /// Recursively measures how many bytes each element in the buffer is using.
    /// This will let you know how many bytes can be saved from a compaction.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None)?;
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
    pub fn calc_bytes(&self) -> Result<NP_Size_Data, NP_Error> {

        let root = NP_Cursor::new(ROOT_PTR_ADDR, 0, &self.memory, NP_Cursor_Parent::None);
        let real_bytes = NP_Cursor::calc_size(root, &self.memory)? + ROOT_PTR_ADDR;
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

    fn select(&self, cursor: NP_Cursor_Addr, create_path: bool, path: &[&str], mut path_index: usize) -> Result<Option<NP_Cursor_Addr>, NP_Error> {

        let mut loop_cursor = cursor;

        let addr_size = self.memory.addr_size_bytes();

        loop {
            if path.len() == path_index {
                return Ok(Some(loop_cursor));
            }

            let cursor = self.memory.get_cache(&loop_cursor);

            let mut value_addr = cursor.value.get_value_address();

            // no value in the buffer here
            if value_addr == 0 && create_path {
                match self.memory.schema[cursor.schema_addr] {
                    NP_Parsed_Schema::Table { columns, .. } => {

                        let mut value_addrs: [usize; 255] = [0usize; 255];
                        let mut value_offset = self.memory.read_bytes().len();
                        let table_addr = value_offset;

                        // create vtable
                        let new_vtable: Vec<u8> = Vec::with_capacity(columns.len() * addr_size + 4);
                        // length
                        new_vtable.push(columns.len() as u8);
                        value_offset += 1;
                        // addresses
                        for x in columns {
                            match self.memory.size {
                                NP_Size::U8  => new_vtable.push(0),
                                NP_Size::U16 => new_vtable.extend_from_slice(&[0u8; 2]),
                                NP_Size::U32 => new_vtable.extend_from_slice(&[0u8; 4]),
                            }
                            value_addrs[x.0 as usize] = value_offset;
                            self.memory.insert_cache(value_offset, NP_Cursor {
                                buff_addr: value_offset,
                                data: NP_Cursor_Data::new(&self.memory.schema[x.2]),
                                schema_addr: x.2,
                                value: NP_Cursor_Value::TableItem { value_addr: 0 },
                                parent_addr: table_addr,
                                prev_cursor: None
                            });
                            value_offset += addr_size;
                        }
                        // next vtable address
                        match self.memory.size {
                            NP_Size::U8  => new_vtable.push(0),
                            NP_Size::U16 => new_vtable.extend_from_slice(&[0u8; 2]),
                            NP_Size::U32 => new_vtable.extend_from_slice(&[0u8; 4]),
                        }
                        
                        // update cursor data
                        value_addr = self.memory.malloc(new_vtable)?;
                        cursor.value = cursor.value.update_value_address(value_addr);
                        self.memory.write_address(cursor.buff_addr, value_addr);

                        cursor.data = NP_Cursor_Data::Table {
                            values: value_addrs,
                            length: columns.len()
                        };
                    },
                    NP_Parsed_Schema::Tuple { values: schema_addrs, .. } => {
                        let mut value_addrs: [usize; 255] = [0usize; 255];
                        let mut value_offset = self.memory.read_bytes().len();
                        let tuple_addr = value_offset;

                        // create vtable
                        let new_vtable: Vec<u8> = Vec::with_capacity(schema_addrs.len() * addr_size + 4);
                        // length
                        new_vtable.push(schema_addrs.len() as u8);
                        value_offset += 1;
                        // addresses
                        for (i, x) in schema_addrs.iter().enumerate() {
                            match self.memory.size {
                                NP_Size::U8  => new_vtable.push(0),
                                NP_Size::U16 => new_vtable.extend_from_slice(&[0u8; 2]),
                                NP_Size::U32 => new_vtable.extend_from_slice(&[0u8; 4]),
                            }
                            value_addrs[i] = value_offset;
                            self.memory.insert_cache(value_offset, NP_Cursor {
                                buff_addr: value_offset,
                                data: NP_Cursor_Data::new(&self.memory.schema[*x]),
                                schema_addr: *x,
                                value: NP_Cursor_Value::TupleItem { value_addr: 0 },
                                parent_addr: tuple_addr,
                                prev_cursor: None
                            });
                            value_offset += addr_size;
                        }
                        // next vtable address
                        match self.memory.size {
                            NP_Size::U8  => new_vtable.push(0),
                            NP_Size::U16 => new_vtable.extend_from_slice(&[0u8; 2]),
                            NP_Size::U32 => new_vtable.extend_from_slice(&[0u8; 4]),
                        }
                        
                        // update cursor data
                        value_addr = self.memory.malloc(new_vtable)?;
                        cursor.value = cursor.value.update_value_address(value_addr);
                        self.memory.write_address(cursor.buff_addr, value_addr);

                        cursor.data = NP_Cursor_Data::Tuple {
                            values: value_addrs,
                            length: schema_addrs.len()
                        };
                    },
                    NP_Parsed_Schema::List  { of, .. } => {
                        
                        value_addr = match self.memory.size { // head + tail
                            NP_Size::U8  => self.memory.malloc_borrow(&[0u8; 2]),
                            NP_Size::U16 => self.memory.malloc_borrow(&[0u8; 4]),
                            NP_Size::U32 => self.memory.malloc_borrow(&[0u8; 8])
                        }?;
           
                        cursor.value = cursor.value.update_value_address(value_addr);
                        self.memory.write_address(cursor.buff_addr, value_addr);

                        cursor.data = NP_Cursor_Data::List {
                            head: 0,
                            tail: 0
                        };
                    },
                    NP_Parsed_Schema::Map   { value, .. } => {
                        value_addr = match self.memory.size { // head + length
                            NP_Size::U8  => self.memory.malloc_borrow(&[0u8; 2]), //  u8 |  u8
                            NP_Size::U16 => self.memory.malloc_borrow(&[0u8; 4]), // u16 | u16
                            NP_Size::U32 => self.memory.malloc_borrow(&[0u8; 6])  // u32 | u16
                        }?;
           
                        cursor.value = cursor.value.update_value_address(value_addr);
                        self.memory.write_address(cursor.buff_addr, value_addr);

                        cursor.data = NP_Cursor_Data::Map {
                            head: 0,
                            length: 0
                        };
                    },
                    _ => {
                        // scalar
                        panic!()
                    }
                }
            }

            // now select into collections

            match self.memory.schema[cursor.schema_addr] {
                NP_Parsed_Schema::Table { columns: column_schemas, .. } => {
                    let mut found = false;                         
                            
                    for col in column_schemas {
                        if &col.1 == path[path_index] {
                            loop_cursor = if value_addr == 0 { // no table here, step into virtual
                                let virtual_cursor = self.memory.get_cache(&NP_Cursor_Addr::Virtual);
                                virtual_cursor.parent_addr = 0;
                                virtual_cursor.value = NP_Cursor_Value::TableItem { value_addr: 0 };
                                virtual_cursor.schema_addr = col.2;
                                NP_Cursor_Addr::Virtual 
                            } else { //table exists
                                let next_addr = match cursor.data {
                                    NP_Cursor_Data::Table { values, .. } => { values[col.0 as usize] },
                                    _ => { unsafe { unreachable_unchecked() } }
                                };
                                NP_Cursor_Addr::Real(next_addr)
                            };
                            path_index += 1;
                            found = true;
                        }
                    }

                    // column with desired name doesn't exist
                    if found == false { 
                        return Ok(None);
                    }
                },
                NP_Parsed_Schema::Tuple { values: values_schemas, .. } => {
                    
                    let list_key = &path[path_index];
                    let list_key_int = list_key.parse::<usize>();
                    match list_key_int {
                        Ok(x) => {
                            if x >=  values_schemas.len() {
                                return Ok(None);
                            }
                            loop_cursor = if value_addr == 0 { // no tuple here, step into virtual
                                let virtual_cursor = self.memory.get_cache(&NP_Cursor_Addr::Virtual);
                                virtual_cursor.parent_addr = 0;
                                virtual_cursor.value = NP_Cursor_Value::TupleItem { value_addr: 0 };
                                virtual_cursor.schema_addr = values_schemas[x];
                                NP_Cursor_Addr::Virtual 
                            } else { // tuple exists
                                let next_addr = match cursor.data {
                                    NP_Cursor_Data::Tuple { values, .. } => { values[x] },
                                    _ => { unsafe { unreachable_unchecked() } }
                                };
                                NP_Cursor_Addr::Real(next_addr)
                            };
                            path_index += 1;
                        },
                        Err(_e) => {
                            let mut err = String::from("Can't query tuple with string, need number! Path: \n");
                            err.push_str(print_path(&path, path_index).as_str());
                            return Err(NP_Error::new(err))
                        }
                    }
                },
                NP_Parsed_Schema::List { of, .. } => {
                    let list_key = &path[path_index];
                    let list_key_int = list_key.parse::<usize>();
                    match list_key_int {
                        Ok(x) => {

                            let (head, tail ) = &match cursor.data {
                                NP_Cursor_Data::List { head , tail} => (head, tail),
                                _ => unsafe { unreachable_unchecked() }
                            };

                            

                            /*
                            loop_cursor = if value_addr == 0 { // virtual list, step into virtual item
                                let virtual_cursor = self.memory.get_cache(&NP_Cursor_Addr::Virtual);
                                virtual_cursor.parent_addr = 0;
                                virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, next: 0, index: x };
                                virtual_cursor.schema_addr = x;
                                NP_Cursor_Addr::Virtual 
                            } else {
                                if *head == 0 { // empty list
                                    if create_path { // need to make initial item
                                        let new_item = match self.memory.size { // value_ptr + next + index
                                            NP_Size::U8  => self.memory.malloc_borrow(&[0u8; 3]), //  u8 |  u8 | u8
                                            NP_Size::U16 => self.memory.malloc_borrow(&[0u8; 6]), // u16 | u16 | u16
                                            NP_Size::U32 => self.memory.malloc_borrow(&[0u8; 10])  // u32 | u32 | u16
                                        }?;
    
                                        // update index
                                        match self.memory.size {
                                            NP_Size::U8 => {
                                                self.memory.write_bytes()[new_item + 2] = x as u8;
                                            },
                                            _ => {
                                                for (i, b) in (x as u16).to_be_bytes().iter().enumerate() {
                                                    self.memory.write_bytes()[new_item + addr_size + addr_size + i] = *b;
                                                }
                                            }
                                        }
                                        self.memory.insert_cache(new_item, NP_Cursor {
                                            buff_addr: new_item,
                                            data: NP_Cursor_Data::new(&self.memory.schema[of]),
                                            schema_addr: of,
                                            value: NP_Cursor_Value::ListItem { value_addr: 0, next: 0, index: x },
                                            parent_addr: value_addr,
                                            prev_cursor: None
                                        });
    
                                        // update head & tail
                                        cursor.data = NP_Cursor_Data::List { head: new_item, tail: new_item };
                                        self.memory.write_address(value_addr, new_item);
                                        self.memory.write_address(value_addr + addr_size, new_item);
    
                                        NP_Cursor_Addr::Real(new_item)
                                    } else { // can return virtual
                                        let virtual_cursor = self.memory.get_cache(&NP_Cursor_Addr::Virtual);
                                        virtual_cursor.parent_addr = 0;
                                        virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, next: 0, index: x };
                                        virtual_cursor.schema_addr = x;
                                        NP_Cursor_Addr::Virtual 
                                    }
    
                                } else { // list has items

                                    let mut prev_addr = 0usize;
                                    let mut step_addr = *head;

                                    let mut found: Option<NP_Cursor_Addr> = None;
                                    
                                    while found == Option::None {
                                        let next_cursor = self.memory.get_cache(&NP_Cursor_Addr::Real(step_addr));
                                        let (cursor_index, next_next_addr) = match next_cursor.value {
                                            NP_Cursor_Value::ListItem { index, next, .. } => (index, next),
                                            _ => unsafe { unreachable_unchecked() }
                                        };

                                        if cursor_index == x { // found matching index, return this cursor
                                            found = Some(NP_Cursor_Addr::Real(next_cursor.buff_addr));

                                        } else if cursor_index > x { // we've passed the desired index without a matching cursor

                                            if create_path { // make the new item right now

                                                let new_item = match self.memory.size { // value_ptr + next + index
                                                    NP_Size::U8  => self.memory.malloc_borrow(&[0u8; 3]), //  u8 |  u8 | u8
                                                    NP_Size::U16 => self.memory.malloc_borrow(&[0u8; 6]), // u16 | u16 | u16
                                                    NP_Size::U32 => self.memory.malloc_borrow(&[0u8; 10])  // u32 | u32 | u16
                                                }?;
            
                                                // update index
                                                match self.memory.size {
                                                    NP_Size::U8 => {
                                                        self.memory.write_bytes()[new_item + 2] = x as u8;
                                                    },
                                                    _ => {
                                                        for (i, b) in (x as u16).to_be_bytes().iter().enumerate() {
                                                            self.memory.write_bytes()[new_item + addr_size + addr_size + i] = *b;
                                                        }
                                                    }
                                                }
                                                self.memory.insert_cache(new_item, NP_Cursor {
                                                    buff_addr: new_item,
                                                    data: NP_Cursor_Data::new(&self.memory.schema[of]),
                                                    schema_addr: of,
                                                    value: NP_Cursor_Value::ListItem { value_addr: 0, next: 0, index: x },
                                                    parent_addr: value_addr,
                                                    prev_cursor: None
                                                });

                                            } else { // return virtual
                                                let virtual_cursor = self.memory.get_cache(&NP_Cursor_Addr::Virtual);
                                                virtual_cursor.parent_addr = value_addr;
                                                virtual_cursor.value = NP_Cursor_Value::ListItem { value_addr: 0, next: 0, index: x };
                                                virtual_cursor.schema_addr = of;
                                                virtual_cursor.prev_cursor = Some(prev_addr);
                                                found = Some(NP_Cursor_Addr::Virtual)
                                            }

                                        } else { // haven't reached the desired index
                                            prev_addr = step_addr;
                                            step_addr = next_next_addr;
                                        }
                                    }


                                    found.unwrap()
                                }
                            };*/
                            
                            path_index += 1;
                        },
                        Err(_e) => {
                            let mut err = String::from("Can't query list with string, need number! Path: \n");
                            err.push_str(print_path(&path, path_index).as_str());
                            return Err(NP_Error::new(err))
                        }
                    }
                },
                NP_Parsed_Schema::Map { value, .. } => {

                },
                _ => { // we've reached a scalar value but not at the end of the path
                    return Ok(None);
                }
            }
        }
    }
}



/// NP Item
#[derive(Debug)]
pub struct NP_Item<'item> {
    /// index of this value
    pub index: usize,
    /// Key at this index
    pub key: String,
    memory: &'item NP_Memory<'item>,
    cursor: NP_Cursor
}

impl<'item> NP_Item<'item> {

    /// get key
    pub fn get_key<'key>(&'key self) -> &'key str {
        &self.key
    }

    /// If this item has a value
    pub fn has_value(&self) -> bool {
        self.cursor.value.get_value_address() != 0
    }

    /// Get value at this pointer
    pub fn get<X>(&'item self) -> Result<Option<X>, NP_Error> where X: NP_Value<'item> + NP_Scalar {

        match X::into_value(self.cursor, &self.memory)? {
            Some(x) => {
                Ok(Some(x))
            },
            None => {
                match X::schema_default(&self.memory.schema[self.cursor.schema_addr]) {
                    Some(y) => {
                        Ok(Some(y))
                    },
                    None => {
                        Ok(None)
                    }
                }
            }
        }
        
    }
    /// Set value at this pointer
    pub fn set<X>(&'item mut self, value: X) -> Result<(), NP_Error> where X: NP_Value<'item> + NP_Scalar {

        // virtual cursor
        if self.cursor.buff_addr == 0 {
            match self.cursor.value {
                NP_Cursor_Value::ListItem { .. } => {
                    self.cursor = NP_List::commit_virtual_cursor(self.cursor, &self.memory)?;
                },
                NP_Cursor_Value::TableItem { .. } => {
                    self.cursor = NP_Table::commit_virtual_cursor(self.cursor, &self.memory)?;
                },
                NP_Cursor_Value::TupleItem { .. } => { panic!() } ,
                NP_Cursor_Value::MapItem { .. } => { panic!() } ,
                _ => panic!()
            }
        }

        if self.cursor.buff_addr == 0 { // can't commit to cursor that's still virtual
            panic!()
        }

        self.cursor = X::set_value(self.cursor, &self.memory, value)?;

        Ok(())
    }
    /// Clear the value at this pointer
    pub fn del(&'item mut self) -> bool {
        if self.cursor.value.get_value_address() == 0 {
            false
        } else {
            self.memory.write_address(self.cursor.buff_addr, 0);
            true
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
    List(NP_List<'col>),
    /// Table
    Table(NP_Table<'col>),
    /// Tuple
    Tuple(NP_Tuple<'col>)
}


/// Generic iterator
#[derive(Debug)]
#[doc(hidden)]
pub struct NP_Generic_Iterator<'coll> { 
    /// The colleciton iterator
    pub iterator: NP_Iterator_Collection<'coll>,
    /// what index we're on
    pub index: usize
}

impl<'collection> Iterator for NP_Generic_Iterator<'collection> {
    type Item = NP_Item<'collection>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.iterator {
            NP_Iterator_Collection::Map(x) => {
                if let Some(p) = x.next() {
                    let item = NP_Item {
                        index: self.index,
                        key: String::from(p.0),
                        cursor: p.1,
                        memory: x.memory
                    };
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::List(x) => {
                if let Some(p) = x.next() {

                    let item = NP_Item {
                        key: p.0.to_string(),
                        index: p.0,
                        cursor: p.1,
                        memory: x.memory
                    };

                    self.index = p.0;
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Table(x) => {
                if let Some(p) = x.next() {
                    let item = NP_Item {
                        index: p.0,
                        cursor: p.2,
                        key: String::from(p.1),
                        memory: x.memory
                    };
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Tuple(x) => {
                if let Some(p) = x.next() {
                    let item = NP_Item {
                        index: p.0,
                        cursor: p.1,
                        memory: x.memory,
                        key: p.0.to_string(),
                    };
                    self.index = p.0;
                    Some(item)
                } else {
                    None
                }
            },
            _ => panic!()
        }
    }
}