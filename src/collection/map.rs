use crate::memory::{blank_ptr_u16_map_item, blank_ptr_u32_map_item, blank_ptr_u8_map_item};
use crate::pointer::NP_Cursor;
use crate::{json_flex::JSMAP, pointer::{NP_Cursor_Addr, NP_Cursor_Kinds}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{str::from_utf8_unchecked, hint::unreachable_unchecked};

use super::NP_Collection;

/// The map type.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Map<'map> {
    cursor: NP_Cursor_Addr,
    current: Option<NP_Cursor_Addr>,
    pub memory: NP_Memory<'map>
}


impl<'map> NP_Map<'map> {

    /// reads from buffer to get data about this cursor
    pub fn cache_map_item(cursor_addr: &NP_Cursor_Addr, schema: &'map Box<NP_Parsed_Schema>, parent: usize, memory: NP_Memory<'map>) -> Result<(), NP_Error> {
        
        // should never attempt to cache a virtual cursor
        if cursor_addr.is_virtual { panic!() }

        // real map item in buffer, (maybe) needs to be cached
        match cursor_addr.get_data(&memory) {
            Ok(_x) => { /* already in cache */ },
            Err(_e) => {
                let mut new_cursor = NP_Cursor::new(cursor_addr.address, Some(parent), &memory, schema);

                let addr_size = memory.addr_size_bytes();

                new_cursor.schema = schema;
                new_cursor.parent_addr = Some(parent);
                new_cursor.address_value = memory.read_address(new_cursor.address);
                new_cursor.item_next_addr = Some(memory.read_address(new_cursor.address + addr_size));
                let key_addr = memory.read_address(new_cursor.address + addr_size + addr_size);
                new_cursor.item_key_addr = Some(key_addr);
                let key_size = memory.read_address(key_addr);
                let key_bytes = &memory.read_bytes()[(key_addr + addr_size)..(key_addr + addr_size + key_size)];
                new_cursor.item_key = Some(unsafe { from_utf8_unchecked(key_bytes) });
                new_cursor.kind = NP_Cursor_Kinds::MapItem;

                memory.insert_cache(new_cursor);          
            }
        }

        Ok(())
    }

    /// Get details of map object at this location in buffer
    pub fn commit_or_cache_map(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<(), NP_Error> {
        
        let cursor = cursor_addr.get_data(&memory)?;

        if cursor_addr.is_virtual { // virtual cursor, just return blank details
            cursor.coll_head = Some(0);
            cursor.coll_length = Some(0);
            cursor.address_value = 0;
            cursor.kind = NP_Cursor_Kinds::None;
        } else if cursor.address_value == 0 { // real cursor but need to make list
            cursor.coll_head = Some(0);
            cursor.coll_length = Some(0);

            match memory.size {
                NP_Size::U8 => {
                    cursor.address_value = memory.malloc_borrow(&[0u8; 2])?; // stores HEAD & LENGTH (u8) for map
                },
                NP_Size::U16 => {  
                    cursor.address_value = memory.malloc_borrow(&[0u8; 4])?; // stores HEAD & LENGTH (u16) for map
                },
                NP_Size::U32 => {
                    cursor.address_value = memory.malloc_borrow(&[0u8; 6])?; // stores HEAD & LENGTH (u16) for map
                }
            };
            memory.set_value_address(cursor.address, cursor.address_value);

            cursor.kind = NP_Cursor_Kinds::Map;
        } else if cursor.kind == NP_Cursor_Kinds::Standard { // real cursor with value, need to cache list values

            cursor.coll_head = Some(memory.read_address(cursor.address_value));
            cursor.coll_length = Some(match memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([memory.get_1_byte(cursor.address_value + 1).unwrap_or(0)]) as usize
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*memory.get_2_bytes(cursor.address_value + 2).unwrap_or(&[0; 2])) as usize
                },
                NP_Size::U32 => {
                    u16::from_be_bytes(*memory.get_2_bytes(cursor.address_value + 4).unwrap_or(&[0; 2])) as usize
                }
            });

            cursor.kind = NP_Cursor_Kinds::Map;
        }
        
        Ok(())
    }


    /// Accepts a cursor that is currently on a list type and moves the cursor to a list item
    /// The list item may be virtual
    pub fn select_to_ptr<'sel>(cursor_addr: NP_Cursor_Addr, memory: &NP_Memory, key: &str, quick_select: bool) -> Result<NP_Cursor_Addr, NP_Error> {
       
        NP_Map::commit_or_cache_map(&cursor_addr, &memory)?;

        let map_cursor = cursor_addr.get_data(&memory)?;

        let map_of = match &**map_cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let head = map_cursor.coll_head.unwrap();

        // map is empty, return virtual pointer
        if head == 0 {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = map_of;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::MapItem;
            virtual_cursor.item_index = None;
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = None;
            virtual_cursor.item_key = Some(key.to_string().as_str());
            virtual_cursor.item_key_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate key
        if quick_select {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = map_of;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::MapItem;
            virtual_cursor.item_index = None;
            virtual_cursor.item_prev_addr = Some(head);
            virtual_cursor.item_next_addr = None;
            virtual_cursor.item_key = Some(key.to_string().as_str());
            virtual_cursor.item_key_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
        }

        let mut running_ptr: usize = 0;

        // key might be somewhere in existing records
        for item in NP_Map::start_iter(&cursor_addr, memory.clone())? {

            let map_item = memory.get_cursor_data(&item)?;
            if map_item.item_key.unwrap() == key {
                return Ok(item.clone())
            }

            running_ptr = item.address;
        }

        // key doesn't exist
        let virtual_cursor = memory.get_virt_cursor();
        virtual_cursor.address = 0;
        virtual_cursor.address_value = 0;
        virtual_cursor.schema = map_of;
        virtual_cursor.parent_addr = Some(cursor_addr.address);
        virtual_cursor.kind = NP_Cursor_Kinds::MapItem;
        virtual_cursor.item_index = None;
        virtual_cursor.item_prev_addr = Some(running_ptr);
        virtual_cursor.item_next_addr = None;
        virtual_cursor.item_key = Some(key.to_string().as_str());
        virtual_cursor.item_key_addr = None;

        return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
    }
}

impl<'value> NP_Value<'value> for NP_Map<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    
    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let value_of = match &*schema_ptr {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("value".to_owned(), NP_Schema::_type_to_json(&value_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }
    
    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {
        Err(NP_Error::new("Type (map) doesn't support .set()! Use .into() instead."))
    }

    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory<'value>) -> Result<usize, NP_Error> {

        if cursor_addr.is_virtual {
            return Ok(0);     
        }

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return Ok(0);
        }

        let base_size = match &memory.size {
            NP_Size::U8  => { 2usize }, // u8 head | u8 length
            NP_Size::U16 => { 4usize }, // u16 head | u16 length
            NP_Size::U32 => { 6usize }  // u32 head | u16 length
        };

        let addr_size = memory.addr_size_bytes();

        let mut acc_size = 0usize;

        let map_of = match &**cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        for l in Self::start_iter(&cursor_addr, memory.clone())? {
            let map_item = memory.get_cursor_data(&l)?;
            acc_size += addr_size; // key length bytes
            acc_size += map_item.item_key.unwrap().len(); // key length
            acc_size += NP_Cursor::calc_size(l, memory).unwrap(); // item
        };

        Ok(acc_size + base_size)
   
    }

    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        if cursor_addr.is_virtual {
            return NP_JSON::Null;
        }

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return NP_JSON::Null;
        }

        let mut json_map = JSMAP::new();

        let map_of = match &**cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        for item in NP_Map::start_iter(&cursor_addr, memory.clone()).unwrap() {

            let map_item = memory.get_cursor_data(&item).unwrap();
            let key = map_item.item_key.unwrap().to_string();

            json_map.insert(key, NP_Cursor::json_encode(item.clone(), memory));
        }

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact(from_cursor_addr: NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor_addr: NP_Cursor_Addr, to_memory: &'value NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: NP_Value<'value> {

        if from_cursor_addr.address == 0 {
            return Ok(to_cursor_addr);
        }

        NP_Map::commit_or_cache_map(&from_cursor_addr, from_memory).unwrap();
        NP_Map::commit_or_cache_map(&to_cursor_addr, to_memory).unwrap();

        let from_cursor = from_memory.get_cursor_data(&from_cursor_addr)?;

        let map_of = match &**from_cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        for old_item in NP_Map::start_iter(&from_cursor_addr, from_memory.clone()).unwrap() {
            if old_item.address != 0 { // pointer is not virutal
                let old_cursor = from_memory.get_cursor_data(&old_item)?;
                if old_cursor.address_value != 0 { // pointer has value
                    let index = old_cursor.item_index.unwrap();
                    let mut new_item = NP_Map::select_to_ptr(to_cursor_addr.clone(), to_memory, old_cursor.item_key.unwrap(), true)?;
                    NP_Map::commit_pointer(&new_item, to_memory.clone())?;
                    NP_Cursor::compact(old_item, from_memory, new_item, to_memory)?;
                }
            }
        }

        Ok(to_cursor_addr)
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {
        let type_str = NP_Schema::_get_type(json_schema)?;

        if "map" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Map as u8);

            match json_schema["value"] {
                NP_JSON::Null => {
                    return Err(NP_Error::new("Maps require a 'value' property that is a schema type!"))
                },
                _ => { }
            }

            let child_type = NP_Schema::from_json(Box::new(json_schema["value"].clone()))?;
            schema_data.extend(child_type.0);
            return Ok(Some((schema_data, NP_Parsed_Schema::Map {
                i: NP_TypeKeys::Map,
                value: Box::new(child_type.1),
                sortable: false
            })))
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<&'value Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        NP_Parsed_Schema::List {
            i: NP_TypeKeys::List,
            sortable: false,
            of: Box::new(NP_Schema::from_bytes(address + 1, bytes))
        }
    }
}



impl<'collection> NP_Collection<'collection> for NP_Map<'collection> {


    fn start_iter(map_cursor_addr: &NP_Cursor_Addr, memory: NP_Memory<'collection>) -> Result<Self, NP_Error> {
        
        NP_Map::commit_or_cache_map(&map_cursor_addr, &memory)?;

        let map_cursor = memory.get_cursor_data(&map_cursor_addr)?;
        
        let map_of = match &**map_cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let addr_size = memory.addr_size_bytes();

        let head = map_cursor.coll_head.unwrap();


        Ok(if head != 0 { // map has objectes, return the first one
            let head_cursor_addr = NP_Cursor_Addr { address: head, is_virtual: false};
            Self::cache_map_item(&head_cursor_addr, map_of, map_cursor_addr.address, memory.clone());
            let head_cursor = memory.get_cursor_data(&head_cursor_addr)?;
            Self {
                cursor: map_cursor_addr.clone(),
                current: Some(head_cursor_addr),
                memory: memory
            }
        } else { // no map objects
            Self {
                cursor: map_cursor_addr.clone(),
                current: None,
                memory: memory
            }
        })
    }

    /// Step a pointer to the next item in the collection
    fn step_pointer(&self, cursor_addr: &NP_Cursor_Addr) -> Option<NP_Cursor_Addr> {
        // can't step with virtual pointer
        if cursor_addr.is_virtual {
            return None;
        }

        // save current pointer as previous pointer for next pointer
        let prev_addr = cursor_addr.address;

        let cursor = self.cursor_addr.get_data(&memory).unwrap();

        if cursor.item_next_addr.unwrap() == 0 { // no more pointers
            return None;
        }

        let mut next_cursor = NP_Cursor_Addr { address: cursor.item_next_addr.unwrap(), is_virtual: false};

        let map_cursor = self.memory.get_cursor_data(&self.cursor).unwrap();

        let map_of = match &**map_cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        Self::cache_map_item(&next_cursor, map_of, self.current.unwrap().address, self.memory.clone()).unwrap();

        let next_cursor_item = self.memory.get_cursor_data(&next_cursor).unwrap();

        next_cursor_item.item_prev_addr = Some(prev_addr);

        return Some(next_cursor)
    }

    /// Commit a virtual pointer into the buffer
    fn commit_pointer<'mem>(cursor_addr: &NP_Cursor_Addr, memory: NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> {

        // pointer already committed
        if cursor_addr.address != 0 {
            return Ok(cursor_addr.clone());
        }

        if cursor_addr.is_virtual == false { panic!() }

        let cursor = memory.get_virt_cursor();

        let parent_addr = cursor.parent_addr.unwrap();

        let map_cursor = memory.get_cursor_data(&NP_Cursor_Addr { address: parent_addr, is_virtual: false})?;

        let map_of = match &**map_cursor.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        cursor.address = match &memory.size {
            NP_Size::U8 => {
                memory.malloc_borrow(&blank_ptr_u8_map_item())?
            },
            NP_Size::U16 => {
                memory.malloc_borrow(&blank_ptr_u16_map_item())?
            },
            NP_Size::U32 => {
                memory.malloc_borrow(&blank_ptr_u32_map_item())?
            }
        };

        let addr_size = memory.addr_size_bytes();
        
        // write key to buffer
        let key_length = cursor.item_key.unwrap().len();

        let key_address = match &memory.size {
            NP_Size::U8 => { memory.malloc_borrow(&(key_length as u8).to_be_bytes())? },
            NP_Size::U16 => { memory.malloc_borrow(&(key_length as u16).to_be_bytes())? },
            NP_Size::U32 => { memory.malloc_borrow(&(key_length as u32).to_be_bytes())? },
        };

        memory.malloc_borrow(cursor.item_key.unwrap().as_bytes())?;

        memory.set_value_address(cursor.address + addr_size + addr_size, key_address);



        if map_cursor.coll_head.unwrap() == 0 { // empty map
            // update head
            memory.set_value_address(parent_addr, cursor.address);
            map_cursor.coll_head = Some(cursor.address);
        } else if map_cursor.coll_head.unwrap() == cursor.item_prev_addr.unwrap() { // inserting at beginning

            
            let head = map_cursor.coll_head.unwrap();
            // update this cursor's next value to previous head
            memory.set_value_address(cursor.address + addr_size, head);
            cursor.item_next_addr = Some(head);

            // update head
            memory.set_value_address(parent_addr, cursor.address);
            map_cursor.coll_head = Some(cursor.address);


        } else { // inserting at end
            let prev_pointer_addr = cursor.item_prev_addr.unwrap();
            let prev_pointer_cursor = NP_Cursor_Addr { address: prev_pointer_addr, is_virtual: false};
            NP_Map::cache_map_item(&prev_pointer_cursor, map_of, cursor.parent_addr.unwrap(), memory)?;
            // update previous cursor's NEXT to the cursor we just made
            memory.set_value_address(prev_pointer_addr + addr_size, cursor.address);
            let prev_pointer_data = memory.get_cursor_data(&prev_pointer_cursor)?;
            prev_pointer_data.item_next_addr = Some(cursor.address);
        }

        Ok(NP_Cursor_Addr { address: cursor.address, is_virtual: false})
    }
}


impl<'it> Iterator for NP_Map<'it> {
    type Item = NP_Cursor_Addr;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(x) => {
                let current = x.clone();
                self.current = self.step_pointer(&current);
                Some(current)
            },
            None => None
        }
    }

    fn count(self) -> usize where Self: Sized {

        NP_Map::commit_or_cache_map(&self.cursor, &self.memory);

        let map_cursor = self.memory.get_cursor_data(&self.cursor).unwrap();

        map_cursor.coll_length.unwrap()

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
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&["name"], String::from("hello, world"))?;
    assert_eq!(buffer.get::<String>(&["name"])?, Some(Box::new(String::from("hello, world"))));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 34usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}