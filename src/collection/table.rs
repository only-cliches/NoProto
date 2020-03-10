use crate::{memory::NoProtoMemory, pointer::NoProtoPointer, error::NoProtoError, schema::NoProtoSchema};
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoTable<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    columns: &'a Vec<Option<(u8, String, NoProtoSchema)>>
}

impl<'a> NoProtoTable<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<RefCell<NoProtoMemory>>, columns: &'a Vec<Option<(u8, String, NoProtoSchema)>>) -> Self {
        NoProtoTable {
            address,
            head,
            memory,
            columns
        }
    }

    pub fn select(&mut self, column: &str) -> std::result::Result<NoProtoPointer, NoProtoError> {

        let mut column_schema: Option<&NoProtoSchema> = None;

        let column_index = &self.columns.iter().fold(0u8, |prev, cur| {
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
        }) as &u8;

        match column_schema {
            Some(some_column_schema) => {

                if self.head == 0 { // no values, create one

                    let addr;
        
                    {
                        let mut memory = self.memory.try_borrow_mut()?;
        
                        let mut ptr_bytes: [u8; 9] = [0; 9];
        
                        // set column index in pointer
                        ptr_bytes[8] = *column_index;
        
                        addr = memory.malloc(ptr_bytes.to_vec())?;
                    }
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);
                    
                    // provide 
                    return Ok(NoProtoPointer::new_table_item_ptr(self.head, some_column_schema, Rc::clone(&self.memory))?);
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    while has_next {

                        let index;

                        {
                            let memory = self.memory.try_borrow()?;
                            index = memory.bytes[(next_addr + 8)];
                        }

                        // found our value!
                        if index == *column_index {
                            return Ok(NoProtoPointer::new_table_item_ptr(next_addr as u32, some_column_schema, Rc::clone(&self.memory))?)
                        }

                        
                        // not found yet, get next address
                        let mut next: [u8; 4] = [0; 4];
                        {
                            let memory = self.memory.try_borrow()?;
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
                        let mut memory = self.memory.try_borrow_mut()?;
        
                        let mut ptr_bytes: [u8; 9] = [0; 9];
        
                        // set column index in pointer
                        ptr_bytes[8] = *column_index;
                
                        addr = memory.malloc(ptr_bytes.to_vec())?;

                        // set previouse pointer's "next" value to this new pointer
                        let addr_bytes = addr.to_le_bytes();
                        for x in 0..addr_bytes.len() {
                            memory.bytes[(next_addr + 4 + x)] = addr_bytes[x];
                        }

                    }
                    
                    // provide 
                    return Ok(NoProtoPointer::new_table_item_ptr(addr, some_column_schema, Rc::clone(&self.memory))?);

                }
            },
            None => {
                return Err(NoProtoError::new("Column not found, unable to select!"));
            }
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let mut memory = self.memory.borrow_mut();

        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    pub fn has(&self, column: &str) -> std::result::Result<bool, NoProtoError> {
        let mut found = false;

        if self.head == 0 { // no values in this table
           return Ok(false);
        }

        let column_index = &self.columns.iter().fold(0, |prev, cur| {
            match cur {
                Some(x) => {
                    if x.1.as_str() == column { 
                        found = true;
                        return x.0; 
                    }
                    prev
                }
                None => {
                    prev
                }
            }
        }) as &u8;

        // no column with this name
        if found == false { return Ok(false); };

        // values exist, loop through values to see if we have a matching column

        let mut next_addr = self.head as usize;

        let mut has_next = true;

        while has_next {

            let index;

            {
                let memory = self.memory.try_borrow()?;
                index = memory.bytes[(next_addr + 8)];
            }

            // found our value!
            if index == *column_index {
                return Ok(true);
            }

            
            // not found yet, get next address
            let mut next: [u8; 4] = [0; 4];
            {
                let memory = self.memory.try_borrow()?;
                next.copy_from_slice(&memory.bytes[(next_addr + 4)..(next_addr + 8)]);
            }
            
            next_addr = u32::from_le_bytes(next) as usize;
            if next_addr== 0 {
                has_next = false;
            }
        }

        // ran out of pointers, value doesn't exist!
        return Ok(false);
    }

}