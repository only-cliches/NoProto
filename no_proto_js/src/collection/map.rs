use alloc::string::String;
use crate::{pointer::NP_Map_Bytes};
use crate::pointer::NP_Cursor;
use crate::{json_flex::JSMAP};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
struct Map_Item<'item> {
    key: &'item str,
    buff_addr: usize
}

impl<'item> Map_Item<'item> {
    pub fn new(key: &'item str, buff_addr: usize) -> Self {
        Self { key, buff_addr}
    }
}

/// The map type.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Map<'map> { 
    current: Option<Map_Item<'map>>,
    head: Option<Map_Item<'map>>,
    map: NP_Cursor,
    value_of: usize
}

#[allow(missing_docs)]
impl<'map> NP_Map<'map> {

    #[inline(always)]
    pub fn select<M: NP_Memory>(map_cursor: NP_Cursor, key: &str, make_path: bool, memory: &'map M) -> Result<Option<NP_Cursor>, NP_Error> {

        let mut map_iter = Self::new_iter(&map_cursor, memory);

        // key is maybe in map
        while let Some((ikey, item)) = map_iter.step_iter(memory) {
            if ikey == key {
                return Ok(Some(item.clone()))
            }
        }

        // key is not in map
        if make_path {
            Ok(Some(Self::insert(&map_cursor, memory, key)?))
        } else {
            Ok(None)
        }
    }

    #[inline(always)]
    pub fn get_map<'get, M: NP_Memory>(map_buff_addr: usize, memory: &'get M) -> &'get mut NP_Map_Bytes {
        if map_buff_addr > memory.read_bytes().len() { // attack
            unsafe { &mut *(memory.write_bytes().as_ptr() as *mut NP_Map_Bytes) }
        } else { // normal operation
            unsafe { &mut *(memory.write_bytes().as_ptr().add(map_buff_addr as usize) as *mut NP_Map_Bytes) }
        }
    }

    #[inline(always)]
    pub fn new_iter<M: NP_Memory>(map_cursor: &NP_Cursor, memory: &'map M) -> Self {

        let value_of = match memory.get_schema(map_cursor.schema_addr) {
            NP_Parsed_Schema::Map { value, .. } => *value,
            _ => 0
        };

        if map_cursor.get_value(memory).get_addr_value() == 0 {
            return Self {
                current: None,
                head: None,
                map: map_cursor.clone(),
                value_of
            }
        }

        let head_addr = Self::get_map(map_cursor.buff_addr, memory).get_head();

        let head_cursor = NP_Cursor::new(head_addr as usize, value_of, map_cursor.schema_addr);
        let head_cursor_value = head_cursor.get_value(memory);

        Self {
            current: None,
            head: Some(Map_Item::new(head_cursor_value.get_key(memory), head_cursor.buff_addr )),
            map: map_cursor.clone(),
            value_of
        }
    }

    #[inline(always)]
    pub fn step_iter<M: NP_Memory>(&mut self, memory: &'map M) -> Option<(&'map str, NP_Cursor)> {
        
        match self.head {
            Some(head) => {

                match self.current {
                    Some(current) => { // subsequent iterations
                        let current_item = NP_Cursor::new(current.buff_addr, self.value_of, self.map.schema_addr);
                        let current_value = current_item.get_value(memory);
                        let next_value = current_value.get_next_addr() as usize;
                        if next_value == 0 { //nothing left to step
                            return None;
                        } else {
                            let next_value_cursor = NP_Cursor::new(next_value, self.value_of, self.map.schema_addr);
                            let next_value_value = next_value_cursor.get_value(memory);
                            let key = next_value_value.get_key(memory);
                            self.current = Some(Map_Item { buff_addr: next_value, key: key });
                            return Some((key, next_value_cursor))
                        }
                    },
                    None => { // first iteration, get head
                        self.current = Some(head.clone());
                        return Some((head.key, NP_Cursor::new(head.buff_addr, self.value_of, self.map.schema_addr)))
                    }
                }
            },
            None => return None
        }


    }

    #[inline(always)]
    pub fn insert<M: NP_Memory>(map_cursor: &NP_Cursor, memory: &M, key: &str) -> Result<NP_Cursor, NP_Error> {

        let value_of = match memory.get_schema(map_cursor.schema_addr) {
            NP_Parsed_Schema::Map { value, .. } => *value,
            _ => 0
        };

        if key.len() >= 255 {
            return Err(NP_Error::new("Key length cannot be larger than 255 charecters!"));
        }

        let map_value = map_cursor.get_value(memory);

        let new_cursor_addr = memory.malloc_borrow(&[0u8; 6])?;
        let new_cursor = NP_Cursor::new(new_cursor_addr, value_of, map_cursor.schema_addr);
        let new_cursor_value = new_cursor.get_value(memory);

        // set key
        let key_item_addr = memory.malloc_borrow(&[key.len() as u8])?;
        memory.malloc_borrow(key.as_bytes())?;
        new_cursor_value.set_key_addr(key_item_addr as u16);

        let head = map_value.get_addr_value() as usize;

        // Set head of map to new cursor
        map_value.set_addr_value(new_cursor_addr as u16);

        if head != 0 { // set new cursors NEXT to old HEAD
            new_cursor_value.set_next_addr(head as u16);
        }

        Ok(new_cursor)
    }

}

impl<'value> NP_Value<'value> for NP_Map<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("map", NP_TypeKeys::Map) }
    
    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let value_of = match schema[address] {
            NP_Parsed_Schema::Map { value, .. } => { value },
            _ => 0
        };

        schema_json.insert("value".to_owned(), NP_Schema::_type_to_json(schema, value_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        let mut map_iter = Self::new_iter(&cursor, memory);

        while let Some((_index, item)) = Self::step_iter(&mut map_iter, memory) {
            let key_size = item.get_value(memory).get_key_size(memory);
            acc_size += 1; // length byte
            acc_size += key_size;
            acc_size += NP_Cursor::calc_size(&item, memory)?;
        }


        Ok(acc_size)
   
    }

    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            return NP_JSON::Null
        }

        let mut json_map = JSMAP::new();

        let mut map_iter = Self::new_iter(&cursor, memory);

        while let Some((key, item)) = Self::step_iter(&mut map_iter, memory) {
            json_map.insert(String::from(key), NP_Cursor::json_encode(&item, memory));     
        }

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact<M: NP_Memory, M2: NP_Memory>(from_cursor: NP_Cursor, from_memory: &'value M, to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        let mut map_iter = Self::new_iter(&from_cursor, from_memory);

        while let Some((key, item)) = Self::step_iter(&mut map_iter, from_memory) {
            let new_item = Self::insert(&to_cursor, to_memory, key)?;
            NP_Cursor::compact(item.clone(), from_memory, new_item, to_memory)?;    
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

    fn default_value(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
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

