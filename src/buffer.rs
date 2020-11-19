//! Top level abstraction for buffer objects

use alloc::string::String;
use crate::collection::{NP_Collection, tuple::NP_Tuple};
use crate::{collection::tuple::NP_Tuple_Iterator, schema::NP_Parsed_Schema, collection::table::NP_Table};
use crate::collection::table::NP_Table_Iterator;
use crate::collection::list::NP_List_Iterator;
use alloc::boxed::Box;
use alloc::vec::Vec;
use crate::{collection::{list::NP_List, map::{NP_Map, NP_Map_Iterator}}, pointer::{NP_Iterator_Helper, NP_Value}};
use crate::error::NP_Error;
use crate::pointer::{NP_Ptr};
use crate::memory::{NP_Size, NP_Memory};
use crate::{schema::{NP_Schema}, json_flex::NP_JSON};
use crate::alloc::string::ToString;
use crate::alloc::borrow::ToOwned;

/// The address location of the root pointer.
pub const ROOT_PTR_ADDR: usize = 2;

/// Buffers contain the bytes of each object and allow you to perform reads, updates, deletes and compaction.
/// 
/// 
#[derive(Debug)]
pub struct NP_Buffer<'buffer> {
    /// Schema data used by this buffer
    pub schema: &'buffer NP_Schema,
    memory: NP_Memory
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
    pub fn _new(schema: &'buffer NP_Schema, memory: NP_Memory) -> Self { // make new buffer

        NP_Buffer {
            schema: schema,
            memory: memory
        }
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set("name", String::from("Jeb Kermin"));
    /// new_buffer.set("age", 30u8);
    /// 
    /// assert_eq!("{\"age\":30,\"name\":\"Jeb Kermin\"}", new_buffer.json_encode("")?.stringify());
    /// assert_eq!("\"Jeb Kermin\"", new_buffer.json_encode("name")?.stringify());
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn json_encode(&self, path: &str) -> Result<NP_JSON, NP_Error> {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        let json_ptr = root._deep_get(vec_path, 0)?;

        if let Some(x) = json_ptr {
            Ok(x.json_encode())
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set("", String::from("hello"))?;
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
    /// new_buffer.set("3.alpha.9", String::from("look at all this nesting madness"))?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<String>("3.alpha.9")?;
    /// 
    /// assert_eq!(message, Some(Box::new(String::from("look at all this nesting madness"))));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn set<'set, X: 'set>(&'set mut self, path: &str, value: X) -> Result<(), NP_Error> where X: NP_Value<'set> + Default {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        root._deep_set_value(vec_path, 0, value)
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value at 1 index
    /// new_buffer.set("1", String::from("hello"))?;
    /// // set value at 4 index
    /// new_buffer.set("4", String::from("world"))?;
    /// // push value onto the end
    /// new_buffer.list_push("", String::from("!"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    /// new_buffer.set("age", 20u8)?;
    /// // set value of name
    /// new_buffer.set("name", String::from("Bill Kerman"))?;
    /// // push value onto tags list
    /// new_buffer.list_push("tags", String::from("rocket"))?;
    /// 
    /// // get iterator of root (table)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    /// new_buffer.get_iter("tags")?.unwrap().into_iter().for_each(|item| {
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
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "map",
    ///    "value": {"type": "string"}
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set value of color key
    /// new_buffer.set("color", String::from("blue"))?;
    /// // set value of sport key
    /// new_buffer.set("sport", String::from("soccor"))?;
    /// 
    /// // get iterator of root (table)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    /// new_buffer.set("0", String::from("hello"))?;
    /// // set value at 2 index
    /// new_buffer.set("2", false)?;
    /// 
    /// // get iterator of root (tuple item)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    pub fn get_iter(&self, path: &str) -> Result<Option<NP_Generic_Iterator>, NP_Error> {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        let collection_ptr = root._deep_get(vec_path, 0)?;

        if let Some(iter) = collection_ptr {
            if iter.has_value() == false {
                Ok(None)
            } else {
                match &**iter.schema {
                    NP_Parsed_Schema::Table { i: _, sortable: _, columns: _} => {
                        Ok(Some(NP_Generic_Iterator {
                            index: 0,
                            iterator: NP_Iterator_Collection::Table(NP_Table::into_value(iter)?.unwrap().it())
                        }))
                    },
                    NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                        Ok(Some(NP_Generic_Iterator {
                            index: 0,
                            iterator: NP_Iterator_Collection::Map(NP_Map::into_value(iter)?.unwrap().it())
                        }))
                    },
                    NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                        Ok(Some(NP_Generic_Iterator {
                            index: 0,
                            iterator: NP_Iterator_Collection::List(NP_List::into_value(iter)?.unwrap().it())
                        }))
                    },
                    NP_Parsed_Schema::Tuple { i: _, sortable: _, values: _} => {
                        Ok(Some(NP_Generic_Iterator {
                            index: 0,
                            iterator: NP_Iterator_Collection::Tuple(NP_Tuple::into_value(iter)?.unwrap().it())
                        }))
                    },
                    _ => {
                        Err(NP_Error::new("Attempted to ierate on non collection!"))
                    }
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Push a value onto the end of a list.
    /// The path provided must resolve to a list type, and the type being pushed must match the schema
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set("3", String::from("launch"))?;
    /// new_buffer.list_push("", String::from("this"))?;
    /// new_buffer.list_push("", String::from("rocket"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    /// new_buffer.list_push("", String::from("launch"))?;
    /// new_buffer.list_push("", String::from("this"))?;
    /// new_buffer.list_push("", String::from("rocket"))?;
    /// 
    /// // get iterator of root (list item)
    /// new_buffer.get_iter("")?.unwrap().into_iter().for_each(|item| {
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
    pub fn list_push<'push, X: 'push>(&'push mut self, path: &str, value: X) -> Result<Option<u16>, NP_Error> where X: NP_Value<'push> + Default {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        let list_ptr = root._deep_set(vec_path, 0)?;

        match &**list_ptr.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                let mut list = NP_List::into_value(list_ptr).unwrap().unwrap();

                let mut new_ptr = list.push(None)?;

                // type does not match schema
                if X::type_idx().0 != new_ptr.schema.into_type_data().0 {
                    let mut err = "TypeError: Attempted to set value for type (".to_owned();
                    err.push_str(X::type_idx().1.as_str());
                    err.push_str(") into schema of type (");
                    err.push_str(new_ptr.schema.into_type_data().1.as_str());
                    err.push_str(")\n");
                    return Err(NP_Error::new(err));
                }

                let new_index = match new_ptr.helper { NP_Iterator_Helper::List { index, prev_addr: _, next_addr: _, next_index: _ } => index, _ => panic!()};

                new_ptr = NP_List::commit_pointer(new_ptr)?;

                X::set_value(&mut new_ptr, Box::new(&value))?;

                Ok(Some(new_index))
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
    /// 
    /// let factory: NP_Factory = NP_Factory::new(r#"{
    ///    "type": "string"
    /// }"#)?;
    /// 
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // get schema of root
    /// let type_key = new_buffer.get_schema("")?.unwrap();
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
    pub fn get_schema(&self, path: &str) -> Result<Option<&NP_Parsed_Schema>, NP_Error> {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        let collection_ptr = root._deep_get(vec_path, 0)?;

        if let Some(ptr) = collection_ptr {
            Ok(Some(&ptr.schema))
        } else {
            Ok(None)
        }
    }

    /// Get length of String, Bytes or Collection Type
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set("", String::from("hello"))?;
    /// // get length of value at root
    /// assert_eq!(new_buffer.length("")?, Some(5));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn length(&self, path: &str) -> Result<Option<usize>, NP_Error> {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        let collection_ptr = root._deep_get(vec_path, 0)?;

        if let Some(ptr) = collection_ptr {

            if ptr.address == 0 {
                return Ok(None);
            }

            match &**ptr.schema {
                NP_Parsed_Schema::List { i: _, sortable: _, of: _} => {
                    Ok(Some(NP_List::into_value(ptr)?.unwrap().it().into_iter().count()))
                },
                NP_Parsed_Schema::Map { i: _, sortable: _, value: _} => {
                    Ok(Some(NP_Map::into_value(ptr)?.unwrap().it().into_iter().count()))
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
                        let addr_size = self.memory.addr_size_bytes();
                        Ok(Some(self.memory.read_address(ptr.address + addr_size)))
                    }
                },
                NP_Parsed_Schema::UTF8String { i: _, sortable: _, default: _, size} => {
                    if *size > 0 {
                        Ok(Some(*size as usize))
                    } else {
                        let addr_size = self.memory.addr_size_bytes();
                        Ok(Some(self.memory.read_address(ptr.address + addr_size)))
                    }
                },
                _ => {
                    Ok(None)
                }
            }
        } else {
            Ok(None)
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set index 0
    /// new_buffer.set("0", String::from("hello"))?;
    /// // del index 0
    /// new_buffer.del("0")?;
    /// // value is gone now!
    /// assert_eq!(None, new_buffer.get::<String>("0")?);
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn del(&mut self, path: &str) -> Result<bool, NP_Error> {
        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        root._deep_delete(vec_path, 0)
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // third item in the top level list -> key "alpha" of map at 3rd element -> 9th element of list at "alpha" key
    /// // 
    /// new_buffer.set("3.alpha.9", String::from("who would build a schema like this"))?;
    /// 
    /// // get the same item we just set
    /// let message = new_buffer.get::<String>("3.alpha.9")?;
    /// 
    /// assert_eq!(message, Some(Box::new(String::from("who would build a schema like this"))));
    /// 
    /// # Ok::<(), NP_Error>(()) 
    /// ```
    /// 
    pub fn get<'get, X:'get>(&'get self, path: &'buffer str) -> Result<Option<Box<X>>, NP_Error> where X: NP_Value<'get> + Default {

        let vec_path: Vec<String> = path.split(".").filter_map(|v| { 
            if v.len() > 0 { Some(String::from(v)) } else { None }
        }).collect();
        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
        root._deep_get_type::<X>(vec_path, 0)
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set("", String::from("hello"))?;
    /// // using 11 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set("", String::from("hello, world"))?;
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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// // set initial value
    /// new_buffer.set("", String::from("hello"))?;
    /// // using 11 bytes
    /// assert_eq!(NP_Size_Data {
    ///     current_buffer: 11,
    ///     after_compaction: 11,
    ///     wasted_bytes: 0
    /// }, new_buffer.calc_bytes()?);
    /// // update the value
    /// new_buffer.set("", String::from("hello, world"))?;
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

        let old_root = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);
 
        let new_bytes = NP_Memory::new(Some(capacity), size);
        let mut new_root = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &new_bytes);

        old_root.compact(&mut new_root)?;

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
    /// let mut new_buffer = factory.empty_buffer(None, None);
    /// new_buffer.set("", String::from("hello"))?;
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

        let root: NP_Ptr = NP_Ptr::_new_standard_ptr(ROOT_PTR_ADDR, &self.schema.parsed, &self.memory);

        let real_bytes = root.calc_size()? + ROOT_PTR_ADDR;
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
    ptr: NP_Ptr<'item>
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
                
                let memory = self.ptr.memory;
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
    pub fn clear(&self) -> bool {
        self.ptr.clear_here()
    }
}

/// Iterator Enum
#[derive(Debug)]
pub enum NP_Iterator_Collection<'col> {
    /// Map
    Map(NP_Map_Iterator<'col>),
    /// List
    List(NP_List_Iterator<'col>),
    /// Table
    Table(NP_Table_Iterator<'col>),
    /// Tuple
    Tuple(NP_Tuple_Iterator<'col>)
}


/// Generic iterator
#[derive(Debug)]
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
                        ptr: p.clone()
                    };
                    self.index += 1;
                    Some(item)
                } else {
                    None
                }
            },
            NP_Iterator_Collection::List(x) => {
                if let Some(p) = x.next() {
                    let index = match p.helper {
                        NP_Iterator_Helper::List { index, prev_addr: _, next_index: _, next_addr: _ } => {
                            index
                        },
                        _ => panic!()
                    };

                    let item = NP_Item {
                        index: index as usize,
                        ptr: p.clone()
                    };

                    self.index += 1;
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
                        ptr: p.clone()
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