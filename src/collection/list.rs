use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;


pub struct NoProtoList<'a> {
    address: u32, // pointer location
    head: u32,
    tail: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    of: &'a NoProtoSchema
}

impl<'a> NoProtoList<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, tail:u32,  memory: Rc<RefCell<NoProtoMemory>>, of: &'a NoProtoSchema) -> Self {
        NoProtoList {
            address,
            head,
            tail,
            memory,
            of
        }
    }

    pub fn select(&mut self, index: u16) -> std::result::Result<NoProtoPointer, NoProtoError> {

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
            
            // provide 
            return NoProtoPointer::new_list_item_ptr(self.head, &self.of, Rc::clone(&self.memory));
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0;

            let mut do_continue = true;

            let mut is_head = true;

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
                    return NoProtoPointer::new_list_item_ptr(curr_addr as u32, &self.of, Rc::clone(&self.memory));
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

                    if is_head {
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
                    return NoProtoPointer::new_list_item_ptr(new_addr as u32, &self.of, Rc::clone(&self.memory));
                } else {
                    // not found yet, get next address
                    let mut next: [u8; 4] = [0; 4];
                    {
                        let memory = self.memory.try_borrow()?;
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

                is_head = false;
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
            return Some(NoProtoPointer::new_table_item_ptr(addr, some_column_schema, Rc::clone(&self.memory)));
            */
        }
        Err(NoProtoError::new(""))
    }

    pub fn delete(&self, index: u16) -> bool {
        false
    }

    pub fn clear(&self) {

    }

    pub fn has(&self, column: &str) {

    }

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
    }
}