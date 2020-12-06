use alloc::string::String;
use crate::{hashmap::NP_HashMap, pointer::{NP_Cursor_Addr, NP_Cursor_Data}, schema::NP_Schema_Addr};
use crate::pointer::NP_Cursor;
use crate::{json_flex::JSMAP};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{hint::unreachable_unchecked};

/// The map type.
/// 
#[doc(hidden)]
pub struct NP_Map { }


impl NP_Map {


    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, parent_schema_addr: usize, memory: &NP_Memory<'parse>, of_schema: usize) {

        let list_value = NP_Cursor::parse_cursor_value(buff_addr, parent_addr, parent_schema_addr, &memory);

        let mut new_cursor = NP_Cursor { 
            buff_addr: buff_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: list_value, 
            parent_addr: parent_addr
        };

        let map_head = new_cursor.value.get_addr_value();

        let mut next_item = map_head;

        let mut map_addrs: NP_HashMap = NP_HashMap::new();

        while next_item != 0 {
            NP_Cursor::parse(next_item as usize, of_schema, buff_addr, schema_addr, &memory);
            let map_item = memory.get_parsed(&NP_Cursor_Addr::Real(next_item as usize));
            map_addrs.insert_hash(map_item.value.get_key_hash(), next_item as usize);
            next_item = map_item.value.get_next_addr();
        }
        
        new_cursor.data = NP_Cursor_Data::Map { value_map: map_addrs };
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

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        for (key, item) in NP_Map::new(cursor.clone(), memory) {
            json_map.insert(String::from(key), NP_Cursor::json_encode(&item, memory));
        }

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact(from_cursor: NP_Cursor_Addr, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'value + Sized {

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
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&["name"], "hello, world")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 34usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // values are preserved through compaction
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&["name"], "hello, world")?;
    buffer.set(&["name2"], "hello, world2")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 62usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello, world"));
    assert_eq!(buffer.get::<&str>(&["name2"])?, Some("hello, world2"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 62usize);

    Ok(())
}