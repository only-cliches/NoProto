use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;


pub struct NoProtoList<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    of: &'a NoProtoSchema
}

impl<'a> NoProtoList<'a> {

    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, of: &'a NoProtoSchema) -> Self {
        NoProtoList {
            address,
            head,
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

            /*let mut prev_addr = self.head as usize;

            let mut do_continue = true;

            while do_continue {

                let index;

                {
                    let memory = self.memory.borrow();
                    index = memory.bytes[(prev_addr + 8)];
                }

                // found our value!
                if index == *column_index {
                    return Some(NoProtoPointer::new_table_item_ptr(prev_addr as u32, some_column_schema, Rc::clone(&self.memory)))
                }

                
                // not found yet, get next address
                let mut next: [u8; 4] = [0; 4];
                {
                    let memory = self.memory.borrow();
                    next.copy_from_slice(&memory.bytes[(prev_addr + 4)..(prev_addr + 8)]);
                }
                
                let next_ptr = u32::from_le_bytes(next) as usize;
                if next_ptr == 0 {
                    do_continue = false;
                } else {
                    prev_addr = next_ptr;
                }
            }

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
}