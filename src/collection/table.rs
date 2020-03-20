use crate::pointer::NP_ValueInto;
use crate::pointer::NP_PtrKinds;
use crate::{memory::NP_Memory, pointer::{NP_Value, NP_Ptr}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

pub struct NP_Table<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Option<&'a NP_Memory>,
    columns: Option<&'a Vec<Option<(u8, String, NP_Schema)>>>
}

impl<'a> NP_Value for NP_Table<'a> {
    fn new<T: NP_Value + Default>() -> Self {
        unreachable!()
    }
    fn is_type( _type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "table".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "table".to_owned()) }
    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .get()! Use .into() instead."))
    }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a> NP_ValueInto<'a> for NP_Table<'a> {
    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<NP_Table<'a>>>, NP_Error> {
        
        match &*schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut addr = kind.get_value();

                let mut head: [u8; 4] = [0; 4];

                if addr == 0 {
                    // no table here, make one
                    addr = buffer.malloc([0 as u8; 4].to_vec())?; // stores HEAD for table
                    buffer.set_value_address(address, addr, &kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                }

                Ok(Some(Box::new(NP_Table::new(addr, u32::from_le_bytes(head), buffer, &columns))))
            },
            _ => {
                Err(NP_Error::new(""))
            }
        }
    }
}

impl<'a> Default for NP_Table<'a> {

    fn default() -> Self {
        NP_Table { address: 0, head: 0, memory: None, columns: None}
    }
}

impl<'a> NP_Table<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: &'a NP_Memory, columns: &'a Vec<Option<(u8, String, NP_Schema)>>) -> Self {
        NP_Table {
            address,
            head,
            memory: Some(memory),
            columns: Some(columns)
        }
    }

    pub fn select<X: NP_Value + Default + NP_ValueInto<'a>>(&mut self, column: &str) -> core::result::Result<NP_Ptr<'a, X>, NP_Error> {

        let mut column_schema: Option<&NP_Schema> = None;

        let column_index = &self.columns.unwrap().iter().fold(0u8, |prev, cur| {
            match cur {
                Some(x) => {
                    if x.1 == column { 
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

                let memory = self.memory.unwrap();


                // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
                if X::type_idx().0 != NP_TypeKeys::Any as i64 && some_column_schema.type_data.0 != NP_TypeKeys::Any as i64  {

                    // not using any casting, check type
                    if some_column_schema.type_data.0 != X::type_idx().0 {
                        let mut err = "TypeError: Attempted to cast type (".to_owned();
                        err.push_str(X::type_idx().1.as_str());
                        err.push_str(") to schema of type (");
                        err.push_str(some_column_schema.type_data.1.as_str());
                        err.push_str(")");
                        return Err(NP_Error::new(err));
                    }
                }

                if self.head == 0 { // no values, create one

                    let mut ptr_bytes: [u8; 9] = [0; 9];
    
                    // set column index in pointer
                    ptr_bytes[8] = *column_index;
        
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);

                    // provide
                    return Ok(NP_Ptr::new_table_item_ptr(self.head, some_column_schema, &memory));
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    while has_next {

                        let index;
                     
                        index = memory.read_bytes()[(next_addr + 8)];
                        
                        // found our value!
                        if index == *column_index {
                            return Ok(NP_Ptr::new_table_item_ptr(next_addr as u32, some_column_schema, &memory))
                        }
                        
                        // not found yet, get next address
                        let mut next: [u8; 4] = [0; 4];
                        next.copy_from_slice(&memory.read_bytes()[(next_addr + 4)..(next_addr + 8)]);

                        let next_ptr = u32::from_le_bytes(next) as usize;
                        if next_ptr == 0 {
                            has_next = false;
                        } else {
                            next_addr = next_ptr;
                        }
                    }

                    // ran out of pointers to check, make one!

                    
                    let mut ptr_bytes: [u8; 9] = [0; 9];
    
                    // set column index in pointer
                    ptr_bytes[8] = *column_index;

                    let mem_bytes = self.memory.unwrap();
            
                    let addr = mem_bytes.malloc(ptr_bytes.to_vec())?;

                    // set previouse pointer's "next" value to this new pointer
                    let addr_bytes = addr.to_le_bytes();
                    let write_bytes = mem_bytes.write_bytes();
                    for x in 0..addr_bytes.len() {
                        write_bytes[(next_addr + 4 + x)] = addr_bytes[x];
                    }
                    
                    // provide 
                    return Ok(NP_Ptr::new_table_item_ptr(addr, some_column_schema, &mem_bytes));

                }
            },
            None => {
                let mut err_msg = "Column (".to_owned();
                err_msg.push_str(column);
                err_msg.push_str(") not found, unable to select!");
                return Err(NP_Error::new(err_msg.as_str()));
            }
        }
    }

    pub fn empty(self) -> Self {

        let memory_bytes = self.memory.unwrap().write_bytes();
       
        let head_bytes = 0u32.to_le_bytes();

        for x in 0..head_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = head_bytes[x as usize];
        }

        NP_Table {
            address: self.address,
            head: 0,
            memory: self.memory,
            columns: self.columns
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let memory_bytes = self.memory.unwrap().write_bytes();
       
        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
      
    }

    pub fn has(&self, column: &str) -> core::result::Result<bool, NP_Error> {
        let mut found = false;

        if self.head == 0 { // no values in this table
           return Ok(false);
        }

        let column_index = &self.columns.unwrap().iter().fold(0, |prev, cur| {
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
        let memory = self.memory.unwrap();

        while has_next {

            let index;

        
            index = memory.read_bytes()[(next_addr + 8)];

            // found our value!
            if index == *column_index {
                return Ok(true);
            }

            
            // not found yet, get next address
            let mut next: [u8; 4] = [0; 4];
            
            next.copy_from_slice(&memory.read_bytes()[(next_addr + 4)..(next_addr + 8)]);
            
            next_addr = u32::from_le_bytes(next) as usize;
            if next_addr== 0 {
                has_next = false;
            }
        }

        // ran out of pointers, value doesn't exist!
        return Ok(false);
    }

}