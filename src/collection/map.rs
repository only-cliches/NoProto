use crate::schema::NP_Schema_Addr;
use crate::pointer::NP_Cursor_Parent;
use crate::pointer::NP_Cursor;
use crate::{json_flex::JSMAP, pointer::{NP_Cursor_Value}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{hint::unreachable_unchecked};

/// The map type.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Map<'map> {
    cursor: NP_Cursor,
    map: NP_Cursor_Parent,
    map_of_addr: NP_Schema_Addr,
    current: Option<(&'map str, NP_Cursor)>,
    memory: &'map NP_Memory<'map>
}


impl<'map> NP_Map<'map> {

    /// Create new map iterator
    /// 
    pub fn new(cursor: NP_Cursor, memory: &'map NP_Memory<'map>) -> Self {
        let value_addr = cursor.value.get_value_address();
        let addr_size = memory.addr_size_bytes();
        Self {
            cursor: cursor,
            map: NP_Cursor_Parent::Map {
                head: memory.read_address(value_addr),
                length: match memory.size {
                    NP_Size::U8 => memory.read_bytes()[value_addr + addr_size] as usize,
                    _ => u16::from_be_bytes(*memory.get_2_bytes(value_addr + addr_size).unwrap()) as usize
                },
                addr: value_addr,
                schema_addr: cursor.schema_addr
            },
            map_of_addr: match memory.schema[cursor.schema_addr] {
                NP_Parsed_Schema::Map { value, ..} => {
                    value
                },
                _ => { unsafe { unreachable_unchecked() } }
            },
            current: None,
            memory: memory
        }
    }

    /// Get the key for a given cursor
    pub fn get_key(cursor: NP_Cursor, memory: &'map NP_Memory<'map>) -> (&'map str, NP_Cursor) {
        match cursor.value {
            NP_Cursor_Value::MapItem { key_addr, .. } => {
                let addr_size = memory.addr_size_bytes();
                let key_size = memory.read_address(key_addr);
                
                let key_str = unsafe { core::str::from_utf8_unchecked(&memory.read_bytes()[(key_addr + addr_size)..(key_addr + addr_size + key_size)]) };
                (key_str, cursor)
            },
            _ => { unsafe { unreachable_unchecked() }}
        }
    }

    /// Read and/or write a map into the buffer
    /// 
    pub fn read_map(buff_addr: usize, schema_addr: usize, memory: &NP_Memory<'map>, create: bool) -> Result<(NP_Cursor, usize, usize), NP_Error> {

        let mut cursor = NP_Cursor::new(buff_addr, schema_addr, &memory, NP_Cursor_Parent::None);
        let mut value_addr = cursor.value.get_value_address();
        let addr_size = memory.addr_size_bytes();
        
        if value_addr == 0 { // no map here
            if create { // please make one
                assert_ne!(cursor.buff_addr, 0); 
                value_addr = match memory.size { // stores HEAD & LENGTH for map
                    NP_Size::U8 => {  memory.malloc_borrow(&[0u8; 2])? },
                    NP_Size::U16 => { memory.malloc_borrow(&[0u8; 4])? },
                    NP_Size::U32 => { memory.malloc_borrow(&[0u8; 6])? }
                };
                // update buffer
                memory.write_address(cursor.buff_addr, value_addr);
                // update cursor
                cursor.value = cursor.value.update_value_address(value_addr);
                Ok((cursor, 0, 0))
            } else { // no map and no need to make one, just pass empty data
                Ok((cursor, 0, 0))     
            }
        } else { // list found, read info from buffer
            Ok((cursor, memory.read_address(value_addr), match memory.size {
                NP_Size::U8 => memory.read_bytes()[value_addr + addr_size] as usize,
                _ => u16::from_be_bytes(*memory.get_2_bytes(value_addr + addr_size).unwrap()) as usize
            }))
        }
    }




    /// Accepts a cursor that is currently on a list type and moves the cursor to a list item
    /// The list item may be virtual
    pub fn select_into(cursor: NP_Cursor, memory: &NP_Memory<'map>, key: &'map str, create_path: bool, quick_select: bool) -> Result<NP_Cursor, NP_Error> {
       
        let map_of_schema_addr = match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                *value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let addr_size = memory.addr_size_bytes();

        let (map_cursor, mut head, length) = Self::read_map(cursor.buff_addr, cursor.schema_addr, &memory, create_path)?;

        let map_value_addr = map_cursor.value.get_value_address();

        // map is empty, return virtual pointer
        if head == 0 {
            let mut virtual_cursor = NP_Cursor::new(0, map_of_schema_addr, memory, NP_Cursor_Parent::Map { head: head, addr: map_cursor.buff_addr, schema_addr: map_cursor.schema_addr, length });

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // update head 
                head = virtual_cursor.buff_addr;
                memory.write_address(map_value_addr, head);

                // write key into buffer
                let key_len = key.len();
                let key_addr = match memory.size {
                    NP_Size::U8 => memory.malloc_borrow(&[key_len as u8]),
                    NP_Size::U16 => memory.malloc_borrow(&(key_len as u16).to_be_bytes()),
                    NP_Size::U32 => memory.malloc_borrow(&(key_len as u32).to_be_bytes()),
                }?;
                memory.malloc_borrow(key.as_bytes())?;
                memory.write_address(virtual_cursor.buff_addr + addr_size + addr_size, key_addr);

                virtual_cursor.value = NP_Cursor_Value::MapItem { value_addr: 0, key_addr: key_addr, next: 0 };
            }

            virtual_cursor.parent = NP_Cursor_Parent::Map { head: head, addr: map_cursor.buff_addr, schema_addr: map_cursor.schema_addr, length };

            return Ok(virtual_cursor)
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate key
        if quick_select {
            let mut virtual_cursor = NP_Cursor::new(0, map_of_schema_addr, memory, NP_Cursor_Parent::Map { head, length, addr: map_cursor.buff_addr , schema_addr: map_cursor.schema_addr});
            virtual_cursor.value = NP_Cursor_Value::MapItem { value_addr: 0, key_addr: 0, next: 0 };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // update NEXT to old head
                memory.write_address(virtual_cursor.buff_addr + addr_size, head);
                let next_addr = head;

                // update head 
                head = virtual_cursor.buff_addr;
                memory.write_address(map_value_addr, head);

                // write key into buffer
                let key_len = key.len();
                let key_addr = match memory.size {
                    NP_Size::U8 => memory.malloc_borrow(&[key_len as u8]),
                    NP_Size::U16 => memory.malloc_borrow(&(key_len as u16).to_be_bytes()),
                    NP_Size::U32 => memory.malloc_borrow(&(key_len as u32).to_be_bytes()),
                }?;
                memory.malloc_borrow(key.as_bytes())?;
                memory.write_address(virtual_cursor.buff_addr + addr_size + addr_size, key_addr);

                virtual_cursor.value = NP_Cursor_Value::MapItem { value_addr: 0, key_addr: key_addr, next: next_addr };
            }

            return Ok(virtual_cursor)
        }

        let mut running_ptr: usize = 0;

        for (ikey, item) in NP_Map::new(map_cursor.clone(), memory) {
            if key == ikey {
                return Ok(item.clone())
            }
            running_ptr = item.buff_addr;
        }


        let mut virtual_cursor = NP_Cursor::new(0, map_of_schema_addr, memory, NP_Cursor_Parent::Map { head, length, addr: map_cursor.buff_addr , schema_addr: map_cursor.schema_addr});
        virtual_cursor.value = NP_Cursor_Value::MapItem { value_addr: 0, key_addr: 0, next: 0 };

        if create_path {
            virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

            // update previous pointer NEXT to this one
            memory.write_address(running_ptr + addr_size, virtual_cursor.buff_addr);

            // write key into buffer
            let key_len = key.len();
            let key_addr = match memory.size {
                NP_Size::U8 => memory.malloc_borrow(&[key_len as u8]),
                NP_Size::U16 => memory.malloc_borrow(&(key_len as u16).to_be_bytes()),
                NP_Size::U32 => memory.malloc_borrow(&(key_len as u32).to_be_bytes()),
            }?;
            memory.malloc_borrow(key.as_bytes())?;
            memory.write_address(virtual_cursor.buff_addr + addr_size + addr_size, key_addr);

            virtual_cursor.value = NP_Cursor_Value::MapItem { value_addr: 0, key_addr: key_addr, next: 0 };
        }

        return Ok(virtual_cursor)
    }
}

impl<'value> NP_Value<'value> for NP_Map<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    
    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let value_of = match schema[address] {
            NP_Parsed_Schema::Map { value, .. } => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("value".to_owned(), NP_Schema::_type_to_json(schema, value_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn get_size(cursor: NP_Cursor, memory: &NP_Memory) -> Result<usize, NP_Error> {

        if cursor.value.get_value_address() == 0 {
            return Ok(0) 
        }

        let base_size = match &memory.size {
            NP_Size::U8  => { 2usize }, // u8 head | u8 length
            NP_Size::U16 => { 4usize }, // u16 head | u16 length
            NP_Size::U32 => { 6usize }  // u32 head | u16 length
        };

        let addr_size = memory.addr_size_bytes();

        let mut acc_size = 0usize;

        for (key, item) in NP_Map::new(cursor.clone(), memory) {
            acc_size += addr_size; // key length bytes
            acc_size += key.len();
            acc_size += NP_Cursor::calc_size(item.clone(), memory)?; // item
        };

        Ok(acc_size + base_size)
   
    }

    fn to_json(cursor: &NP_Cursor, memory: &NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        for (key, item) in NP_Map::new(cursor.clone(), memory) {
            json_map.insert(String::from(key), NP_Cursor::json_encode(&item, memory));
        }

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact(from_cursor: &NP_Cursor, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor, NP_Error> where Self: 'value {

        if from_cursor.buff_addr == 0 {
            return Ok(to_cursor);
        }

        for (old_key, old_item) in NP_Map::new(from_cursor.clone(), from_memory) {
            if old_item.buff_addr != 0 && old_item.value.get_value_address() != 0 { // pointer has value
                let new_item = NP_Map::select_into(to_cursor, to_memory, old_key, true, true)?;
                NP_Cursor::compact(&old_item, from_memory, new_item, to_memory)?;
            }
        }

        Ok(to_cursor)
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
      
        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Map as u8);

        let value_addr = schema.len();
        schema.push(NP_Parsed_Schema::Map {
            i: NP_TypeKeys::Map,
            value: value_addr + 1,
            sortable: false
        });

        match json_schema["value"] {
            NP_JSON::Null => {
                return Err(NP_Error::new("Maps require a 'value' property that is a schema type!"))
            },
            _ => { }
        }

        
        let (_sortable, child_bytes, schema) = NP_Schema::from_json(schema, &Box::new(json_schema["value"].clone()))?;
        
        schema_data.extend(child_bytes);

        return Ok((false, schema_data, schema))

    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let of_addr = schema.len();
        schema.push(NP_Parsed_Schema::Map {
            i: NP_TypeKeys::Map,
            sortable: false,
            value: of_addr + 1
        });
        let (_sortable, schema) = NP_Schema::from_bytes(schema, address + 1, bytes);
        (false, schema)
    }
}



impl<'it> Iterator for NP_Map<'it> {
    type Item = (&'it str, NP_Cursor);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.current { // step pointer
            if let Some(next) = current.1.next_cursor {

                let mut next_cursor = NP_Cursor::new(next, current.1.schema_addr, &self.memory, current.1.parent.clone());

                let next_next_addr = match next_cursor.value {
                    NP_Cursor_Value::MapItem { next, ..} => { next },
                    _ => { unsafe { unreachable_unchecked() } }
                };


                next_cursor.prev_cursor = Some(current.1.buff_addr);

                if next_next_addr == 0 {
                    next_cursor.next_cursor = None;
                } else {
                    next_cursor.next_cursor = Some(next_next_addr);
                }
                
                self.current = Some(NP_Map::get_key(next_cursor, self.memory));

                self.current
            } else { // nothing left in map
                None
            }

        } else { // make first pointer

            let (map, head, length) = Self::read_map(self.cursor.buff_addr, self.cursor.schema_addr, self.memory, true).unwrap();

            // nothing here bro
            if head == 0 {
                return None;
            }

            let map_of_schema_addr = match &self.memory.schema[self.cursor.schema_addr] {
                NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                    *value
                },
                _ => { unsafe { unreachable_unchecked() } }
            };

            let mut first_cursor = NP_Cursor::new(head, map_of_schema_addr, &self.memory, NP_Cursor_Parent::Map { addr: map.buff_addr, head, length, schema_addr: self.cursor.schema_addr });

            match first_cursor.value {
                NP_Cursor_Value::MapItem { next, ..} => {
                    if next != 0 {
                        first_cursor.next_cursor = Some(next);
                    }
                },
                _ => { unsafe { unreachable_unchecked() }}
            }

            self.current = Some(NP_Map::get_key(first_cursor, self.memory));

            self.current
        }
    }

    fn count(self) -> usize where Self: Sized {

        if self.cursor.buff_addr == 0 {
            return 0;
        }

        let (_cursor, _head, length) = Self::read_map(self.cursor.buff_addr,  self.cursor.schema_addr, self.memory, false).unwrap();

        return length;
    }
}


#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"map\",\"value\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"map\",\"value\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;

    // compaction works
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&["name"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 34usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // values are preserved through compaction
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&["name"], "hello, world")?;
    buffer.set(&["name2"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 62usize);
    buffer.compact(None, None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 62usize);

    Ok(())
}