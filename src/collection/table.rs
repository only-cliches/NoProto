use crate::pointer::NP_ValueInto;
use crate::pointer::NP_PtrKinds;
use crate::{memory::NP_Memory, pointer::{NP_Value, NP_Ptr, any::NP_Any}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, json_flex::{JFMap, JFObject}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::result::Result;

pub struct NP_Table<'a> {
    address: u32, // pointer location
    head: u32,
    memory: Option<&'a NP_Memory>,
    columns: Option<&'a Vec<Option<(u8, String, NP_Schema)>>>
}

impl<'a> NP_Value for NP_Table<'a> {
    fn is_type( _type_str: &str) -> bool {  // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::Table as i64, "table".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Table as i64, "table".to_owned()) }
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
                    addr = buffer.malloc([0 as u8; 4].to_vec())?; // stores HEAD
                    buffer.set_value_address(address, addr, &kind); // set pointer to new table
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                }

                Ok(Some(Box::new(NP_Table::new(addr, u32::from_be_bytes(head), buffer, &columns))))
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_get_size(address: u32, kind: &'a NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> Result<u32, NP_Error> {
        let base_size = 4u32; // head
        let addr = kind.get_value();

        if addr == 0 {
            return Ok(0);
        }

        match &*schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut acc_size = 0u32;

                match NP_Table::buffer_into(address, *kind, schema, buffer)? {
                    Some(mut real_table) => {

                        for c in columns {
                            match c {
                                Some(x) => {
                                    let has_value = real_table.has(&*x.1)?;

                                    if has_value.1 {
                                        let col_ptr = real_table.select::<NP_Any>(x.1.as_str())?;
                                        let size = col_ptr.calc_size()?;
                                        acc_size += size;
                                    }
                                },
                                None => {
                                    
                                }
                            }
                        }

                    },
                    None => {
                        
                    }
                }

                Ok(base_size + acc_size)
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &'a NP_Memory) -> JFObject {
        let addr = kind.get_value();

        if addr == 0 {
            return JFObject::Null;
        }

        match &*schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut object = JFMap::<JFObject>::new();

                let table = NP_Table::buffer_into(address, *kind, schema, buffer);

                match table {
                    Ok(good_table) => {
                        match good_table {
                            Some(mut real_table) => {

                                for c in columns {
                                    match c {
                                        Some(x) => {
                                            let col_ptr = real_table.select::<NP_Any>(x.1.as_str());
                                            match col_ptr {
                                                Ok(ptr) => {
                                                    object.insert(x.1.to_owned(), ptr.json_encode());
                                                },
                                                Err(_e) => {
                                                    object.insert(x.1.to_owned(), JFObject::Null);
                                                }
                                            }
                                        },
                                        None => {
                                            
                                        }
                                    }
                                }

                            },
                            None => {
                                return JFObject::Null;
                            }
                        }
                    },
                    Err(_e) => {
                        return JFObject::Null;
                    }
                }

                JFObject::Dictionary(object)
            },
            _ => {
                JFObject::Null
            }
        }
    }

    fn buffer_do_compact<X: NP_Value + Default + NP_ValueInto<'a>>(from_ptr: &NP_Ptr<'a, X>, to_ptr: NP_Ptr<'a, NP_Any>) -> Result<(u32, NP_PtrKinds, &'a NP_Schema), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok((0, from_ptr.kind, from_ptr.schema));
        }

        let to_ptr_list = NP_Any::cast::<NP_Table>(to_ptr)?;

        let new_address = to_ptr_list.location;

        match Self::buffer_into(from_ptr.location, from_ptr.kind, from_ptr.schema, from_ptr.memory)? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(mut new_list) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value.0 && item.has_value.1 {
                                let new_ptr = new_list.select(&item.column)?;
                                let old_ptr = item.select::<NP_Any>()?;
                                old_ptr.compact(new_ptr)?;
                            }

                        }

                        return Ok((new_address, from_ptr.kind, from_ptr.schema));
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok((0, from_ptr.kind, from_ptr.schema))
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

    pub fn it(self) -> NP_Table_Iterator<'a> {
        NP_Table_Iterator::new(self.address, self.head, self.memory.unwrap(), self.columns.unwrap())
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
                        let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);

                        let next_ptr = u32::from_be_bytes(next) as usize;
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
                    let addr_bytes = addr.to_be_bytes();
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


    pub fn delete(&mut self, column: &str) -> Result<bool, NP_Error> {

        let memory = self.memory.unwrap();

        let column_index = &self.columns.unwrap().iter().fold(0u8, |prev, cur| {
            match cur {
                Some(x) => {
                    if x.1 == column {
                        return x.0; 
                    }
                    prev
                }
                None => {
                    prev
                }
            }
        }) as &u8;

        if self.head == 0 { // no values, nothing to delete
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut has_next = true;

            while has_next {

                let index;
                     
                index = memory.read_bytes()[(curr_addr + 8)];
                
                // found our value!
                if index == *column_index {

                    let next_pointer_bytes: [u8; 4];

                    match memory.get_4_bytes(curr_addr + 4) {
                        Some(x) => {
                            next_pointer_bytes = *x;
                        },
                        None => {
                            return Err(NP_Error::new("Out of range request"));
                        }
                    }

                    if curr_addr == self.head as usize { // item is HEAD, just set head to following pointer
                        self.set_head(u32::from_be_bytes(next_pointer_bytes));
                    } else { // item is NOT head, set previous pointer's NEXT value to the pointer following this one
                
                        let memory_bytes = memory.write_bytes();
                
                        for x in 0..next_pointer_bytes.len() {
                            memory_bytes[(prev_addr + x as u32 + 4) as usize] = next_pointer_bytes[x as usize];
                        }
                    }

                    return Ok(true);
                }
                
                // not found yet, get next address
                let next: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);

                let next_ptr = u32::from_be_bytes(next) as usize;
                if next_ptr == 0 {
                    has_next = false;
                } else {
                    // store old value for next loop
                    prev_addr = curr_addr as u32;

                    // set next pointer for next loop
                    curr_addr = next_ptr;
                }
            }

            // out of pointers to check, nothing to delete
            Ok(false)
        }
    }

    pub fn empty(self) -> Self {

        let memory_bytes = self.memory.unwrap().write_bytes();
       
        let head_bytes = 0u32.to_be_bytes();

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
       
        let addr_bytes = addr.to_be_bytes();

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
      
    }

    pub fn has(&self, column: &str) -> core::result::Result<(bool, bool), NP_Error> {
        let mut found = false;

        if self.head == 0 { // no values in this table
           return Ok((false, false));
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
        if found == false { return Ok((false, false));};

        // values exist, loop through values to see if we have a matching column

        let mut next_addr = self.head as usize;

        let mut has_next = true;
        let memory = self.memory.unwrap();

        while has_next {

            let index;

        
            index = memory.read_bytes()[(next_addr + 8)];

            // found our value!
            if index == *column_index {
                let value: [u8; 4] = *memory.get_4_bytes(next_addr).unwrap_or(&[0; 4]);
                let value_addr = u32::from_be_bytes(value);
                return Ok((true, value_addr != 0));
            }

            
            // not found yet, get next address
            let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);
            
            next_addr = u32::from_be_bytes(next) as usize;
            if next_addr== 0 {
                has_next = false;
            }
        }

        // ran out of pointers, value doesn't exist!
        return Ok((false, false));
    }

}

pub struct NP_Table_Iterator<'a> {
    address: u32, // pointer location
    head: u32,
    memory: &'a NP_Memory,
    columns: &'a Vec<Option<(u8, String, NP_Schema)>>,
    column_index: u8,
    table: NP_Table<'a>
}

impl<'a> NP_Table_Iterator<'a> {

    pub fn new(address: u32, head: u32, memory: &'a NP_Memory, columns: &'a Vec<Option<(u8, String, NP_Schema)>>) -> Self {
        NP_Table_Iterator {
            address,
            head,
            memory: memory,
            columns: columns,
            column_index: 0,
            table: NP_Table::new(address, head, memory, columns)
        }
    }

    pub fn into_table(self) -> NP_Table<'a> {
        self.table
    }
}

impl<'a> Iterator for NP_Table_Iterator<'a> {
    type Item = NP_Table_Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.column_index as usize) < self.columns.len() {

            let mut column_info;

            loop {
                if (self.column_index as usize) >= self.columns.len() {
                    return None;
                }
                column_info = &self.columns[self.column_index as usize];
                match column_info {
                    Some(x) => {
                        self.column_index += 1;
                        let exists = self.table.has(x.1.as_str()).unwrap();
                        return Some(NP_Table_Item {
                            index: x.0,
                            column: x.1.clone(),
                            has_value: exists,
                            schema: &x.2,
                            table: NP_Table::new(self.address, self.head, self.memory, self.columns)
                        });
                    },
                    None => {
                        self.column_index += 1;
                    }
                }
            }
        }

        None
    }
}

pub struct NP_Table_Item<'a> { 
    pub index: u8,
    pub column: String,
    pub has_value: (bool, bool),
    pub schema: &'a NP_Schema,
    table: NP_Table<'a>
}

impl<'a> NP_Table_Item<'a> {
    // TODO: Build a select statement that grabs the current index in place instead of seeking to it.
    pub fn select<X: NP_Value + Default + NP_ValueInto<'a>>(&mut self) -> Result<NP_Ptr<'a, X>, NP_Error> {
        self.table.select(&self.column)
    }
    // TODO: Same as select for delete
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.table.delete(&self.column)
    }
}

