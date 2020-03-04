use crate::NoProtoSchema;
use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use crate::pointer::NoProtoPointerKinds;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoColumn {
    name: String,
    index: u8,
    type_string: String
}

pub struct NoProtoTable<'a> {
    pub address: u32, // pointer location
    pub head: u32,
    pub memory: Rc<RefCell<NoProtoMemory>>,
    pub columns: &'a Vec<Option<(u8, String, NoProtoSchema)>>
}

impl<'a> NoProtoTable<'a> {


    pub fn select(&mut self, column: &str) -> Option<NoProtoPointer> {

        let mut column_schema: Option<&NoProtoSchema> = None;

        let column_index = &self.columns.iter().fold(0, |prev, cur| {
            match cur {
                Some(x) => {
                    if x.1.as_str() == column { 
                        column_schema = Some(&x.2);
                        return x.0; 
                    }
                    prev
                }
                None => {
                    prev
                }
            }
        });

        match column_schema {
            Some(some_column_schema) => {

                if self.head == 0 { // no values, create one

                    let mut addr = self.head;
        
                    {
                        let mut memory = self.memory.borrow_mut();
        
                        let mut ptr_bytes: [u8; 13] = [0; 13];
        
                        // set column index in pointer
                        ptr_bytes[4] = *column_index;
        
                        // set prev to table head address
                        let head_bytes = self.address.to_le_bytes();
                        for i in 0..4 {
                            let prev_addr = 4 + 1 + 4 + i;
                            ptr_bytes[prev_addr] = head_bytes[i];
                        }
        
                        addr = memory.malloc(ptr_bytes.to_vec()).unwrap_or(0);
            
                        // out of memory
                        if addr == 0 { return None; }
                    }
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);
                    
                    // provide 
                    return Some(NoProtoPointer::new_table_item(self.head, some_column_schema, Rc::clone(&self.memory)));
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    let memory = self.memory.borrow();

                    while has_next {

                        let index = memory.bytes[(next_addr + 4)];
                        
                        // found our value!
                        if index == *column_index {
                            return Some(NoProtoPointer::new_table_item(next_addr as u32, some_column_schema, Rc::clone(&self.memory)))
                        }

                        // not found yet, get next address
                        let mut next: [u8; 4] = [0; 4];
                        next.copy_from_slice(&memory.bytes[(next_addr + 5)..(next_addr + 9)]);
                        let next_ptr = u32::from_le_bytes(next) as usize;
                        if next_ptr == 0 {
                            has_next = false;
                        } else {
                            next_addr = next_ptr;
                        }
                    }

                    // ran out of pointers to check, make one!

                }

                return None;
            },
            None => {
                return None;
            }
        }

        None
    }

    pub fn set_head(&mut self, addr: u32) {
        let mut memory = self.memory.borrow_mut();

        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    pub fn delete(&mut self, column: &str) -> bool {
        false
    }

    pub fn clear(&mut self) {
        self.set_head(0);
    }

    pub fn has(&self, column: &str) {

    }

}