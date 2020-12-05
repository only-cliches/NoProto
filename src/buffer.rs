//! Top level abstraction for buffer objects

use core::hint::unreachable_unchecked;

use crate::{hashmap::{SEED, murmurhash3_x86_32}, pointer::{NP_Cursor_Addr, NP_Cursor_Data, NP_Scalar}};
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
pub const ROOT_PTR_ADDR: usize = 0;
/// Maximum size of list collections
#[doc(hidden)]
pub const LIST_MAX_SIZE: usize = core::u16::MAX as usize;

/// Buffers contain the bytes of each object and allow you to perform reads, updates, deletes and compaction.
/// 
/// 
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

        // Parse buffer
        NP_Cursor::parse(ROOT_PTR_ADDR, 0, 0, 0, &memory)?;

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
    pub fn json_encode(&self, path: &[&str]) -> Result<NP_JSON, NP_Error> {

        let value_cursor = self.select(&self.cursor_addr, false, path, 0)?;

        if let Some(x) = value_cursor {
            Ok(NP_Cursor::json_encode(x, &self.memory))
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

        let value_cursor = self.select(&self.cursor_addr, true, path, 0)?;

        let cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(false);
        };

        self.cursor_addr = cursor;

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

        let value_cursor = self.select(&self.cursor_addr, true, path, 0)?;
        match value_cursor {
            Some(x) => {
                X::set_value(x, &self.memory, value)?;
                Ok(true)
            }
            None => Ok(false)
        }
    }

    /*
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
    */

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
        let value_cursor = self.select(&self.cursor_addr, false, path, 0)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None)
        };

        Ok(Some(self.memory.get_schema(&found_cursor).get_type_key()))
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

        let value_cursor = self.select(&self.cursor_addr, false, path, 0)?;
        match value_cursor {
            Some(x) => {
                // clear value address in buffer
                self.memory.get_parsed(&x).value.set_addr_value(0);
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
    pub fn get<'get, X: 'get>(&'get self, path: &'get [&str]) -> Result<Option<X>, NP_Error> where X: NP_Value<'buffer> + NP_Scalar {
        let value_cursor = self.select(&self.cursor_addr, false, path, 0)?;

        match value_cursor {
            Some(x) => {
                match X::into_value(x, &self.memory)? {
                    Some(x) => {
                        Ok(Some(x))
                    },
                    None => {
                        /*
                        match X::schema_default(&self.memory.get_schema(&x)) {
                            Some(y) => {
                                Ok(Some(y))
                            },
                            None => {
                                Ok(None)
                            }
                        }*/
                        Ok(None)
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
    pub fn compact(&mut self, new_capacity: Option<u32>) -> Result<(), NP_Error> {

        let capacity = match new_capacity {
            Some(x) => { x as usize },
            None => self.memory.read_bytes().len()
        };

        let old_root = NP_Cursor_Addr::Real(ROOT_PTR_ADDR);

        let new_bytes = NP_Memory::new(Some(capacity), self.memory.schema);
        let new_root  = NP_Cursor_Addr::Real(ROOT_PTR_ADDR);

        NP_Cursor::compact(old_root, &self.memory, new_root, &new_bytes)?;

        self.cursor_addr = NP_Cursor_Addr::Real(ROOT_PTR_ADDR);

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

        let root = NP_Cursor_Addr::Real(ROOT_PTR_ADDR);
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

    fn select(&self, cursor: &NP_Cursor_Addr, create_path: bool, path: &[&str], mut path_index: usize) -> Result<Option<NP_Cursor_Addr>, NP_Error> {

        let mut loop_cursor = cursor.clone();


        loop {
            if path.len() == path_index {
                return Ok(Some(loop_cursor));
            }

            let cursor = self.memory.get_parsed(&loop_cursor);

            // now select into collections

            match self.memory.schema[cursor.schema_addr] {
                NP_Parsed_Schema::Table { columns: column_schemas, columns_mapped,  .. } => {

                    let column_index = columns_mapped.get(path[path_index]);

                    match column_index {
                        // column exists
                        Some(index) => {
                            if let NP_Cursor_Addr::Real(col_index) = loop_cursor { // real loop

                                let v_table =  col_index / 4;
                                let v_table_idx = col_index % 4;

                                match &cursor.data {
                                    NP_Cursor_Data::Table { bytes  } => {
                                        let mut sel_v_table = &bytes[v_table];

                                        if sel_v_table.0 == 0 && create_path { // no vtable here, need to make one
                                            sel_v_table = &NP_Table::extend_vtables(&loop_cursor, &self.memory, col_index)?[v_table];
                                        }

                                        // virtual value
                                        if sel_v_table.0 == 0 {
                                            let virtual_cursor = self.memory.get_parsed(&loop_cursor);
                                            virtual_cursor.reset();
                                            virtual_cursor.parent_addr = cursor.buff_addr;
                                            virtual_cursor.schema_addr = column_schemas[*index].2;
                                            path_index += 1;
                                        } else { // real
                                            let item_addr = sel_v_table.0 + (v_table_idx * 2);
                                            loop_cursor = NP_Cursor_Addr::Real(item_addr);
                                            path_index += 1;                                            
                                        }
                                    },
                                    _ => unsafe { unreachable_unchecked() }
                                };

                            } else { // virtual, just for schemas
                                let virtual_cursor = self.memory.get_parsed(&loop_cursor);
                                virtual_cursor.reset();
                                virtual_cursor.parent_addr = cursor.buff_addr;
                                virtual_cursor.schema_addr = column_schemas[*index].2;
                                path_index += 1;
                            }
                        },
                        // column doesn't exist
                        None => {
                            return Ok(None)
                        }
                    }
                },
                NP_Parsed_Schema::Tuple { values: values_schemas, .. } => {
                    let list_key = &path[path_index];
                    let list_key_int = list_key.parse::<usize>();
                    match list_key_int {
                        Ok(tuple_index) => {
                            if tuple_index > values_schemas.len() {
                                return Ok(None);
                            }

                            let v_table =  tuple_index / 4;
                            let v_table_idx = tuple_index % 4;

                            match &cursor.data {
                                NP_Cursor_Data::Tuple { bytes  } => {
                                    let mut sel_v_table = &bytes[v_table];

                                    if sel_v_table.0 == 0 && create_path { // no vtable here, need to make one
                                        sel_v_table = &NP_Table::extend_vtables(&loop_cursor, &self.memory, tuple_index)?[v_table];
                                    }

                                    // virtual value
                                    if sel_v_table.0 == 0 {
                                        let virtual_cursor = self.memory.get_parsed(&loop_cursor);
                                        virtual_cursor.reset();
                                        virtual_cursor.parent_addr = cursor.buff_addr;
                                        virtual_cursor.schema_addr = values_schemas[tuple_index];
                                        path_index += 1;
                                    } else { // real
                                        let item_addr = sel_v_table.0 + (v_table_idx * 2);
                                        loop_cursor = NP_Cursor_Addr::Real(item_addr);
                                        path_index += 1;                                            
                                    }
                                },
                                _ => unsafe { unreachable_unchecked() }
                            };

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
                        Ok(list_index) => {

                        },
                        Err(_e) => {
                            let mut err = String::from("Can't query tuple with string, need number! Path: \n");
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


/*
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
}*/