use core::ops::Add;
use crate::{error::NP_Error, json_flex::{JSMAP, NP_JSON}, memory::{NP_Size, NP_Memory}, pointer::{NP_Value, NP_Ptr, NP_PtrKinds}, pointer::{NP_Iterator_Helper, NP_Ptr_Collection}, schema::NP_Parsed_Schema, schema::{NP_Schema, NP_TypeKeys}};

use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::{vec::*};
use core::{hint::unreachable_unchecked};

use super::NP_Collection;
/// List data type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug, Clone)]
pub struct NP_List<'list> {
    address: usize,
    head: usize,
    tail: usize,
    memory: &'list NP_Memory,
    schema: &'list Box<NP_Parsed_Schema>
}

impl<'list> NP_List<'list> {

    #[doc(hidden)]
    pub fn new(address: usize, head: usize, tail:usize,  memory: &'list NP_Memory, schema: &'list Box<NP_Parsed_Schema>) -> Self {

        NP_List {
            address,
            head,
            tail,
            memory: memory,
            schema: schema
        }
    }

    /// read schema of list
    pub fn get_schema(&self) -> &'list Box<NP_Parsed_Schema> {
        self.schema
    }

    /// Select while moving self
    /// 
    pub fn select_mv(self, index: u16) -> NP_Ptr<'list> {
        let list_of = match &**self.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        // check if tail and head are zero, if so return virtual pointer with requested index
        // list is empty so providing any index is safe
        if self.head == 0 {
            return NP_Ptr::_new_collection_item_ptr(0, list_of, &self.memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: index,
                prev_addr: 0,
                next_addr: 0,
                next_index: 0
            });
        }

        let memory = self.memory;

        
        let first_pointer_index  = match &memory.size {
            NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(self.head + 8).unwrap_or(&[0; 2])),
            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(self.head + 4).unwrap_or(&[0; 2])),
            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(self.head + 2).unwrap_or(0)]) as u16
        };

        let offset = memory.addr_size_bytes();

        // if first pointer matches requested index, return the head pointer
        if first_pointer_index == index {

            // read into the pointer after the head pointer
            let next_pointer_addr = memory.read_address(self.head + offset);

            // get the index of the next pointer (after this one)
            let next_real_index = if next_pointer_addr == 0 { 0 } else { match &memory.size {
                NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(next_pointer_addr + 8).unwrap_or(&[0; 2])),
                NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(next_pointer_addr + 4).unwrap_or(&[0; 2])),
                NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(next_pointer_addr + 2).unwrap_or(0)]) as u16
            }};

            return NP_Ptr::_new_collection_item_ptr(self.head, list_of, &self.memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: index,
                prev_addr: 0,
                next_addr: next_pointer_addr,
                next_index: next_real_index
            });
        }

        // if requesting index in front of head, return virtual pointer
        if first_pointer_index > index {
            return NP_Ptr::_new_collection_item_ptr(0, list_of, &self.memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: index,
                prev_addr: 0,
                next_addr: self.head,
                next_index: first_pointer_index
            });
        }

        
        let last_pointer_index  = match &memory.size {
            NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(self.tail + 8).unwrap_or(&[0; 2])),
            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(self.tail + 4).unwrap_or(&[0; 2])),
            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(self.tail + 2).unwrap_or(0)]) as u16
        };

        // if requesting index higher than tail index, return virtual pointer
        if last_pointer_index < index {
            return NP_Ptr::_new_collection_item_ptr(0, list_of, &self.memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: index,
                prev_addr: self.tail,
                next_addr: 0,
                next_index: 0
            });
        }

        // index is somewhere in the existing records, loop time!
        for item in self.clone().it().into_iter() {
            let item_index = match item.helper { NP_Iterator_Helper::List { index, prev_addr: _, next_index: _, next_addr: _ } => index, _ => panic!() };
            if item_index == index {
                return item.clone()
            }
        }

        // should never reach this
        panic!()
    }

    /// Select a value from the list.  If the value doesn't exist you'll get a virtual pointer back
    pub fn select(&'list self, index: u16) -> NP_Ptr<'list> {
        NP_List::select_mv(self.clone(), index)
    }

    /// Convert this list into an iterator object
    pub fn it(self) -> NP_List_Iterator<'list> {
        NP_List_Iterator::new(self)
    }

    /// Push a new value onto the back of the list
    pub fn push(&mut self, index: Option<u16>) -> Result<NP_Ptr<'list>, NP_Error> {

        let memory = self.memory;

        let list_of = match &**self.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        

        if self.tail == 0 { // no values in list, return new virtual pointer at index 0

            let use_index = if let Some(x) = index { x } else { 0 };
            
            return Ok(NP_Ptr::_new_collection_item_ptr(0, list_of, &memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: use_index,
                prev_addr: 0,
                next_addr: 0,
                next_index: 0
            }));

        } else { // get tail information and return virtual pointer behind tail
 
            let tail_addr = self.tail;

            let tail_index_bytes = match &memory.size {
                NP_Size::U32 => *memory.get_2_bytes((tail_addr + 8) as usize).unwrap_or(&[0; 2]),
                NP_Size::U16 => *memory.get_2_bytes((tail_addr + 4) as usize).unwrap_or(&[0; 2]),
                NP_Size::U8 => [0, memory.get_1_byte((tail_addr + 2) as usize).unwrap_or(0)],
            };

            if (u16::from_be_bytes(tail_index_bytes) + 1) > core::u16::MAX {
                return Err(NP_Error::new("Error pushing list, out of space!"));
            }

            // auto generate new index from the tail pointer
            let mut new_index = u16::from_be_bytes(tail_index_bytes) + 1;

            // if we were given an index to assign to this pointer, make sure it's higher than the generated one
            if let Some(x) = index {
                if x < new_index {
                    return Err(NP_Error::new(String::from("Requested index is lower than last item, can't push!")));
                } else {
                    new_index = x;
                }
            }

            return Ok(NP_Ptr::_new_collection_item_ptr(0, list_of, &memory, NP_Ptr_Collection::List {
                address: self.address,
                head: self.head,
                tail: self.tail
            }, NP_Iterator_Helper::List {
                index: new_index,
                prev_addr: self.tail,
                next_addr: 0,
                next_index: 0
            }));
        }
    }

    /// Check to see if a value exists in the list.
    pub fn has(&'list self, index: u16) -> bool {
        if self.head == 0 { // empty list, nothing to delete
            false
        } else {
            self.select(index).has_value()
        }
    }

    fn set_head(memory: &NP_Memory, address: usize, head: usize) -> Result<(), NP_Error> {
        memory.write_address(address, head)
    }

    fn set_tail(memory: &NP_Memory, address: usize, tail: usize) -> Result<(), NP_Error> {
        let offset = memory.addr_size_bytes();
        memory.write_address(address + offset, tail)
    }
}

impl<'collection> NP_Collection<'collection> for NP_List<'collection> {

    fn length(&self) -> usize {
        if self.tail == 0 { return 0usize; }

        let memory = self.memory;

        let tail_index = match &memory.size {
            NP_Size::U8 => [0, memory.get_1_byte((self.tail + 2) as usize).unwrap_or(0)],
            NP_Size::U16 => *memory.get_2_bytes((self.tail + 4) as usize).unwrap_or(&[0; 2]),
            NP_Size::U32 => *memory.get_2_bytes((self.tail + 8) as usize).unwrap_or(&[0; 2])
        };

        u16::from_be_bytes(tail_index) as usize
    }

    fn step_pointer(ptr: &mut NP_Ptr<'collection>) -> Option<NP_Ptr<'collection>> {

        match ptr.helper { 
            NP_Iterator_Helper::List { index, prev_addr, next_addr: next_real_addr, next_index: next_real_index} => {
                        
                // No more real items left in list, stop
                if 0 == next_real_addr { return None; }
                if 0 == next_real_index { return None; }

                let head = match ptr.parent {
                    NP_Ptr_Collection::List { address: _, head, tail: _} => head,
                    _ => { unsafe { unreachable_unchecked() } }
                };

                // list has no items in it, nothing to do
                if head == 0 { return None };


                let memory = ptr.memory;

                if ptr.address == 0 { // handle vritual pointer

                    if index + 1 == next_real_index { // step into real pointer

                        let offset = memory.addr_size_bytes();

                        let this_ptr_addr = next_real_addr;

                        // read into the next pointer (after the one we're on)
                        let next_real_addr = memory.read_address(this_ptr_addr + offset);

                        // get the index of the next pointer (after this one)
                        let next_real_index = if next_real_addr == 0 { 0 } else {match &memory.size {
                            NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(next_real_addr + 8).unwrap_or(&[0; 2])),
                            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(next_real_addr + 4).unwrap_or(&[0; 2])),
                            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(next_real_addr + 2).unwrap_or(0)]) as u16
                        }};

                        return Some(NP_Ptr::_new_collection_item_ptr(this_ptr_addr, ptr.schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::List {
                            prev_addr: ptr.address,
                            index: index + 1,
                            next_addr: next_real_addr,
                            next_index: next_real_index
                        }));
                    } else { // go to next index with virtual pointer
                        return Some(NP_Ptr::_new_collection_item_ptr(ptr.address, ptr.schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::List {
                            prev_addr,
                            index: index + 1,
                            next_addr: next_real_addr,
                            next_index: next_real_index
                        }));
                    }

                } else { // handle real pointer

                    let offset = memory.addr_size_bytes();

                    let this_ptr_addr = next_real_addr;

                    if index + 1 == next_real_index { // step into another real pointer

                        // read into the next pointer (after the one we're on)
                        let next_real_addr = memory.read_address(this_ptr_addr + offset);

                        // get the index of the next pointer (after this one)
                        let next_real_index = match &memory.size {
                            NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(next_real_addr + 8).unwrap_or(&[0; 2])),
                            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(next_real_addr + 4).unwrap_or(&[0; 2])),
                            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(next_real_addr + 2).unwrap_or(0)]) as u16
                        };

                        return Some(NP_Ptr::_new_collection_item_ptr(this_ptr_addr, ptr.schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::List {
                            prev_addr: ptr.address,
                            index: index + 1,
                            next_addr: next_real_addr,
                            next_index: next_real_index
                        }));
                    } else { // step into virtual pointer
                        return Some(NP_Ptr::_new_collection_item_ptr(0, ptr.schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::List {
                            prev_addr: ptr.address,
                            index: index + 1,
                            next_addr: next_real_addr,
                            next_index: next_real_index
                        }));
                    }
                }
            }
            _ => panic!()
        };

    }

    fn commit_pointer(ptr: NP_Ptr<'collection>) -> Result<NP_Ptr<'collection>, NP_Error> {
        // already committed
        if ptr.address != 0 {
            return Ok(ptr)
        }

        let (list_addr, head, tail) = match ptr.parent {
            NP_Ptr_Collection::List { address, head, tail} => {
                (address, head, tail)
            },
            _ => panic!()
        };

        match ptr.helper {
            NP_Iterator_Helper::List { index, next_addr, next_index: _, prev_addr} => {

                let mut ptr_bytes: Vec<u8> = ptr.memory.blank_ptr_bytes(&NP_PtrKinds::ListItem { addr: 0, i: 0, next: 0 }); 

                let addr_size = ptr.memory.addr_size_bytes();

                // write index to new pointer memory
                let index_offset = addr_size * 2;
                for (i, x) in index.to_be_bytes().iter().enumerate() {
                    ptr_bytes[index_offset + i] = *x;
                }

                // write new pointer to memory
                let new_addr = ptr.memory.malloc(ptr_bytes)?;

                let mut new_head = head;
                let mut new_tail = tail;

                // update previous pointer so that it points to this new pointer
                if prev_addr > 0 {
                    ptr.memory.write_address(prev_addr + addr_size, new_addr)?;
                } else { // update head
                    new_head = new_addr;
                    NP_List::set_head(ptr.memory, list_addr, new_head)?;
                }

                // adjust this pointer to point to next pointer
                if next_addr > 0 {
                    ptr.memory.write_address(new_addr + addr_size, next_addr)?;
                } else { // update tail
                    new_tail = new_addr;
                    NP_List::set_tail(ptr.memory, list_addr, new_tail)?;
                }

                Ok(NP_Ptr::_new_collection_item_ptr(new_addr, ptr.schema, ptr.memory, NP_Ptr_Collection::List {
                    address: list_addr,
                    head: new_head,
                    tail: new_tail
                }, ptr.helper))
            },
            _ => panic!()
        }
    }
}

impl<'list> NP_Value<'list> for NP_List<'list> {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::List as u8, "list".to_owned(), NP_TypeKeys::List) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::List as u8, "list".to_owned(), NP_TypeKeys::List) }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));


        let list_of = match schema_ptr {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("of".to_owned(), NP_Schema::_type_to_json(&list_of)?);

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_pointer: &mut NP_Ptr, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Type (list) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Ptr<'list>) -> Result<Option<Box<Self>>, NP_Error> {
       
        let mut addr = ptr.kind.get_value_addr(); // get pointer of list (head/tail)

        match &ptr.memory.size {
            NP_Size::U8 => {
                let mut head: [u8; 1] = [0; 1];
                let mut tail: [u8; 1] = [0; 1];

                if addr == 0 {
                    // no list here, make one
                    addr = ptr.memory.malloc([0u8; 2].to_vec())?; // stores HEAD & TAIL for list
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = [ptr.memory.get_1_byte(a).unwrap_or(0)];
                    tail = [ptr.memory.get_1_byte(a + 1).unwrap_or(0)];
                }

                Ok(Some(Box::new(Self::new(addr, u8::from_be_bytes(head) as usize, u8::from_be_bytes(tail) as usize, ptr.memory, ptr.schema))))
            },
            NP_Size::U16 => {
                let mut head: [u8; 2] = [0; 2];
                let mut tail: [u8; 2] = [0; 2];

                if addr == 0 {
                    // no list here, make one
                    addr = ptr.memory.malloc([0u8; 4].to_vec())?; // stores HEAD & TAIL for list
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2]);
                    tail = *ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2]);
                }


                Ok(Some(Box::new(Self::new(addr, u16::from_be_bytes(head) as usize, u16::from_be_bytes(tail) as usize, ptr.memory, ptr.schema))))
            },
            NP_Size::U32 => {
                let mut head: [u8; 4] = [0; 4];
                let mut tail: [u8; 4] = [0; 4];

                if addr == 0 {
                    // no list here, make one
                    addr = ptr.memory.malloc([0u8; 8].to_vec())?; // stores HEAD & TAIL for list
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]);
                    tail = *ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]);
                }

                Ok(Some(Box::new(Self::new(addr, u32::from_be_bytes(head) as usize, u32::from_be_bytes(tail) as usize, ptr.memory, ptr.schema))))
            }
        }
        
    }

    fn get_size(ptr: &'list NP_Ptr<'list>) -> Result<usize, NP_Error> {
        // head + tail;,
        let base_size = match ptr.memory.size {
            NP_Size::U32 => 8usize,
            NP_Size::U16 => 4usize,
            NP_Size::U8 => 2usize
        };

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        // existing head, read value
        let a = addr as usize;
        let head = match &ptr.memory.size {
            NP_Size::U8 => u8::from_be_bytes([ptr.memory.get_1_byte(a).unwrap_or(0)]) as u32,
            NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]))
        } as usize;
        let tail = match &ptr.memory.size {
            NP_Size::U8 => u8::from_be_bytes([ptr.memory.get_1_byte(a + 1).unwrap_or(0)]) as u32,
            NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]))
        } as usize;
    
        let list = Self::new(addr, head, tail, ptr.memory, ptr.schema);

        let mut acc_size = 0usize;

        for l in list.it().into_iter() {
            acc_size += l.calc_size()?;
        }

        Ok(acc_size + base_size)
    }
    
    fn to_json(ptr: &NP_Ptr<'list>) -> NP_JSON {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let a = addr as usize;
        let head = match &ptr.memory.size {
            NP_Size::U8 => u8::from_be_bytes([ptr.memory.get_1_byte(a).unwrap_or(0)]) as u32,
            NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]))
        } as usize;
        let tail = match &ptr.memory.size {
            NP_Size::U8 => u8::from_be_bytes([ptr.memory.get_1_byte(a + 1).unwrap_or(0)]) as u32,
            NP_Size::U16 => u16::from_be_bytes(*ptr.memory.get_2_bytes(a + 2).unwrap_or(&[0; 2])) as u32,
            NP_Size::U32 => u32::from_be_bytes(*ptr.memory.get_4_bytes(a + 4).unwrap_or(&[0; 4]))
        } as usize;

        let list = Self::new(addr, head, tail, ptr.memory, ptr.schema);

        let mut json_list = Vec::new();

        for l in list.it().into_iter() {
            json_list.push(l.json_encode());      
        }

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_ptr: NP_Ptr<'list>, to_ptr: &'list mut NP_Ptr<'list>) -> Result<(), NP_Error> where Self: NP_Value<'list> {

        let old_list = Self::into_value(from_ptr.clone())?.unwrap();
        let mut new_list = Self::into_value(to_ptr.clone())?.unwrap();

        for old_item in old_list.it().into_iter() {
            if old_item.has_value() {
                let this_index = match old_item.helper { NP_Iterator_Helper::List { index, prev_addr: _, next_addr: _, next_index: _} => index, _ => panic!() };

                let mut new_item = new_list.push(Some(this_index))?;
                new_item = NP_List::commit_pointer(new_item)?;

                old_item.clone().compact(&mut new_item)?;
            }
        }

        Ok(())
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

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

            let child_type = NP_Schema::from_json(Box::new(json_schema["of"].clone()))?;
            schema_data.extend(child_type.0);
            return Ok(Some((schema_data, NP_Parsed_Schema::List {
                i: NP_TypeKeys::List,
                of: Box::new(child_type.1),
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


/// The iterator type for lists
#[derive(Debug)]
pub struct NP_List_Iterator<'it> {
    list_schema: &'it Box<NP_Parsed_Schema>,
    current: Option<NP_Ptr<'it>>
}

impl<'it> NP_List_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(list: NP_List<'it>) -> Self {
        let list_of = match &**list.schema {
            NP_Parsed_Schema::List { i: _, sortable: _, of} => {
                of
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let memory = list.memory;

        let addr_size = memory.addr_size_bytes();

        // Check if the first pointer in the list is at index 0
        // If it is, we can use the first real pointer in our loop
        // If it isn't, we need to make a virtual pointer
        let (addr, prev_addr, next_addr, next_index) = if list.head != 0 { // list has items

            let head_index = match &memory.size {
                NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(list.head + 8).unwrap_or(&[0; 2])),
                NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(list.head + 4).unwrap_or(&[0; 2])),
                NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(list.head + 2).unwrap_or(0)]) as u16
            };

            if head_index == 0 { // head item is at index 0, we can return the head pointer

                let pointer_after_head = memory.read_address(list.head + addr_size);
                let index_after_head = if pointer_after_head == 0 { 0 } else { match &memory.size {
                    NP_Size::U32 => u16::from_be_bytes(*memory.get_2_bytes(pointer_after_head + 8).unwrap_or(&[0; 2])),
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(pointer_after_head + 4).unwrap_or(&[0; 2])),
                    NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(pointer_after_head + 2).unwrap_or(0)]) as u16
                }};

                (list.head, 0, pointer_after_head, index_after_head)
            } else { // head is not zero index, need to return virtual pointer in front of head
                (0, 0, list.head, head_index)
            }
        } else { // empty list, everything is virtual
            (0, 0, 0, 0)
        };

        // make first initial pointer
        NP_List_Iterator {
            list_schema: list.schema,
            current: Some(NP_Ptr::_new_collection_item_ptr(addr, list_of, &memory, NP_Ptr_Collection::List {
                address: list.address,
                head: list.head,
                tail: list.tail
            }, NP_Iterator_Helper::List {
                index: 0,
                prev_addr,
                next_addr,
                next_index
            }))
        }
    }
}

impl<'it> Iterator for NP_List_Iterator<'it> {
    type Item = NP_Ptr<'it>;

    fn next(&mut self) -> Option<Self::Item> {

        match &mut self.current {
            Some(x) => {
                let current = x.clone();
                self.current = NP_List::step_pointer(x);
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
    buffer.set("10", String::from("hello, world"))?;
    assert_eq!(buffer.get::<String>("10")?, Some(Box::new(String::from("hello, world"))));
    buffer.del("")?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}