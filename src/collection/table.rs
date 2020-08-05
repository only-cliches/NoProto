use crate::pointer::NP_PtrKinds;
use crate::{memory::NP_Memory, pointer::{NP_Value, NP_Ptr, any::NP_Any}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};
use core::result::Result;

pub struct NP_Table {
    address: u32, // pointer location
    head: u32,
    memory: Option<Rc<NP_Memory>>,
    columns: Option<Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>>>
}

impl NP_Value for NP_Table {
    fn is_type( _type_str: &str) -> bool {  // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::Table as i64, "table".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Table as i64, "table".to_owned()) }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: Rc<NP_Schema>, _buffer: Rc<NP_Memory>, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()! Use .into() instead."))
    }

    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        
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

                Ok(Some(Box::new(NP_Table::new(addr, u32::from_be_bytes(head), buffer, Rc::clone(&columns)))))
            },
            _ => {
                unreachable!();
            }
        }
    }
 
    fn buffer_get_size(address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> Result<u32, NP_Error> {
        let base_size = 4u32; // head
        let addr = kind.get_value();

        if addr == 0 {
            return Ok(0);
        }

        match &*schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut acc_size = 0u32;

                match NP_Table::buffer_into(address, *kind, Rc::clone(&schema), buffer)? {
                    Some(mut real_table) => {

                        for c in columns.as_ref() {
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

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> NP_JSON {
        let addr = kind.get_value();

        if addr == 0 {
            return NP_JSON::Null;
        }

        match &*schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut object = JSMAP::<NP_JSON>::new();

                let table = NP_Table::buffer_into(address, *kind, Rc::clone(&schema), buffer);

                match table {
                    Ok(good_table) => {
                        match good_table {
                            Some(mut real_table) => {

                                for c in columns.as_ref() {
                                    match c {
                                        Some(x) => {
                                            let col_ptr = real_table.select::<NP_Any>(x.1.as_str());
                                            match col_ptr {
                                                Ok(ptr) => {
                                                    object.insert(x.1.to_owned(), ptr.json_encode());
                                                },
                                                Err(_e) => {
                                                    object.insert(x.1.to_owned(), NP_JSON::Null);
                                                }
                                            }
                                        },
                                        None => {
                                            
                                        }
                                    }
                                }

                            },
                            None => {
                                return NP_JSON::Null;
                            }
                        }
                    },
                    Err(_e) => {
                        return NP_JSON::Null;
                    }
                }

                NP_JSON::Dictionary(object)
            },
            _ => {
                NP_JSON::Null
            }
        }
    }

    fn buffer_do_compact<'b, X: NP_Value + Default>(from_ptr: &'b NP_Ptr<X>, to_ptr: NP_Ptr<NP_Any>) -> Result<(u32, NP_PtrKinds, Rc<NP_Schema>), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok((0, from_ptr.kind, Rc::clone(&from_ptr.schema)));
        }

        let to_ptr_list = NP_Any::cast::<NP_Table>(to_ptr)?;

        let new_address = to_ptr_list.location;

        match Self::buffer_into(from_ptr.location, from_ptr.kind, Rc::clone(&from_ptr.schema), Rc::clone(&from_ptr.memory))? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(mut new_list) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value.0 && item.has_value.1 {
                                let new_ptr = new_list.select(&item.column)?;
                                let old_ptr = item.select::<NP_Any>()?;
                                old_ptr._compact(new_ptr)?;
                            }

                        }

                        return Ok((new_address, from_ptr.kind, Rc::clone(&from_ptr.schema)));
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok((0, from_ptr.kind, Rc::clone(&from_ptr.schema)))
    }
}

impl Default for NP_Table {

    fn default() -> Self {
        NP_Table { address: 0, head: 0, memory: None, columns: None}
    }
}

impl NP_Table {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: Rc<NP_Memory>, columns: Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>>) -> Self {
        NP_Table {
            address,
            head,
            memory: Some(memory),
            columns: Some(columns)
        }
    }

    pub fn it(self) -> NP_Table_Iterator {
        NP_Table_Iterator::new(self.address, self.head, self.memory.unwrap(), self.columns.unwrap())
    }

    pub fn select<X: NP_Value + Default>(&mut self, column: &str) -> core::result::Result<NP_Ptr<X>, NP_Error> {

        let mut column_schema: Option<Rc<NP_Schema>> = None;

        let cols = match &self.columns {
            Some(x) => x,
            None => unreachable!()
        };

        let column_index = cols.iter().fold(0u8, |prev, cur| {
            match cur {
                Some(x) => {
                    if x.1 == column { 
                        column_schema = Some(Rc::clone(&x.2));
                        return x.0; 
                    }
                    prev
                }
                None => {
                    prev
                }
            }
        }) as u8;

        match column_schema {
            Some(some_column_schema) => {

                let memory = match &self.memory {
                    Some(x) => Rc::clone(x),
                    None => unreachable!()
                };

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
                    ptr_bytes[8] = column_index;
        
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);

                    // provide
                    return Ok(NP_Ptr::new_table_item_ptr(self.head, Rc::clone(&some_column_schema), Rc::clone(&memory)));
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    while has_next {

                        let index;
                     
                        index = memory.read_bytes()[(next_addr + 8)];
                        
                        // found our value!
                        if index == column_index {
                            return Ok(NP_Ptr::new_table_item_ptr(next_addr as u32, Rc::clone(&some_column_schema), Rc::clone(&memory)))
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
                    ptr_bytes[8] = column_index;
            
                    let addr = memory.malloc(ptr_bytes.to_vec())?;

                    // set previouse pointer's "next" value to this new pointer
                    let addr_bytes = addr.to_be_bytes();
                    let write_bytes = memory.write_bytes();
                    for x in 0..addr_bytes.len() {
                        write_bytes[(next_addr + 4 + x)] = addr_bytes[x];
                    }
                    
                    // provide 
                    return Ok(NP_Ptr::new_table_item_ptr(addr, Rc::clone(&some_column_schema), memory));

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

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let cols = match &self.columns {
            Some(x) => x,
            None => unreachable!()
        };

        let column_index = &cols.iter().fold(0u8, |prev, cur| {
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

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let memory_bytes = memory.write_bytes();
       
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

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let memory_bytes = memory.write_bytes();
       
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

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let cols = match &self.columns {
            Some(x) => x,
            None => unreachable!()
        };

        let column_index = &cols.iter().fold(0, |prev, cur| {
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

pub struct NP_Table_Iterator {
    address: u32, // pointer location
    head: u32,
    memory: Rc<NP_Memory>,
    columns: Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>>,
    column_index: u8,
    table: NP_Table
}

impl NP_Table_Iterator {

    pub fn new(address: u32, head: u32, memory: Rc<NP_Memory>, columns: Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>>) -> Self {
        NP_Table_Iterator {
            address,
            head,
            memory: Rc::clone(&memory),
            columns: Rc::clone(&columns),
            column_index: 0,
            table: NP_Table::new(address, head, memory, columns)
        }
    }

    pub fn into_table(self) -> NP_Table {
        self.table
    }
}

impl Iterator for NP_Table_Iterator {
    type Item = NP_Table_Item;

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
                            schema: Rc::clone(&x.2),
                            table: NP_Table::new(self.address, self.head, Rc::clone(&self.memory), Rc::clone(&self.columns))
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

pub struct NP_Table_Item { 
    pub index: u8,
    pub column: String,
    pub has_value: (bool, bool),
    pub schema: Rc<NP_Schema>,
    table: NP_Table
}

impl NP_Table_Item {
    // TODO: Build a select statement that grabs the current index in place instead of seeking to it.
    pub fn select<X: NP_Value + Default>(&mut self) -> Result<NP_Ptr<X>, NP_Error> {
        self.table.select(&self.column)
    }
    // TODO: Same as select for delete
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.table.delete(&self.column)
    }
}

