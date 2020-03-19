use crate::pointer::NP_ValueInto;
use crate::{memory::NP_Memory, pointer::{NP_Value, NP_Ptr}, error::NP_Error, schema::NP_Schema};

use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::string::ToString;

pub struct NP_List<'a> {
    address: u32, // pointer location
    head: u32,
    tail: u32,
    memory: Option<&'a NP_Memory>,
    of: Option<&'a NP_Schema>
}

impl<'a> NP_List<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, tail:u32,  memory: &'a NP_Memory, of: &'a NP_Schema) -> Self {
        NP_List {
            address,
            head,
            tail,
            memory: Some(memory),
            of: Some(of)
        }
    }

    /*
    pub fn select<X: NP_Value + Default + NP_ValueInto<'a>>(&'a mut self, index: u16) -> core::result::Result<NP_Ptr<X>, NP_Error> {

        if self.head == 0 { // no values, create one

            let addr;

            {
                let mut memory = self.memory.try_borrow_mut()?;

                let mut ptr_bytes: [u8; 10] = [0; 10];

                // set index in pointer
                let index_bytes = index.to_le_bytes();

                for x in 0..index_bytes.len() {
                    ptr_bytes[x + 8] = index_bytes[x];
                }

                addr = memory.malloc(ptr_bytes.to_vec())?;

            }
            
            // update head to point to newly created ListItem pointer
            self.set_head(addr);
            self.set_tail(addr);
            
            // provide 
            return NP_Ptr::new_list_item_ptr(self.head, &self.of.unwrap(), Rc::clone(&self.memory));
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0;

            let mut do_continue = true;

            while do_continue {

                let ptr_index: u16;

                {
                    let memory = self.memory.borrow();
                    let mut index_bytes: [u8; 2] = [0; 2];
                    index_bytes.copy_from_slice(&memory.bytes[(curr_addr + 8)..(curr_addr + 10)]);
                    ptr_index = u16::from_le_bytes(index_bytes);
                }

                // found our value!
                if ptr_index == index {
                    return NP_Ptr::new_list_item_ptr(curr_addr as u32, &self.of.unwrap(), Rc::clone(&self.memory));
                }

                // we've found an existing value above the requested index
                // insert a new pointer in before the current one in the loop
                if ptr_index > index {

                    let new_addr = {
                        let mut memory = self.memory.try_borrow_mut()?;

                        let mut ptr_bytes: [u8; 10] = [0; 10];

                        // set "next" value of this new pointer to current pointer in the loop
                        let curr_addr_bytes = curr_addr.to_le_bytes();
                        for x in 0..curr_addr_bytes.len() {
                            ptr_bytes[4 + x] = curr_addr_bytes[x]; 
                        }

                        // set index of the new pointer
                        let index_bytes = index.to_le_bytes();
                        for x in 0..index_bytes.len() {
                            ptr_bytes[8 + x] = index_bytes[x]; 
                        }
    
                        memory.malloc(ptr_bytes.to_vec())?
                    };

                    if curr_addr == self.head as usize {
                        // update head to new pointer
                        self.set_head(new_addr);
                    } else {
                        // update "next" value of previous pointer to the one we just made
                        let new_addr_bytes = new_addr.to_le_bytes();
                        let mut memory = self.memory.try_borrow_mut()?;
                        for x in 0..new_addr_bytes.len() {
                            memory.bytes[prev_addr + 4 + x] = new_addr_bytes[x];
                        }
                    }
                    return NP_Ptr::new_list_item_ptr(new_addr as u32, &self.of.unwrap(), Rc::clone(&self.memory));
                } else {
                    // not found yet, get next address
                    let mut next: [u8; 4] = [0; 4];
                    {
                        let memory = self.memory;
                        next.copy_from_slice(&memory.bytes[(curr_addr + 4)..(curr_addr + 8)]);
                    }
                    
                    let next_ptr = u32::from_le_bytes(next) as usize;
                    if next_ptr == 0 { // out of values to check
                        do_continue = false;
                    } else {
                        // store old value for next loop
                        prev_addr = curr_addr;

                        // set next pointer for next loop
                        curr_addr = next_ptr;
                    }
                }
            }

            /*
            // ran out of pointers to check, make one!

            let addr;

            {
                let mut memory = self.memory.borrow_mut();

                let mut ptr_bytes: [u8; 10] = [0; 10];

                // set column index in pointer
                ptr_bytes[8] = *column_index;
        
                addr = memory.malloc(ptr_bytes.to_vec()).unwrap_or(0);
    
                // out of memory
                if addr == 0 { return None; }

                // set previouse pointer's "next" value to this new pointer
                let addr_bytes = addr.to_le_bytes();
                for x in 0..addr_bytes.len() {
                    memory.bytes[(prev_addr + 4 + x)] = addr_bytes[x];
                }

            }
            
            // provide 
            return Some(NP_Ptr::new_table_item_ptr(addr, some_column_schema, Rc::clone(&self.memory)));
            */
        }
        Err(NP_Error::new(""))
    }
*/
    pub fn delete(&self, _index: u16) -> bool {
        false
    }

    pub fn has(&self, _column: &str) {

    }
/*
    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let mut memory = self.memory.borrow_mut();

        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    fn set_tail(&mut self, addr: u32) {

        self.tail = addr;

        let mut memory = self.memory.borrow_mut();

        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32 + 4) as usize] = addr_bytes[x as usize];
        }
    }*/
}

impl<'a> NP_Value for NP_List<'a> {
    fn new<T: NP_Value + Default>() -> Self {
        unreachable!()
    }
    fn is_type( _type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "map".to_owned()) }
    /*fn buffer_get(&self, address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("This type doesn't support .get()!"))
    }
    fn buffer_set(&mut self, address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("This type doesn't support .set()!"))
    }
    fn buffer_into(&self, address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        self.buffer_get(address, kind, schema, buffer)
    }*/
}

impl<'a> Default for NP_List<'a> {

    fn default() -> Self {
        NP_List { address: 0, head: 0, tail: 0, memory: None, of: None}
    }
}