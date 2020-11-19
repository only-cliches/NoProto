use crate::{json_flex::JSMAP, utils::from_utf8_lossy, pointer::{NP_Iterator_Helper, NP_PtrKinds, NP_Ptr_Collection}};
use crate::pointer::{NP_Value, NP_Ptr};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{hint::unreachable_unchecked};
use core::ops::Add;

use super::NP_Collection;

/// The map type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug, Clone)]
pub struct NP_Map<'map> {
    address: usize, // pointer location
    head: usize,
    len: u16,
    memory: &'map NP_Memory,
    schema: &'map Box<NP_Parsed_Schema>,
}

impl<'map> NP_Value<'map> for NP_Map<'map> {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Map as u8, "map".to_owned(), NP_TypeKeys::Map) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Map as u8, "map".to_owned(), NP_TypeKeys::Map) }
    
    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let value_of = match &*schema_ptr {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("value".to_owned(), NP_Schema::_type_to_json(&value_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }
    
    fn set_value(_pointer: &mut NP_Ptr<'map>, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Type (map) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Ptr<'map>) -> Result<Option<Box<Self>>, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let a = addr as usize;

        let head = if addr == 0 {
            0usize
        } else { 
            ptr.memory.read_address(addr)
        };
    
        let size = if addr == 0 { 0u16 } else {
            match &ptr.memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([ptr.memory.get_1_byte(a + 1).unwrap_or(0)]) as u16
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2]))
                },
                NP_Size::U32 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 4).unwrap_or(&[0; 2]))
                }
            }
        };

        if addr == 0 {
            // no map here, make one
            let bytes = match &ptr.memory.size {
                NP_Size::U8  => { [0u8; 2].to_vec() },
                NP_Size::U16 => { [0u8; 4].to_vec() },
                NP_Size::U32 => { [0u8; 6].to_vec() }
            };
            addr = ptr.memory.malloc(bytes)?; // stores HEAD & LENGTH for map
            ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);
        }

        Ok(Some(Box::new(NP_Map::new(addr, head, size, ptr.memory, ptr.schema))))
    

    }

    fn get_size(ptr: &'map NP_Ptr<'map>) -> Result<usize, NP_Error> {

        let base_size = match &ptr.memory.size {
            NP_Size::U8  => { 2usize }, // u8 head | u8 length
            NP_Size::U16 => { 4usize }, // u16 head | u16 length
            NP_Size::U32 => { 6usize }  // u32 head | u16 length
        };

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let a = addr as usize;

        let head = if addr == 0 {
            0usize
        } else { 
            match &ptr.memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([ptr.memory.get_1_byte(a).unwrap_or(0)]) as usize
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as usize
                },
                NP_Size::U32 => {
                    u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4])) as usize
                }
        }};
        let size = if addr == 0 { 0u16 } else {
            match &ptr.memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([ptr.memory.get_1_byte(a + 1).unwrap_or(0)]) as u16
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2]))
                },
                NP_Size::U32 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 4).unwrap_or(&[0; 2]))
                }
            }
        };
        

        let list = Self::new(addr, head, size, &ptr.memory, ptr.schema);

        let mut acc_size = 0usize;

        for ptr in list.it().into_iter() {

            if ptr.has_value() == true {
                acc_size += ptr.calc_size()?;
                let key_addr = match ptr.helper { NP_Iterator_Helper::Map { key: _, key_addr, prev_addr: _} => key_addr, _ => panic!()};
                match &ptr.memory.size {
                    NP_Size::U8 => {
                        acc_size += NP_Map::get_key_size(key_addr, ptr.memory) + 1; // key + key length bytes
                    },
                    NP_Size::U16 => {
                        acc_size += NP_Map::get_key_size(key_addr, ptr.memory) + 2; // key + key length bytes
                    },
                    NP_Size::U32 => {
                        acc_size += NP_Map::get_key_size(key_addr, ptr.memory) + 4; // key + key length bytes
                    }
                }
                
            }

        };

        Ok(acc_size + base_size)
   
    }

    fn to_json(ptr: &'map NP_Ptr<'map>) -> NP_JSON {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let a = addr as usize;

        let head = if addr == 0 {
            0usize
        } else { 
            match &ptr.memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([ptr.memory.get_1_byte(a).unwrap_or(0)]) as usize
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as usize
                },
                NP_Size::U32 => {
                    u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4])) as usize
                }
        }};

        let size = if addr == 0 { 0u16 } else {
            match &ptr.memory.size {
                NP_Size::U8 => {
                    u8::from_be_bytes([ptr.memory.get_1_byte(a + 1).unwrap_or(0)]) as u16
                },
                NP_Size::U16 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2]))
                },
                NP_Size::U32 => {
                    u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 4).unwrap_or(&[0; 2]))
                }
            }
        };

        let map = Self::new(addr, head, size, ptr.memory, ptr.schema);

        let mut json_map = JSMAP::new();

        for item in map.it().into_iter() {

            let key_addr = match item.helper { NP_Iterator_Helper::Map { key: _, key_addr, prev_addr: _} => key_addr, _ => panic!()};

            let key = NP_Map::get_key(key_addr, item.memory);

            json_map.insert(key, item.json_encode());
        }

        NP_JSON::Dictionary(json_map)
   
    }

    fn do_compact(from_ptr: NP_Ptr<'map>, to_ptr: &'map mut NP_Ptr<'map>) -> Result<(), NP_Error> where Self: NP_Value<'map> {

        if from_ptr.address == 0 {
            return Ok(());
        }

        let old_map = Self::into_value(from_ptr)?.unwrap();

        let new_map = Self::into_value(to_ptr.clone())?.unwrap();

        for old_ptr_item in old_map.it().into_iter() {
            if old_ptr_item.has_value() {
                let key_addr = match old_ptr_item.helper { NP_Iterator_Helper::Map { key: _, key_addr, prev_addr: _} => key_addr, _ => panic!()};
                let key = NP_Map::get_key(key_addr, old_ptr_item.memory);
                let mut new_item_ptr = new_map.select(key, true);
                new_item_ptr = NP_Map::commit_pointer(new_item_ptr)?;
                old_ptr_item.clone().compact(&mut new_item_ptr)?;
            }
        }

        Ok(())
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {
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

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
        NP_Parsed_Schema::List {
            i: NP_TypeKeys::List,
            sortable: false,
            of: Box::new(NP_Schema::from_bytes(address + 1, bytes))
        }
    }
}


impl<'map> NP_Map<'map> {

    #[doc(hidden)]
    pub fn new(address: usize, head: usize, length: u16, memory: &'map NP_Memory, schema: &'map Box<NP_Parsed_Schema>) -> Self {
        NP_Map {
            address,
            head,
            memory: memory,
            schema: schema,
            len: length
        }
    }

    /// read schema of map
    pub fn get_schema(&self) -> &'map Box<NP_Parsed_Schema> {
        self.schema
    }

    /// Convert this map into an iterator
    pub fn it(self) -> NP_Map_Iterator<'map> {
        NP_Map_Iterator::new(self)
    }

    /// Get key size
    pub fn get_key_size(key_addr: usize, memory: &NP_Memory) -> usize {
        match memory.size {
            NP_Size::U8 => {
                let mut size_bytes: [u8; 1] = [0; 1];
                size_bytes.copy_from_slice(&memory.read_bytes()[key_addr..(key_addr+1)]);
                u8::from_be_bytes(size_bytes) as usize
            },
            NP_Size::U16 => {
                let mut size_bytes: [u8; 2] = [0; 2];
                size_bytes.copy_from_slice(&memory.read_bytes()[key_addr..(key_addr+2)]);
                u16::from_be_bytes(size_bytes) as usize
            },
            NP_Size::U32 => { 
                let mut size_bytes: [u8; 4] = [0; 4];
                size_bytes.copy_from_slice(&memory.read_bytes()[key_addr..(key_addr+4)]);
                u32::from_be_bytes(size_bytes) as usize
            }
        }
    }

    /// Get key string
    pub fn get_key(key_addr: usize, memory: &NP_Memory) -> String {

        let addr = key_addr;

        let bytes_size = NP_Map::get_key_size(key_addr, memory);

        // get bytes
        let bytes = match memory.size {
            NP_Size::U8 => { &memory.read_bytes()[(addr+1)..(addr+1+bytes_size)] },
            NP_Size::U16 => { &memory.read_bytes()[(addr+2)..(addr+2+bytes_size)] },
            NP_Size::U32 => { &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)] }
        };

        from_utf8_lossy(bytes)
    }   
    
    /// Select key of map
    pub fn select(&'map self, key: String, quick_select: bool) -> NP_Ptr<'map> {
        NP_Map::select_mv(self.clone(), key, quick_select)
    }

    /// Select a specific value at the given key
    pub fn select_mv(self, key: String, quick_select: bool) -> NP_Ptr<'map> {
        let map_of = match &**self.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        // map is empty, return virtual pointer
        if self.head == 0 {
            return NP_Ptr::_new_collection_item_ptr(0, &map_of, &self.memory, NP_Ptr_Collection::Map {
                address: self.address,
                head: self.head,
                length: 0
            }, NP_Iterator_Helper::Map {
                key_addr: 0,
                prev_addr: 0,
                key: Some(key)
            })
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate key
        if quick_select {
            return NP_Ptr::_new_collection_item_ptr(0, &map_of, &self.memory, NP_Ptr_Collection::Map {
                address: self.address,
                head: self.head,
                length: self.len
            }, NP_Iterator_Helper::Map {
                key_addr: 0,
                prev_addr: self.head,
                key: Some(key)
            })
        }

        let mut running_ptr: usize = 0;

        // key might be somewhere in existing records
        for item in self.clone().it().into_iter() {
            match &item.helper { 
                NP_Iterator_Helper::Map { key_addr, prev_addr: _, key: key_opt } => {
                    if let Some(x) = key_opt {
                        // found matched key
                        if *x == key.as_str() {
                            return item.clone();
                        }
                    } else {
                        // found matched key
                        if NP_Map::get_key(*key_addr, item.memory) == key {
                            return item.clone();
                        }
                    }
                }
                _ => panic!()
            };

            running_ptr = item.address;
        }


        // key not found, make a virutal pointer at the end of the map pointers
        return NP_Ptr::_new_collection_item_ptr(0, &map_of, &self.memory, NP_Ptr_Collection::Map {
            address: self.address,
            head: self.head,
            length: self.len
        }, NP_Iterator_Helper::Map {
            key_addr: 0,
            prev_addr: running_ptr,
            key: Some(key)
        })
    }

    /// Check to see if a key exists in this map
    pub fn has(&'map self, key: String) -> bool {
        if self.head == 0 { // no values in this map
           false
        } else {
            self.select(key, false).has_value()
        }
    }

}

impl<'collection> NP_Collection<'collection> for NP_Map<'collection> {

    /// Get length of collection
    fn length(&self) -> usize {
        self.len.into()
    }

    /// Step a pointer to the next item in the collection
    fn step_pointer(ptr: &mut NP_Ptr<'collection>) -> Option<NP_Ptr<'collection>> {
        // can't step with virtual pointer
        if ptr.address == 0 {
            return None;
        }

        let addr_size = ptr.memory.addr_size_bytes();

        // save current pointer as previous pointer for next pointer
        let prev_addr = ptr.address;

        // get address for next pointer
        let curr_addr = ptr.memory.read_address(ptr.address + addr_size);

        if curr_addr == 0 { // no more pointers
            return None;
        }

        // get key address for next pointer
        let key_addr = ptr.memory.read_address(curr_addr + addr_size + addr_size);

        // provide next pointer
        Some(NP_Ptr::_new_collection_item_ptr(curr_addr, ptr.schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::Map {
            key_addr,
            prev_addr,
            key: None
        }))
    }

    /// Commit a virtual pointer into the buffer
    fn commit_pointer(ptr: NP_Ptr<'collection>) -> Result<NP_Ptr<'collection>, NP_Error> {

        // pointer already committed
        if ptr.address != 0 {
            return Ok(ptr);
        }

        match ptr.helper {
            NP_Iterator_Helper::Map { prev_addr, key_addr: _, key} => {
                let (mut head, map_address, mut length) = match ptr.parent { NP_Ptr_Collection::Map { head, address, length } => { (head, address, length)}, _ => panic!()};

                let ptr_bytes: Vec<u8> = ptr.memory.blank_ptr_bytes(&NP_PtrKinds::MapItem { addr: 0, next: 0, key: 0}); 

                let addr_size = ptr.memory.addr_size_bytes();

                let new_addr = ptr.memory.malloc(ptr_bytes)?;

                let key_bytes = key.unwrap().as_bytes().to_vec();

                let key_len_bytes = match ptr.memory.size {
                    NP_Size::U8 => {
                        (key_bytes.len() as u8).to_be_bytes().to_vec()
                    },
                    NP_Size::U16 => {
                        (key_bytes.len() as u16).to_be_bytes().to_vec()
                    },
                    NP_Size::U32 => {
                        (key_bytes.len() as u32).to_be_bytes().to_vec()
                    }
                };

                let key_addr = ptr.memory.malloc(key_len_bytes)?;
                ptr.memory.malloc(key_bytes)?;

                // update pointer to key address
                ptr.memory.write_address(new_addr + addr_size + addr_size, key_addr)?;

                if head == 0 { // empty map
                    // set head to this new pointer
                    head = new_addr;
                    ptr.memory.write_address(map_address, new_addr)?;
                    // set length to one
                    ptr.memory.write_bytes()[map_address + addr_size + 1] = 1;
                } else { // map has existing values

                    // update map length
                    length += 1;
                    for (i, x) in length.to_be_bytes().to_vec().into_iter().enumerate() {
                        ptr.memory.write_bytes()[map_address + addr_size + i] = x;
                    }

                    if prev_addr == head { // inserting in beggining
                        // update this poitner's "next" to old head
                        ptr.memory.write_address(new_addr + addr_size, head)?;
                        // update head to this new pointer
                        head = new_addr;
                        ptr.memory.write_address(map_address, new_addr)?;
                    } else { // inserting at end
                        // update previous pointer "next" to this new pointer
                        ptr.memory.write_address(prev_addr + addr_size, new_addr)?;
                    }
                }

                Ok(NP_Ptr::_new_collection_item_ptr(new_addr, ptr.schema, ptr.memory, NP_Ptr_Collection::Map {
                    address: map_address,
                    head: head,
                    length: length
                }, NP_Iterator_Helper::Map {
                    key_addr,
                    prev_addr,
                    key: None
                }))
            },
            _ => panic!()
        }
    }
}




impl<'it> NP_Map_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(map: NP_Map<'it>) -> Self {
        let map_of = match &**map.schema {
            NP_Parsed_Schema::Map { i: _, sortable: _, value} => {
                value
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let memory = map.memory;

        let addr_size = memory.addr_size_bytes();

        let length = if map.address == 0 { 0 } else {
            u16::from_be_bytes(*memory.get_2_bytes(map.address + addr_size).unwrap())
        };

        // Check if there's a pointer in the map, if so use it as the first element in the loop
        let (addr, prev_addr, key_addr) = if map.head != 0 { // map has items
            let key_addr = memory.read_address(map.head + addr_size + addr_size);
            (map.head, 0, key_addr)
        } else { // empty map, everything is virtual
            (0, 0, 0)
        };

        // make first initial pointer
        NP_Map_Iterator {
            map_schema: map.schema,
            current: Some(NP_Ptr::_new_collection_item_ptr(addr, map_of, &memory, NP_Ptr_Collection::Map {
                address: map.address,
                head: map.head,
                length: length
            }, NP_Iterator_Helper::Map {
                key_addr,
                prev_addr,
                key: None
            }))
        }
    }
}

/// The iterator type for maps
#[derive(Debug)]
pub struct NP_Map_Iterator<'it> {
    map_schema: &'it Box<NP_Parsed_Schema>,
    current: Option<NP_Ptr<'it>>
}

impl<'it> Iterator for NP_Map_Iterator<'it> {
    type Item = NP_Ptr<'it>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.current {
            Some(x) => {
                let current = x.clone();
                self.current = NP_Map::step_pointer(x);
                Some(current)
            },
            None => None
        }
    }

    fn count(self) -> usize where Self: Sized {
        #[inline]
        fn add1<T>(count: usize, _: T) -> usize {
            // Might overflow.
            Add::add(count, 1)
        }

        self.fold(0, add1)
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
    buffer.set("name", String::from("hello, world"))?;
    assert_eq!(buffer.get::<String>("name")?, Some(Box::new(String::from("hello, world"))));
    buffer.del("")?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}