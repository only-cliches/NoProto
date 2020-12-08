use alloc::string::String;
use crate::{hashmap::{NP_HashMap, SEED, murmurhash3_x86_32}, pointer::{NP_Cursor_Addr, NP_Cursor_Data}, schema::NP_Schema_Addr};
use crate::pointer::NP_Cursor;
use crate::{json_flex::JSMAP};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{str::from_utf8_unchecked, hint::unreachable_unchecked};

/// The map type.
/// 
#[doc(hidden)]
pub struct NP_Map { 
    cursor: NP_Cursor_Addr,
    index: usize,
    value: Option<NP_Cursor_Addr>,
    value_of: usize
}

impl NP_Map {

    pub fn make_map<'make>(map_cursor_addr: &NP_Cursor_Addr, memory: &'make NP_Memory) -> Result<(), NP_Error> {

        let cursor = memory.get_parsed(map_cursor_addr);
            
        cursor.data = NP_Cursor_Data::Map { value_map: NP_HashMap::new() };

        Ok(())
    }


    pub fn new_iter(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Self {
        let map_cursor = memory.get_parsed(cursor_addr);

        let value_of = match memory.schema[map_cursor.schema_addr] {
            NP_Parsed_Schema::Map { value, .. } => value,
            _ => unsafe { unreachable_unchecked() }
        };

        Self {
            cursor: cursor_addr.clone(),
            value: None,
            index: 0,
            value_of
        }
    }

    pub fn step_iter(map: &mut Self, memory: &NP_Memory) -> Option<(usize, NP_Cursor_Addr)> {
        let map_cursor = memory.get_parsed(&map.cursor);

        if map.value == Option::None { // first iteration, get head

            let head = map_cursor.value.get_addr_value() as usize;
            if head == 0 {
                return None
            } else {
                map.value = Some(NP_Cursor_Addr::Real(head));
                map.index += 1;
                return Some((map.index - 1, map.value.unwrap()))
            }

        } else { // subsequent iterations
            let this_item_addr = map.value.unwrap();
            let this_item = memory.get_parsed(&this_item_addr);
            
            let next_item_addr = this_item.value.get_next_addr() as usize;
            if next_item_addr == 0 {
                return None;
            } else {
                let this_cursor = NP_Cursor_Addr::Real(next_item_addr);

                map.index += 1;

                map.value = Some(this_cursor.clone());

                return Some((map.index - 1, this_cursor))
            }
        }
    }

    pub fn insert(map_cursor_addr: &NP_Cursor_Addr, schema_addr: NP_Schema_Addr, memory: &NP_Memory, key: &str) -> Result<NP_Cursor_Addr, NP_Error> {
        let new_item_addr = memory.malloc_borrow(&[0u8; 6])?;

        let map_cursor = memory.get_parsed(&map_cursor_addr);

        let new_item_cursor = NP_Cursor { 
            buff_addr: new_item_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: NP_Cursor::parse_cursor_value(new_item_addr, map_cursor.buff_addr, map_cursor.schema_addr, &memory), 
            parent_addr: map_cursor.buff_addr,
            index: 0
        };

        if key.len() >= 255 {
            return Err(NP_Error::new("Key length cannot be larger than 255 charecters!"));
        }

        // set key
        let key_item_addr = memory.malloc_borrow(&[key.len() as u8])?;
        memory.malloc_borrow(key.as_bytes())?;
        new_item_cursor.value.set_key_addr(key_item_addr as u16);

        let head = map_cursor.value.get_addr_value();

        map_cursor.value.set_addr_value(new_item_addr as u16);

        if head != 0 { // set new cursors NEXT to old HEAD
            new_item_cursor.value.set_next_addr(head);
        }

        let key_hash = murmurhash3_x86_32(key.as_bytes(), SEED);
        match &mut map_cursor.data {
            NP_Cursor_Data::Map { value_map } => {
                value_map.insert_hash(key_hash, new_item_addr)?;
            },
            _ => unsafe { unreachable_unchecked() }
        }

        memory.insert_parsed(new_item_addr, new_item_cursor);

        Ok(NP_Cursor_Addr::Real(new_item_addr))
    }

    pub fn for_each<F>(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory, callback: &mut F) where F: FnMut((usize, NP_Cursor_Addr)) {

        let mut map_iter = Self::new_iter(cursor_addr, memory);

        while let Some((index, item)) = Self::step_iter(&mut map_iter, memory) {
            callback((index, item))
        }

    }

    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, parent_schema_addr: usize, memory: &NP_Memory<'parse>, of_schema: usize, index: usize) {

        let list_value = NP_Cursor::parse_cursor_value(buff_addr, parent_addr, parent_schema_addr, &memory);

        let mut new_cursor = NP_Cursor { 
            buff_addr: buff_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: list_value, 
            parent_addr: parent_addr,
            index
        };

        let map_head = new_cursor.value.get_addr_value();

        let mut next_item = map_head;

        let mut map_addrs: NP_HashMap = NP_HashMap::new();


        while next_item != 0 {
            NP_Cursor::parse(next_item as usize, of_schema, buff_addr, schema_addr, &memory, 0);
            let map_item = memory.get_parsed(&NP_Cursor_Addr::Real(next_item as usize));
            let key_addr = map_item.value.get_key_addr() as usize;
            let key_length = memory.read_bytes()[key_addr] as usize;
            let key_data = &memory.read_bytes()[(key_addr + 1)..(key_addr + 1 + key_length)];
            let key_hash = murmurhash3_x86_32(key_data, SEED);
            map_addrs.insert_hash(key_hash, next_item as usize);
            next_item = map_item.value.get_next_addr();
        }
        
        new_cursor.data = NP_Cursor_Data::Map { value_map: map_addrs};
        memory.insert_parsed(buff_addr, new_cursor);
    }
}

impl<'value> NP_Value<'value> for NP_Map {

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

    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        Self::for_each(&cursor, memory, &mut |(_i, item)| {
            let key_addr = memory.get_parsed(&item).value.get_key_addr() as usize;
            let key_size = memory.read_bytes()[key_addr] as usize;
            acc_size += 1; // length byte
            acc_size += key_size;
            acc_size += NP_Cursor::calc_size(item.clone(), memory).unwrap();
        });

        Ok(acc_size)
   
    }

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        Self::for_each(&cursor, memory, &mut |(_i, item)| {
            let key_addr = memory.get_parsed(&item).value.get_key_addr() as usize;
            let key_size = memory.read_bytes()[key_addr] as usize;
            let key_bytes = &memory.read_bytes()[(key_addr + 1)..(key_addr + 1 + key_size)];
            json_map.insert(String::from(unsafe{ from_utf8_unchecked(key_bytes) }), NP_Cursor::json_encode(item.clone(), memory));
        });

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact(from_cursor: &NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: Sized {

        let from_c = from_memory.get_parsed(from_cursor);
 
        if from_c.value.get_addr_value() == 0 {
            return Ok(to_cursor);
        }

        Self::make_map(&to_cursor, to_memory)?;

       let to_c = to_memory.get_parsed(&to_cursor);

        let value_of = match from_memory.schema[from_c.schema_addr] {
            NP_Parsed_Schema::Map { value, .. } => value,
            _ => unsafe { unreachable_unchecked() }
        };
        Self::for_each(from_cursor, from_memory,  &mut |(index, item)| {
            let old_item = from_memory.get_parsed(&item);
            if old_item.buff_addr != 0 && old_item.value.get_addr_value() != 0 { // pointer has value

                let key = old_item.value.get_key(from_memory);
                let new_item = Self::insert(&to_cursor, value_of, to_memory, key).unwrap();
                NP_Cursor::compact(&item, from_memory, new_item, to_memory).unwrap();
            }    
        });

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
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["name"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 27usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 2usize);

    // values are preserved through compaction
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["name"], "hello, world")?;
    buffer.set(&["name2"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 54usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 54usize);

    Ok(())
}