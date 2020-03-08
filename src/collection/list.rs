use crate::NoProtoSchema;
use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
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

    pub fn select(&mut self, index: u16) -> NoProtoPointer {


        if self.head == 0 { // no values, create one

            let addr;

            {
                let mut memory = self.memory.borrow_mut();

                let mut ptr_bytes: [u8; 9] = [0; 9];

                // set column index in pointer
                ptr_bytes[8] = *column_index;

                addr = memory.malloc(ptr_bytes.to_vec()).unwrap_or(0);
    
                // out of memory
                if addr == 0 { return None; }
            }
            
            // update head to point to newly created TableItem pointer
            self.set_head(addr);
            
            // provide 
            return Some(NoProtoPointer::new_table_item_ptr(self.head, some_column_schema, Rc::clone(&self.memory)));
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut next_addr = self.head as usize;

            let mut has_next = true;

            while has_next {

                let index;

                {
                    let memory = self.memory.borrow();
                    index = memory.bytes[(next_addr + 8)];
                }

                // found our value!
                if index == *column_index {
                    return Some(NoProtoPointer::new_table_item_ptr(next_addr as u32, some_column_schema, Rc::clone(&self.memory)))
                }

                
                // not found yet, get next address
                let mut next: [u8; 4] = [0; 4];
                {
                    let memory = self.memory.borrow();
                    next.copy_from_slice(&memory.bytes[(next_addr + 4)..(next_addr + 8)]);
                }
                
                let next_ptr = u32::from_le_bytes(next) as usize;
                if next_ptr == 0 {
                    has_next = false;
                } else {
                    next_addr = next_ptr;
                }
            }

            // ran out of pointers to check, make one!

            let addr;

            {
                let mut memory = self.memory.borrow_mut();

                let mut ptr_bytes: [u8; 9] = [0; 9];

                // set column index in pointer
                ptr_bytes[8] = *column_index;
        
                addr = memory.malloc(ptr_bytes.to_vec()).unwrap_or(0);
    
                // out of memory
                if addr == 0 { return None; }

                // set previouse pointer's "next" value to this new pointer
                let addr_bytes = addr.to_le_bytes();
                for x in 0..addr_bytes.len() {
                    memory.bytes[(next_addr + 4 + x)] = addr_bytes[x];
                }

            }
            
            // provide 
            return Some(NoProtoPointer::new_table_item_ptr(addr, some_column_schema, Rc::clone(&self.memory)));
        }
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