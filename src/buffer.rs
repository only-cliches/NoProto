//! Top level abstraction for buffer objects

use alloc::rc::Rc;
use crate::collection::map::NP_Map;
use crate::{schema::NP_TypeKeys, pointer::NP_Value};
use crate::pointer::NP_Cursor_Addr;
use alloc::string::String;
use crate::{pointer::NP_Cursor, collection::{NP_Collection, tuple::NP_Tuple}};
use crate::{schema::NP_Parsed_Schema, collection::table::NP_Table};
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::{collection::{list::NP_List}};
use crate::error::NP_Error;
use crate::memory::{NP_Size, NP_Memory};
use crate::{schema::{NP_Schema}, json_flex::NP_JSON};
use crate::alloc::borrow::ToOwned;

/// The address location of the root pointer.
#[doc(hidden)]
pub const ROOT_PTR_ADDR: usize = 2;

pub type MEMORY<'mem> = Rc<NP_Memory<'mem>>;

/// Buffers contain the bytes of each object and allow you to perform reads, updates, deletes and compaction.
/// 
/// 
#[derive(Debug)]
pub struct NP_Buffer<'buffer> {
    /// Schema data used by this buffer
    memory: NP_Memory<'buffer>,
    cursor: NP_Cursor_Addr
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
    pub fn _new(memory: NP_Memory<'buffer>) -> Self { // make new buffer

        let mut root_cursor = NP_Cursor::new(ROOT_PTR_ADDR, 0, 0, &memory);

        root_cursor.address_value = memory.read_address(ROOT_PTR_ADDR);

        memory.insert_cache(root_cursor);

        NP_Buffer {
            cursor: NP_Cursor_Addr { address: ROOT_PTR_ADDR, is_virtual: false},
            memory: memory
        }
    }


    /// Copy an object at the provided path and all it's children into JSON.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::here;
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "uint8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set(path("name"), String::from("Jeb Kermin"));
    /// new_buffer.set(path("age"), 30u8);
    /// 
    /// assert_eq!("{\"age\":30,\"name\":\"Jeb Kermin\"}", new_buffer.json_encode(here())?.stringify());
    /// assert_eq!("\"Jeb Kermin\"", new_buffer.json_encode(path("name"))?.stringify());
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn json_encode<'json>(&'json self, path: &'json [&str]) -> Result<NP_JSON, NP_Error> {

        let value_cursor = NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)?;

        if let Some(x) = value_cursor {
            Ok(NP_Cursor::json_encode(x, self.memory.clone()))
        } else {
            Ok(NP_JSON::Null)
        }

        
    }

    /// Moves the underlying bytes out of the buffer, consuming the buffer in the process.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::here;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set(here(), String::from("hello"))?;
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

    /// Move buffer cursor to new location
    pub fn move_cursor(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        let cursor = NP_Cursor::select_with_commit(self.cursor.clone(), self.memory.clone(), path, 0)?;

        let cursor = if let Some(x) = cursor {
            x
        } else {
            return Ok(false);
        };

        self.cursor = cursor.clone();

        Ok(true)
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // third item in the top level list -> key "alpha" of map at 3rd element -> 9th element of list at "alpha" key
    /// // 
    /// new_buffer.set(&["3", "alpha", "9"], String::from("look at all this nesting madness"))?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<String>(&["3", "alpha", "9"])?;
    /// 
    /// assert_eq!(message, Some(Box::new(String::from("look at all this nesting madness"))));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn set<X>(&mut self, path: &[&str], value: X) -> Result<bool, NP_Error> where X: NP_Value<'buffer> + Default {

        let value_cursor = NP_Cursor::select_with_commit(self.cursor.clone(), self.memory.clone(), path, 0)?;
        let value_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(false);
        };
        self.memory.set_value(&value_cursor, value)?;
        Ok(true)
    }

    /// Get an iterator for a collection
    /// 
    /// 
    /// ## List Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::here;
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value at 1 index
    /// new_buffer.set(path("1"), String::from("hello"))?;
    /// // set value at 4 index
    /// new_buffer.set(path("4"), String::from("world"))?;
    /// // push value onto the end
    /// new_buffer.list_push(here(), String::from("!"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(here())?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         1 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("hello"))),
    ///         2 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         3 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         4 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("world"))),
    ///         5 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("!"))),
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
    /// use no_proto::here;
    /// use no_proto::path;
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value of age
    /// new_buffer.set(path("age"), 20u8)?;
    /// // set value of name
    /// new_buffer.set(path("name"), String::from("Bill Kerman"))?;
    /// // push value onto tags list
    /// new_buffer.list_push(path("tags"), String::from("rocket"))?;
    /// 
    /// // get iterator of root (table)
    /// new_buffer.get_iter(here())?.unwrap().into_iter().for_each(|item| {
    ///     let column = item.get_key();
    ///     if column == String::from("name") {
    ///         assert_eq!(item.get::<String>().unwrap(), Some(String::from("Bill Kerman")))
    ///     } else if column == String::from("age") {
    ///         assert_eq!(item.get::<u8>().unwrap(), Some(20))
    ///     } else if column == String::from("job") {
    ///         assert_eq!(item.get::<String>().unwrap(), None)
    ///     } else if column == String::from("tags") {
    ///         // tags column is list, can't do anything with it here
    ///     } else {
    ///         panic!()
    ///     }
    /// });
    /// 
    /// // we can also loop through items of the tags list
    /// new_buffer.get_iter(path("tags"))?.unwrap().into_iter().for_each(|item| {
    ///     assert_eq!(item.index, 0);
    ///     assert_eq!(item.get::<String>().unwrap(), Some(String::from("rocket")));
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
    /// use no_proto::here;
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value of color key
    /// new_buffer.set(path("color"), String::from("blue"))?;
    /// // set value of sport key
    /// new_buffer.set(path("sport"), String::from("soccor"))?;
    /// 
    /// // get iterator of root (map)
    /// new_buffer.get_iter(here())?.unwrap().into_iter().for_each(|item| {
    ///     let column = item.get_key();
    ///     if column == String::from("color") {
    ///         assert_eq!(item.get::<String>().unwrap(), Some(String::from("blue")))
    ///     } else if column == String::from("sport") {
    ///         assert_eq!(item.get::<String>().unwrap(), Some(String::from("soccor")))
    ///     } else {
    ///         panic!()
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
    /// use no_proto::here;
    /// use no_proto::path;
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value at 0 index
    /// new_buffer.set(path("0"), String::from("hello"))?;
    /// // set value at 2 index
    /// new_buffer.set(path("2"), false)?;
    /// 
    /// // get iterator of root (tuple item)
    /// new_buffer.get_iter(&[])?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("hello"))),
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

        let value = NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)?;

        let value = if let Some(x) = value {
            x
        } else {
            return Ok(None);
        };

        // value doesn't exist
        if value.is_virtual {
            return Ok(None);
        }

        let value_schema= value.get_schema_data_owned(self.memory.clone());

        match value_schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Table(NP_Table::start_iter(value, self.memory.clone())?)
                }))
            },
            NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Map(NP_Map::start_iter(value, self.memory.clone())?)
                }))
            },
            NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::List(NP_List::start_iter(value, self.memory.clone())?)
                }))
            },
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values: _} => {
                Ok(Some(NP_Generic_Iterator {
                    index: 0,
                    iterator: NP_Iterator_Collection::Tuple(NP_Tuple::start_iter(value, self.memory.clone())?)
                }))
            },
            _ => {
                Err(NP_Error::new("Attempted to ierate on non collection!"))
            }
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
    /// use no_proto::here;
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set(path("3"), String::from("launch"))?;
    /// new_buffer.list_push(here(), String::from("this"))?;
    /// new_buffer.list_push(here(), String::from("rocket"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(here())?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         1 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         2 => assert_eq!(item.get::<String>().unwrap(), None),
    ///         3 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("launch"))),
    ///         4 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("this"))),
    ///         5 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("rocket"))),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.list_push(here(), String::from("launch"))?;
    /// new_buffer.list_push(here(), String::from("this"))?;
    /// new_buffer.list_push(here(), String::from("rocket"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter(here())?.unwrap().into_iter().for_each(|item| {
    ///     match item.index {
    ///         0 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("launch"))),
    ///         1 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("this"))),
    ///         2 => assert_eq!(item.get::<String>().unwrap(), Some(String::from("rocket"))),
    ///         _ => panic!()
    ///     };
    /// });
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn list_push<'push, X: 'push>(&'push mut self, path: &'push [&str], value: X) -> Result<Option<u16>, NP_Error> where X: NP_Value<'push> + Default {

        let list_cursor = NP_Cursor::select_with_commit(self.cursor.clone(), self.memory.clone(), path, 0)?;

        let list_cursor = if let Some(x) = list_cursor {
            x
        } else {
            return Ok(None);
        };

        match list_cursor.get_schema_data(&self.memory) {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {

                let of_schema = self.memory.schema.get(*of).unwrap();

                // type does not match schema
                if X::type_idx().1 != of_schema.get_type_key() {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().0);
                    err.push_str(") into schema of type (");
                    err.push_str(of_schema.get_type_data().0);
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                let new_cursor = NP_List::push(list_cursor, &self.memory, None)?;

                NP_List::commit_pointer(&new_cursor, &self.memory)?;

                X::set_value(new_cursor, &self.memory, Box::new(&value))?;

                let new_index = self.memory.get_cursor_data(&new_cursor).unwrap().item_index.unwrap();

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
    /// use no_proto::schema::NP_Parsed_Schema;
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // get schema of root
    /// let type_key = new_buffer.get_schema(here())?.unwrap();
    /// 
    /// let is_string = match type_key {
    ///     NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size: _} => {
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
    pub fn get_type(&mut self, path: &[&str]) -> Result<Option<NP_TypeKeys>, NP_Error> {
        let value_cursor = NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None)
        };

        Ok(Some(found_cursor.get_schema_data_owned(self.memory.clone()).get_type_key()))
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
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set(here(), String::from("hello"))?;
    /// // get length of value at root (String)
    /// assert_eq!(new_buffer.length(here())?, Some(5));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (List) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::path;
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value at 9th index
    /// new_buffer.set(path("9"), String::from("hello"))?;
    /// // get length of value at root (List)
    /// assert_eq!(new_buffer.length(here())?, Some(10));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Table) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "table",
    ///    "columns": [
    ///         ["age", {"type": "u8"}],
    ///         ["name", {"type": "string"}]
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // get length of value at root (Table)
    /// assert_eq!(new_buffer.length(here())?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Map) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::here;
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set values
    /// new_buffer.set(path("foo"), String::from("bar"))?;
    /// new_buffer.set(path("foo2"), String::from("bar2"))?;
    /// // get length of value at root (Map)
    /// assert_eq!(new_buffer.length(here())?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    /// ## Collection (Tuple) Example
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "tuple",
    ///    "values": [
    ///         {"type": "string"}, 
    ///         {"type": "string"}
    ///     ]
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // get length of value at root (Tuple)
    /// assert_eq!(new_buffer.length(here())?, Some(2));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn length(&mut self, path: &[&str]) -> Result<Option<usize>, NP_Error> {
        let value_cursor = NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)?;

        let found_cursor = if let Some(x) = value_cursor {
            x
        } else {
            return Ok(None)
        };

        if found_cursor.is_virtual {
            return Ok(None);
        }

        match &found_cursor.get_schema_data_owned(self.memory.clone()) {
            NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                Ok(Some(NP_List::start_iter(found_cursor.clone(), self.memory.clone()).into_iter().count()))
            },
            NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                Ok(Some(NP_Map::start_iter(found_cursor.clone(), self.memory.clone()).into_iter().count()))
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
                    let cursor = found_cursor.get_data_owned(self.memory.clone())?;
                    Ok(Some(self.memory.read_address(cursor.address_value)))
                }
            },
            NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
                if *size > 0 {
                    Ok(Some(*size as usize))
                } else {
                    let cursor = found_cursor.get_data_owned(self.memory.clone())?;
                    Ok(Some(self.memory.read_address(cursor.address_value)))
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
    /// use no_proto::path;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///     "of": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set index 0
    /// new_buffer.set(path("0"), String::from("hello"))?;
    /// // del index 0
    /// new_buffer.del(path("0"))?;
    /// // value is gone now!
    /// assert_eq!(None, new_buffer.get::<String>(path("0"))?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn del(&mut self, path: &[&str]) -> Result<bool, NP_Error> {

        Ok(match NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)? {
            Some(x) => NP_Cursor::clear_here(x, self.memory.clone()),
            None => false
        })
    }
  
    /// Retrieve an inner value from the buffer.  The path only works with dot notation.
    /// 
    /// The type that you cast the request to will be compared to the schema, if it doesn't match the schema the request will fail.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::buffer::NP_Size_Data;
    /// use no_proto::path;
    /// 
    /// // a list where each item is a map where each key has a value containing a list of strings
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "list",
    ///    "of": {"type": "map", "value": {
    ///         "type": "list", "of": {"type": "string"}
    ///     }}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // third item in the top level list -> key "alpha" of map at 3rd element -> 9th element of list at "alpha" key
    /// // 
    /// new_buffer.set(path("3.alpha.9"), String::from("who would build a schema like this"))?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<String>(path("3.alpha.9"))?;
    /// 
    /// assert_eq!(message, Some(Box::new(String::from("who would build a schema like this"))));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get<'get, X: 'get>(&'get mut self, path: &[&str]) -> Result<Option<&'get X>, NP_Error> where X: NP_Value<'get> + Default {
        let value_cursor = NP_Cursor::select(self.cursor.clone(), self.memory.clone(), path, 0)?;
        match value_cursor {
            Some(x) => {
                X::into_value(x, self.memory.clone())
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
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set(here(), String::from("hello"))?;
    /// // using 11 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set(here(), String::from("hello, world"))?;
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
    /// use no_proto::here;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set(here(), String::from("hello"))?;
    /// // using 11 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set(here(), String::from("hello, world"))?;
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

        let root_cursor = NP_Cursor_Addr {address: ROOT_PTR_ADDR, is_virtual: false};

        let new_bytes = NP_Memory::new(Some(capacity), size, self.memory.schema);
        let mut new_root_cursor = NP_Cursor::new(ROOT_PTR_ADDR, 0, 0, &new_bytes);
        new_bytes.insert_cache(new_root_cursor);

        self.cursor = NP_Cursor::compact(root_cursor.clone(), self.memory.clone(), root_cursor.clone(), &new_bytes)?;

        self.memory = new_bytes;

        Ok(())
    }

    /// Recursively measures how many bytes each element in the buffer is using.
    /// This will let you know how many bytes can be saved from a compaction.
    /// 
    /// ```
    /// use no_proto::error::NP_Error;
    /// use no_proto::NP_Factory;
    /// use no_proto::here;
    /// use no_proto::buffer::NP_Size_Data;
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set(here(), String::from("hello"))?;
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

        let real_bytes = NP_Cursor::calc_size(NP_Cursor_Addr {address: ROOT_PTR_ADDR, is_virtual: false}, self.memory.clone())? + ROOT_PTR_ADDR;
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
#[derive(Debug)]
pub struct NP_Item<'item> {
    /// index of this value
    pub index: usize,
    memory: &'item NP_Memory<'item>,
    cursor: NP_Cursor_Addr
}

impl<'item> NP_Item<'item> {
    /// Clone this item
    pub fn clone(&self) -> Self {
        NP_Item {
            index: self.index,
            ptr: self.ptr.clone(),
        }
    }

    /// Get key at this value
    pub fn get_key<'key>(&'key self) -> String {
        match &self.ptr.helper {
            NP_Iterator_Helper::List { index, prev_addr: _, next_index: _, next_addr: _ } => {
                index.to_string()
            },
            NP_Iterator_Helper::Table { index: _, column, prev_addr: _, skip_step: _} => {
                String::from(*column)
            },
            NP_Iterator_Helper::Map { key_addr , prev_addr: _, key} => {
                if let Some(x) = key {
                    return x.clone()
                }
                
                let memory = (&self.ptr.memory);
                let addr = key_addr;

                NP_Map::get_key(*addr, memory)
            },
            NP_Iterator_Helper::Tuple { index } => {
                index.to_string()
            },
            _ => String::from("")
        }
    }
    /// If this item has a value
    pub fn has_value(&self) -> bool {
        self.ptr.has_value()
    }
    /// Get value at this pointer
    pub fn get<X>(&'item self) -> Result<Option<X>, NP_Error> where X: NP_Value<'item> + Default {
        self.ptr.get_here::<X>()
    }
    /// Set value at this pointer
    pub fn set<X>(&'item mut self, value: X) -> Result<(), NP_Error> where X: NP_Value<'item> + Default {
        self.ptr.set_here(value)
    }
    /// Clear the value at this pointer
    pub fn clear(&'item mut self) -> bool {
        NP_Ptr::clear_here(&mut self.ptr)
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
                        ptr: p
                    };
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::List(x) => {
                if let Some(p) = x.next() {
                    let index = x.memory.get_cursor_data(&p).item_index.unwrap();

                    let item = NP_Item {
                        index: index,
                        cursor: p.clone(),
                        memory: x.memory
                    };

                    self.index = index;
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Table(x) => {
                if let Some(p) = x.next() {
                    let index = match p.helper {
                        NP_Iterator_Helper::Table { index, column: _, prev_addr: _, skip_step: _ } => {
                            index
                        },
                        _ => panic!()
                    };
                    let item = NP_Item {
                        index: index as usize,
                        ptr: p
                    };
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::Tuple(x) => {
                if let Some(mut p) = x.next() {
                    let item = NP_Item {
                        index: self.index,
                        ptr: p.select().unwrap()
                    };
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            }
        }
    }
}