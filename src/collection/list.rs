use crate::{error::NP_Error, json_flex::{JSMAP, NP_JSON}, memory::{NP_Memory, NP_Size, blank_ptr_u16_list_item, blank_ptr_u32_list_item, blank_ptr_u8_list_item}, pointer::{NP_Value}, pointer::{NP_Cursor, NP_Cursor_Addr, NP_Cursor_Kinds}, schema::NP_Parsed_Schema, schema::{NP_Schema, NP_TypeKeys}};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{vec::*};
use core::{hint::unreachable_unchecked};

use super::NP_Collection;
/// List data type.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_List<'list> { 
    cursor: NP_Cursor_Addr,
    current: Option<NP_Cursor_Addr>,
    pub memory: NP_Memory<'list>
}

impl<'list> NP_List<'list> {

    /// Accepts a cursor that is currently on a list type and moves the cursor to a list item
    /// The list item may be virtual
    pub fn select_to_ptr(cursor_addr: NP_Cursor_Addr, memory: &'list NP_Memory<'list>, index: u16) -> Result<NP_Cursor_Addr, NP_Error> {

        Self::commit_or_cache_list(&cursor_addr, memory)?;

        // working cursor is list
        let working_cursor = cursor_addr.get_data(&memory)?;

        let (head, tail, parent_addr) = {(working_cursor.coll_head.unwrap(), working_cursor.coll_tail.unwrap(), cursor_addr.address)};

        let list_of = match &**working_cursor.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        // check if tail and head are zero, if so return virtual pointer with requested index
        // list is empty so providing any index is safe
        if head == 0 {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::None;
            virtual_cursor.item_index = Some(index as usize);
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
        }

        
        NP_List::cache_list_item(&NP_Cursor_Addr { address: head, is_virtual: false}, list_of, cursor_addr.address, memory)?;

        // working cursor is now head
        let working_cursor = cursor_addr.get_data(&memory)?;

        let offset = memory.addr_size_bytes();

        // if head/first pointer matches requested index, return the head pointer
        if working_cursor.item_index.unwrap() == index as usize {

            return Ok(NP_Cursor_Addr { address: head, is_virtual: false});
        }

        // if requesting index in front of head, return virtual pointer
        if working_cursor.item_index.unwrap() > index as usize {

            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(parent_addr);
            virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
            virtual_cursor.item_index = Some(index as usize);
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = Some(head);
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
        }
        
        NP_List::cache_list_item(&NP_Cursor_Addr { address: tail, is_virtual: false}, list_of, parent_addr, memory)?;
        
        // working cursor is now tail
        let working_cursor = cursor_addr.get_data(&memory)?;

        // if requesting index higher than tail index, return virtual pointer
        if working_cursor.item_index.unwrap() < index as usize {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(parent_addr);
            virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
            virtual_cursor.item_index = Some(index as usize);
            virtual_cursor.item_prev_addr = Some(tail);
            virtual_cursor.item_next_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true});
        }

        // index is somewhere in the existing records, loop time!
        for item in Self::start_iter(&cursor_addr, memory.clone())? {
            let item_index = memory.get_cursor_data(&item)?.item_index.unwrap();
            if item_index == index as usize {
                return Ok(item.clone())
            }
        }

        // should never reach this
        panic!()
    }

    pub fn cache_list_item(cursor_addr: &'list NP_Cursor_Addr, schema: &'list Box<NP_Parsed_Schema>, parent: usize, memory: &'list NP_Memory<'list>) -> Result<(), NP_Error> {
        
        // should never attempt to cache a virtual cursor
        if cursor_addr.is_virtual { panic!() }

        // real list item in buffer, (maybe) needs to be cached
        match cursor_addr.get_data(&memory) {
            Ok(_x) => { /* already in cache */ },
            Err(_e) => {
                let mut new_cursor = NP_Cursor::new(cursor_addr.address, Some(parent), memory, schema);

                let pointer_index  = match &memory.size {
                    NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(cursor_addr.address + 8).unwrap_or(&[0; 2])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(cursor_addr.address + 4).unwrap_or(&[0; 2])),
                    NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(cursor_addr.address + 2).unwrap_or(0)]) as u16
                };

                let addr_size = memory.addr_size_bytes();

                new_cursor.item_index = Some(pointer_index as usize);
                new_cursor.item_next_addr = Some(memory.read_address(new_cursor.address + addr_size));
                new_cursor.kind = NP_Cursor_Kinds::ListItem;

                memory.insert_cache(new_cursor);                
            }
        }
        

        Ok(())
    }

    /// Get details of list object at this location in buffer
    pub fn commit_or_cache_list(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<(), NP_Error> {

        let cursor = cursor_addr.get_data(&memory)?;

        if cursor_addr.is_virtual { // virtual cursor, just return blank details
            cursor.coll_head = Some(0);
            cursor.coll_tail = Some(0);
            cursor.address_value = 0;
            cursor.kind = NP_Cursor_Kinds::None;
        } else if cursor.address_value == 0 { // real cursor but need to make list

            cursor.coll_head = Some(0);
            cursor.coll_tail = Some(0);

            match memory.size {
                NP_Size::U8 => {
                    cursor.address_value = memory.malloc_borrow(&[0u8; 2])?; // stores HEAD & TAIL for list
                },
                NP_Size::U16 => {  
                    cursor.address_value = memory.malloc_borrow(&[0u8; 4])?; // stores HEAD & TAIL for list
                },
                NP_Size::U32 => {
                    cursor.address_value = memory.malloc_borrow(&[0u8; 8])?; // stores HEAD & TAIL for list
                }
            };

            // write new address to buffer
            memory.set_value_address(cursor.address, cursor.address_value);

            // upgrade cursor cache
            cursor.kind = NP_Cursor_Kinds::List;
        } else if cursor.kind == NP_Cursor_Kinds::Standard { // real cursor with value, need to cache values
            
            match memory.size {
                NP_Size::U8 => {

                    let head: [u8; 1] = [memory.get_1_byte(cursor.address_value).unwrap_or(0)];
                    let tail: [u8; 1] = [memory.get_1_byte(cursor.address_value + 1).unwrap_or(0)];
                    cursor.coll_head = Some(u8::from_be_bytes(head) as usize);
                    cursor.coll_tail = Some(u8::from_be_bytes(tail) as usize);
    
                },
                NP_Size::U16 => {
  
                    let head = *memory.get_2_bytes(cursor.address_value).unwrap_or(&[0; 2]);
                    let tail = *memory.get_2_bytes(cursor.address_value + 2).unwrap_or(&[0; 2]);
                    cursor.coll_head = Some(u16::from_be_bytes(head) as usize);
                    cursor.coll_tail = Some(u16::from_be_bytes(tail) as usize);
    
                },
                NP_Size::U32 => {

                    let head = *memory.get_4_bytes(cursor.address_value).unwrap_or(&[0; 4]);
                    let tail = *memory.get_4_bytes(cursor.address_value + 4).unwrap_or(&[0; 4]);
                    cursor.coll_head = Some(u32::from_be_bytes(head) as usize);
                    cursor.coll_tail = Some(u32::from_be_bytes(tail) as usize);
                }
            };

            // upgrade cursor cache
            cursor.kind = NP_Cursor_Kinds::List;
        }

        Ok(())
    }

    /// Push a new value onto the back of the list
    pub fn push(cursor_addr: NP_Cursor_Addr, memory: &'list NP_Memory<'list>, index: Option<u16>) -> Result<NP_Cursor_Addr, NP_Error> {

        NP_List::commit_or_cache_list(&cursor_addr, memory)?;

        let list_cursor = cursor_addr.get_data(&memory)?;

        let list_of = match &**list_cursor.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        if list_cursor.coll_tail.unwrap() == 0 { // no values in list, return new virtual pointer at index 0

            let use_index = if let Some(x) = index { x } else { 0 };
            
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
            virtual_cursor.item_index = Some(use_index as usize);
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})

        } else { // get tail information and return virtual pointer behind tail
 
            let tail_addr = list_cursor.coll_tail.unwrap();
            let tail_cursor = NP_Cursor_Addr { address: tail_addr, is_virtual: false};

            NP_List::cache_list_item(&tail_cursor, list_of, cursor_addr.address, memory)?;

            let tail_cursor = memory.get_cursor_data(&tail_cursor)?;

            if (tail_cursor.item_index.unwrap() + 1) > core::u16::MAX as usize {
                return Err(NP_Error::new("Error pushing list, out of space!"));
            }

            // auto generate new index from the tail pointer
            let mut new_index = tail_cursor.item_index.unwrap() + 1;

            // if we were given an index to assign to this pointer, make sure it's higher than the generated one
            if let Some(x ) = index {
                if x < new_index as u16 {
                    return Err(NP_Error::new(String::from("Requested index is lower than last item, can't push!")));
                } else {
                    new_index = x as usize;
                }
            }

            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
            virtual_cursor.item_index = Some(new_index as usize);
            virtual_cursor.item_prev_addr = Some(tail_addr);
            virtual_cursor.item_next_addr = None;
            return Ok(NP_Cursor_Addr { address: 0, is_virtual: true})
        }
    }
}

impl<'collection> NP_Collection<'collection> for NP_List<'collection> {

    fn start_iter(list_cursor_addr: &NP_Cursor_Addr, memory: NP_Memory) -> Result<Self, NP_Error> {

        Self::commit_or_cache_list(&list_cursor_addr, &memory)?;

        let list_cursor = memory.get_cursor_data(&list_cursor_addr)?;

        let (head, tail) = {(list_cursor.coll_head.unwrap(), list_cursor.coll_tail.unwrap())};

        let list_of = match &**list_cursor.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };
    
        // Check if the first pointer in the list is at index 0
        // If it is, we can use the first real pointer in our loop
        // If it isn't, we need to make a virtual pointer
        Ok(if head != 0 { // list has items

            let head_cursor_addr = NP_Cursor_Addr { address: head, is_virtual: false};

            Self::cache_list_item(&head_cursor_addr, list_of, list_cursor_addr.address, &memory)?;
            let head_cursor = memory.get_cursor_data(&head_cursor_addr)?;

            if head_cursor.item_index.unwrap() == 0 { // head item is at index 0, we can return the head pointer

                Self {
                    cursor: list_cursor_addr.clone(),
                    current: Some(NP_Cursor_Addr { address: head, is_virtual: false}),
                    memory: memory
                }
            } else { // head is not zero index, need to return virtual pointer in front of head
                let virtual_cursor = memory.get_virt_cursor();
                virtual_cursor.address = 0;
                virtual_cursor.address_value = 0;
                virtual_cursor.schema = list_of;
                virtual_cursor.parent_addr = Some(list_cursor_addr.address);
                virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
                virtual_cursor.item_index = Some(0);
                virtual_cursor.item_prev_addr = None;
                virtual_cursor.item_next_addr = Some(head);
                Self {
                    cursor: list_cursor_addr.clone(),
                    current: Some(NP_Cursor_Addr { address: 0, is_virtual: true}),
                    memory: memory
                }
            }
        } else { // empty list, everything is virtual
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = list_of;
            virtual_cursor.parent_addr = Some(list_cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
            virtual_cursor.item_index = Some(0);
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = None;
            Self {
                cursor: list_cursor_addr.clone(),
                current: Some(NP_Cursor_Addr { address: 0, is_virtual: true}),
                memory: memory
            }
        })
    }

    fn step_pointer(&self, cursor_addr: &NP_Cursor_Addr) -> Option<NP_Cursor_Addr> {

        NP_List::cache_list_item(&cursor_addr, self.schema, self.cursor.address, &self.memory).unwrap();
    
        let list_item_cursor = self.cursor_addr.get_data(&memory).unwrap();

        let next_index = list_item_cursor.item_index.unwrap() + 1;

        let list_cursor_addr = list_item_cursor.parent_addr.unwrap();

        if list_cursor_addr == 0 { return None };

        let list_of = match &**self.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };
    

        let next_item_addr = list_item_cursor.item_next_addr.unwrap();

        // No more real items left in list, stop
        if 0 == next_item_addr { return None; }

        let offset = self.memory.addr_size_bytes();

        let next_item_cursor_addr = NP_Cursor_Addr { address: next_item_addr, is_virtual: false};

        NP_List::cache_list_item(&next_item_cursor_addr, list_of, list_item_cursor.parent_addr.unwrap(), &self.memory.clone()).unwrap();

        let next_pointer = self.memory.get_cursor_data(&next_item_cursor_addr).unwrap();

        if cursor_addr.is_virtual == true { // handle vritual pointer

            if next_index == next_pointer.item_index.unwrap() { // step into real pointer

                next_pointer.item_prev_addr = Some(cursor_addr.address);

                return Some(NP_Cursor_Addr { address: next_item_addr, is_virtual: false})
            } else { // go to next index with virtual pointer
                list_item_cursor.item_index = Some(next_index);
                return Some(cursor_addr.clone());
            }

        } else { // handle real pointer

            if next_index == next_pointer.item_index.unwrap() { // step into another real pointer
                next_pointer.item_prev_addr = Some(cursor_addr.address);
                return Some(NP_Cursor_Addr { address: 0, is_virtual: false})
            } else { // step into virtual pointer
                let virtual_cursor = self.memory.get_virt_cursor();
                virtual_cursor.address = 0;
                virtual_cursor.address_value = 0;
                virtual_cursor.schema = list_of;
                virtual_cursor.parent_addr = Some(list_item_cursor.parent_addr.unwrap());
                virtual_cursor.kind = NP_Cursor_Kinds::ListItem;
                virtual_cursor.item_index = Some(next_index);
                virtual_cursor.item_prev_addr = Some(cursor_addr.address);
                virtual_cursor.item_next_addr = Some(next_item_addr);
                return Some(NP_Cursor_Addr { address: 0, is_virtual: true})
            }
        }

    }

    fn commit_pointer<'mem>(cursor_addr: &NP_Cursor_Addr, memory: NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> {
        
        // already committed
        if cursor_addr.address != 0 {
            return Ok(cursor_addr.clone())
        }

        if cursor_addr.is_virtual == false { panic!() }

        let cursor = memory.get_virt_cursor();

        let addr_size = memory.addr_size_bytes();
        let index_offset = addr_size * 2;

        cursor.address = match &memory.size {
            NP_Size::U8 => {
                let mut blank_ptr = blank_ptr_u8_list_item();
                blank_ptr[index_offset] = cursor.item_index.unwrap() as u8;
                memory.malloc_borrow(&blank_ptr)?
            },
            NP_Size::U16 => {
                let mut blank_ptr = blank_ptr_u16_list_item();
                for (i, x) in (cursor.item_index.unwrap() as u16).to_be_bytes().iter().enumerate() {
                    blank_ptr[index_offset + i] = *x;
                }
                memory.malloc_borrow(&blank_ptr)?
            },
            NP_Size::U32 => {
                let mut blank_ptr = blank_ptr_u32_list_item();
                for (i, x) in (cursor.item_index.unwrap() as u16).to_be_bytes().iter().enumerate() {
                    blank_ptr[index_offset + i] = *x;
                }
                memory.malloc_borrow(&blank_ptr)?
            }
        };


        let list_cursor_addr = cursor.parent_addr.unwrap();

        let list_cursor = memory.get_cursor_data(&NP_Cursor_Addr { address: list_cursor_addr, is_virtual: false}).unwrap();

        let new_addr = cursor.address;
        let prev_addr = cursor.item_prev_addr.unwrap_or(0);
        let next_addr = cursor.item_next_addr.unwrap_or(0);

        
        if prev_addr > 0 {
            // update previous pointer so that it points to this new pointer
            memory.set_value_address(prev_addr + addr_size, new_addr);
            // update cache
            memory.get_cursor_data(&NP_Cursor_Addr { address: prev_addr, is_virtual: false}).unwrap().item_next_addr = Some(new_addr);
        } else { 
            // update head
            memory.set_value_address(list_cursor.address_value, new_addr);
            // update cache
            list_cursor.coll_head = Some(new_addr);
        }

        // adjust this pointer to point to next pointer
        if next_addr > 0 {
            // update buffer
            memory.set_value_address(new_addr + addr_size, next_addr);
            // update cache
            cursor.item_next_addr = Some(next_addr);
        } else { // update tail
            // update tail
            memory.set_value_address(list_cursor.address_value + addr_size, new_addr);
            // update cache
            list_cursor.coll_tail = Some(new_addr);
        }

        memory.insert_cache(cursor.clone());

        Ok(NP_Cursor_Addr { address: cursor.address, is_virtual: false})
     
    }
}

impl<'value> NP_Value<'value> for NP_List<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("list", NP_TypeKeys::List) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));


        let list_of = match &schema[address] {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                *of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("of".to_owned(), NP_Schema::_type_to_json(schema, list_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .set()! Use .into() instead."))
    }

    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<usize, NP_Error> {
        if cursor_addr.is_virtual {
            return Ok(0);     
        }

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return Ok(0);
        }

        // head + tail;,
        let base_size = match memory.size {
            NP_Size::U32 => 8usize,
            NP_Size::U16 => 4usize,
            NP_Size::U8 => 2usize
        };

        let mut acc_size = 0usize;
 
        for l in NP_List::start_iter(&cursor_addr, memory.clone())? {
            acc_size += NP_Cursor::calc_size(l, memory).unwrap();
        }


        Ok(acc_size + base_size)
    }
    
    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        if cursor_addr.is_virtual { return NP_JSON::Null };

        NP_List::commit_or_cache_list(&cursor_addr, memory).unwrap();

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return NP_JSON::Null;
        }

        let mut json_list = Vec::new();

        for l in Self::start_iter(&cursor_addr, memory.clone()).unwrap() {
            json_list.push(NP_Cursor::json_encode(l, memory));      
        }

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_cursor_addr: NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor_addr: NP_Cursor_Addr, to_memory: &'value NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: NP_Value<'value> {

        if from_cursor_addr.address == 0 {
            return Ok(to_cursor_addr);
        }

        NP_List::commit_or_cache_list(&from_cursor_addr, from_memory).unwrap();
        NP_List::commit_or_cache_list(&to_cursor_addr, to_memory).unwrap();

        let from_cursor = from_memory.get_cursor_data(&from_cursor_addr)?;

        let list_of = match &**from_cursor.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        for old_item in Self::start_iter(&from_cursor_addr, from_memory.clone())? {
            if old_item.address != 0 { // pointer is not virutal
                NP_List::cache_list_item(&old_item, list_of, from_cursor.address, from_memory)?;
                let old_cursor = from_memory.get_cursor_data(&old_item)?;
                if old_cursor.address_value != 0 { // pointer has value
                    let index = old_cursor.item_index.unwrap();
                    let mut new_item= NP_List::push(to_cursor_addr, to_memory, Some(index as u16))?;
                    NP_List::commit_pointer(&new_item, to_memory.clone())?;
                    NP_Cursor::compact(old_item, from_memory, new_item, to_memory)?;
                }
            } 
        }

        Ok(to_cursor_addr)
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "list" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::List as u8);

            match json_schema["of"] {
                NP_JSON::Null => {
                    return Err(NP_Error::new("Lists require an 'of' property that is a schema type!"))
                },
                _ => { }
            }

            let of_addr = schema.len();
            let (child_bytes, schema) = NP_Schema::from_json(schema, Box::new(json_schema["of"].clone()))?;
            
            schema_data.extend(child_bytes);
            schema.push(NP_Parsed_Schema::List {
                i: NP_TypeKeys::List,
                of: of_addr,
                sortable: false
            });
            return Ok(Some((schema_data, schema)))
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<&'value Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        
        let of_addr = schema.len();
        let schema = NP_Schema::from_bytes(schema, address + 1, bytes);
        
        schema.push(NP_Parsed_Schema::List {
            i: NP_TypeKeys::List,
            sortable: false,
            of: of_addr
        }); 
        schema
    }
}

impl<'it> Iterator for NP_List<'it> {
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
        
        NP_List::commit_or_cache_list(&self.cursor, &self.memory).unwrap();

        match self.memory.get_cursor_data(&self.cursor) {
            Ok(list_cursor) => {

                let tail = list_cursor.coll_tail.unwrap();

                if tail == 0 { return 0usize }

                let list_of = match &**list_cursor.schema {
                    NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                        of
                    },
                    _ => { unsafe { unreachable_unchecked() } }
                };

                let tail_cursor = NP_Cursor_Addr { address: tail, is_virtual: false};

                NP_List::cache_list_item(&tail_cursor, list_of, list_cursor.address, &self.memory).unwrap();

                let tail_cursor = self.memory.get_cursor_data(&tail_cursor).unwrap();

                let tail_index = tail_cursor.item_index.unwrap();

                tail_index + 1
            },
            Err(_e) => 0usize
        }

    }
}



#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"list\",\"of\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"list\",\"of\":{\"type\":\"string\"}}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&["10"], String::from("hello, world"))?;
    assert_eq!(buffer.get::<String>(&["10"])?, Some(Box::new(String::from("hello, world"))));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 28usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}