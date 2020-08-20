use crate::pointer::NP_PtrKinds;
use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value, NP_Ptr, any::NP_Any, NP_Lite_Ptr}, error::NP_Error, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};
use core::result::Result;

/// The data type for tables in NoProto buffers. [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug)]
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
    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        
        match &*ptr.schema.kind {
            NP_SchemaKinds::Table { columns } => {

                match &ptr.memory.size {
                    NP_Size::U16 => {
                        let mut addr = ptr.kind.get_value_addr();

                        let mut head: [u8; 2] = [0; 2];
        
                        if addr == 0 {
                            // no table here, make one
                            addr = ptr.memory.malloc([0 as u8; 2].to_vec())?; // stores HEAD
                            ptr.memory.set_value_address(ptr.location, addr, &ptr.kind); // set pointer to new table
                        } else {
                            // existing head, read value
                            let a = addr as usize;
                            head = *ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2]);
                        }
        
                        Ok(Some(Box::new(NP_Table::new(addr, u16::from_be_bytes(head) as u32, ptr.memory, Rc::clone(&columns)))))
                    },
                    NP_Size::U32 => {
                        let mut addr = ptr.kind.get_value_addr();

                        let mut head: [u8; 4] = [0; 4];
        
                        if addr == 0 {
                            // no table here, make one
                            addr = ptr.memory.malloc([0 as u8; 4].to_vec())?; // stores HEAD
                            ptr.memory.set_value_address(ptr.location, addr, &ptr.kind); // set pointer to new table
                        } else {
                            // existing head, read value
                            let a = addr as usize;
                            head = *ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]);
                        }
        
                        Ok(Some(Box::new(NP_Table::new(addr, u32::from_be_bytes(head), ptr.memory, Rc::clone(&columns)))))
                    }
                }


            },
            _ => {
                unreachable!();
            }
        }
    }
 
    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let base_size = match &ptr.memory.size {
            NP_Size::U16 => 2u32,
            NP_Size::U32 => 4u32
        };

        match &*ptr.schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut acc_size = 0u32;

                match NP_Table::into_value(ptr.clone())? {
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

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        match &*ptr.schema.kind {
            NP_SchemaKinds::Table { columns } => {

                let mut object = JSMAP::<NP_JSON>::new();

                let table = NP_Table::into_value(ptr.clone());

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

    fn do_compact(from_ptr: NP_Lite_Ptr, to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok(());
        }

        let to_ptr_list = to_ptr.into::<Self>();

        let new_address = to_ptr_list.location;

        match Self::into_value(from_ptr)? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(mut new_list) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value.0 && item.has_value.1 {
                                let new_ptr = NP_Lite_Ptr::from(new_list.select::<NP_Any>(&item.column)?);
                                let old_ptr = NP_Lite_Ptr::from(item.select::<NP_Any>()?);
                                old_ptr.compact(new_ptr)?;
                            }

                        }
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok(())
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

    /// Convert the table into an iterator.  Allows you to loop through all the values present in the table.
    /// 
    pub fn it(self) -> NP_Table_Iterator {
        NP_Table_Iterator::new(self.address, self.head, self.memory.unwrap(), self.columns.unwrap())
    }

    /// Select a specific column from the table.  If there is no value for the column you selected, you'll get an empty pointer back.
    /// 
    /// If the column does not exist this operation will fail.
    /// 
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

                    let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::TableItem { addr: 0, i: 0, next: 0 }); // Map item pointer
    
                    // set column index in pointer
                    match &memory.size {
                        NP_Size::U16 => { 
                            ptr_bytes[4] = column_index;
                        },
                        NP_Size::U32 => {
                            ptr_bytes[8] = column_index;
                        }
                    }
        
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);

                    // provide
                    return Ok(NP_Ptr::_new_table_item_ptr(self.head, Rc::clone(&some_column_schema), Rc::clone(&memory)));
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    while has_next {

                        let index = match &memory.size {
                            NP_Size::U16 => { memory.read_bytes()[(next_addr + 4)] },
                            NP_Size::U32 => { memory.read_bytes()[(next_addr + 8)] }
                        };
                        
                        // found our value!
                        if index == column_index {
                            return Ok(NP_Ptr::_new_table_item_ptr(next_addr as u32, Rc::clone(&some_column_schema), Rc::clone(&memory)))
                        }
                        
                        // not found yet, get next address
                        let next_ptr = match memory.size {
                            NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(next_addr + 2).unwrap_or(&[0; 2])) as usize,
                            NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4])) as usize
                        };
                        if next_ptr == 0 {
                            has_next = false;
                        } else {
                            next_addr = next_ptr;
                        }
                    }

                    // ran out of pointers to check, make one!

                    
                    let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::TableItem { addr: 0, i: 0, next: 0 }); // Map item pointer
    
                    // set column index in pointer
                    match &memory.size {
                        NP_Size::U16 => { 
                            ptr_bytes[4] = column_index;
                        },
                        NP_Size::U32 => {
                            ptr_bytes[8] = column_index;
                        }
                    }
            
                    let addr = memory.malloc(ptr_bytes.to_vec())?;

                    let write_bytes = memory.write_bytes();

                    // set previouse pointer's "next" value to this new pointer
                    match &memory.size {
                        NP_Size::U16 => { 
                            let addr_bytes = (addr as u16).to_be_bytes();
                            for x in 0..addr_bytes.len() {
                                write_bytes[(next_addr + 2 + x)] = addr_bytes[x];
                            }
                        },
                        NP_Size::U32 => {
                            let addr_bytes = addr.to_be_bytes();
                            for x in 0..addr_bytes.len() {
                                write_bytes[(next_addr + 4 + x)] = addr_bytes[x];
                            }
                        }
                    }
                    
                    // provide 
                    return Ok(NP_Ptr::_new_table_item_ptr(addr, Rc::clone(&some_column_schema), memory));

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


    /// Delets a specific column value from this table.  If the column value doesn't exist or the table is empty this does nothing.
    /// 
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

                let index = match &memory.size {
                    NP_Size::U16 => { memory.read_bytes()[(curr_addr + 4)] },
                    NP_Size::U32 => { memory.read_bytes()[(curr_addr + 8)] }
                };
                
                // found our value!
                if index == *column_index {



                    let next_pointer: u32 = match &memory.size {
                        NP_Size::U16 => { 
                            let next_pointer_bytes: [u8; 2];

                            match memory.get_2_bytes(curr_addr + 2) {
                                Some(x) => {
                                    next_pointer_bytes = *x;
                                },
                                None => {
                                    return Err(NP_Error::new("Out of range request"));
                                }
                            }
                            u16::from_be_bytes(next_pointer_bytes) as u32
                        },
                        NP_Size::U32 => { 
                            let next_pointer_bytes: [u8; 4];

                            match memory.get_4_bytes(curr_addr + 4) {
                                Some(x) => {
                                    next_pointer_bytes = *x;
                                },
                                None => {
                                    return Err(NP_Error::new("Out of range request"));
                                }
                            }
                            u32::from_be_bytes(next_pointer_bytes)
                        }
                    };

                    if curr_addr == self.head as usize { // item is HEAD, just set head to following pointer
                        self.set_head(next_pointer);
                    } else { // item is NOT head, set previous pointer's NEXT value to the pointer following this one
                
                        let memory_bytes = memory.write_bytes();

                        match &memory.size {
                            NP_Size::U16 => {
                                let next_pointer_bytes = (next_pointer as u16).to_be_bytes();
                                for x in 0..next_pointer_bytes.len() {
                                    memory_bytes[(prev_addr + x as u32 + 2) as usize] = next_pointer_bytes[x as usize];
                                };
                            },
                            NP_Size::U32 => {
                                let next_pointer_bytes = next_pointer.to_be_bytes();
                                for x in 0..next_pointer_bytes.len() {
                                    memory_bytes[(prev_addr + x as u32 + 4) as usize] = next_pointer_bytes[x as usize];
                                };
                            }
                        }
                    }

                    return Ok(true);
                }
                
                // not found yet, get next address
                let next_ptr = match memory.size {
                    NP_Size::U16 => u16::from_be_bytes(*memory.get_2_bytes(curr_addr + 2).unwrap_or(&[0; 2])) as usize,
                    NP_Size::U32 => u32::from_be_bytes(*memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4])) as usize
                };
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

    /// Clear all column values from this table.
    /// 
    pub fn empty(self) -> Self {

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let memory_bytes = memory.write_bytes();
       
        let head_bytes = match memory.size {
            NP_Size::U16 => { 0u16.to_be_bytes().to_vec() },
            NP_Size::U32 => { 0u32.to_be_bytes().to_vec() }
        };

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
       
        let addr_bytes = match memory.size {
            NP_Size::U16 => { (addr as u16).to_be_bytes().to_vec() },
            NP_Size::U32 => { addr.to_be_bytes().to_vec() }
        };

        for x in 0..addr_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
      
    }

    /// Check to see if a specific column value has been set in this table.
    /// 
    /// The first bool is if a pointer exists for this column, the second bool is if there is a value set on that pointer.
    /// 
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

            let index = match &memory.size {
                NP_Size::U16 => { memory.read_bytes()[(next_addr + 4)] },
                NP_Size::U32 => { memory.read_bytes()[(next_addr + 8)] }
            };

            // found our value!
            if index == *column_index {
                let value_addr = match &memory.size {
                    NP_Size::U16 => { u16::from_be_bytes(*memory.get_2_bytes(next_addr).unwrap_or(&[0; 2])) as u32 },
                    NP_Size::U32 => { u32::from_be_bytes(*memory.get_4_bytes(next_addr).unwrap_or(&[0; 4])) }
                };
                return Ok((true, value_addr != 0));
            }

            
            // not found yet, get next address
            
            next_addr = match &memory.size {
                NP_Size::U16 => { u16::from_be_bytes(*memory.get_2_bytes(next_addr + 2).unwrap_or(&[0; 2])) as usize },
                NP_Size::U32 => { u32::from_be_bytes(*memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4])) as usize }
            };

            if next_addr== 0 {
                has_next = false;
            }
        }

        // ran out of pointers, value doesn't exist!
        return Ok((false, false));
    }

}

/// Iterator over table column values
#[derive(Debug)]
pub struct NP_Table_Iterator {
    address: u32, // pointer location
    head: u32,
    memory: Rc<NP_Memory>,
    columns: Rc<Vec<Option<(u8, String, Rc<NP_Schema>)>>>,
    column_index: u8,
    table: NP_Table
}

impl NP_Table_Iterator {

    #[doc(hidden)]
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

    /// Convert the iterator back into a table.
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

///  A single item of the table iterator.
#[derive(Debug)]
pub struct NP_Table_Item { 
    /// The index of this item in the table
    pub index: u8,
    /// The column string name for this index
    pub column: String,
    /// (has pointer at this index, his value at this index)
    pub has_value: (bool, bool),
    schema: Rc<NP_Schema>,
    table: NP_Table
}

impl NP_Table_Item {
    /// Select the pointer at this iterator
    // TODO: Build a select statement that grabs the current index in place instead of seeking to it.
    pub fn select<X: NP_Value + Default>(&mut self) -> Result<NP_Ptr<X>, NP_Error> {
        self.table.select(&self.column)
    }
    /// Delete the value and it's pointer at this iterator
    // TODO: Same as select for delete
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.table.delete(&self.column)
    }
}

