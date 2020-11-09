use crate::schema::NP_Schema_Ptr;
use crate::pointer::NP_PtrKinds;
use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value, NP_Ptr, any::NP_Any, NP_Lite_Ptr}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use core::result::Result;

/// The data type for tables in NoProto buffers. [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug)]
pub struct NP_Table<'table> {
    address: u32, // pointer location
    head: u32,
    memory: Option<&'table NP_Memory>,
    schema: Option<NP_Schema_Ptr<'table>>
}

/// Schema state for Tables
#[derive(Debug)]
pub struct NP_Table_Schema_State<'table_schema> {
    columns: Vec<(u8, String, NP_Schema_Ptr<'table_schema>)>
}


impl<'table> NP_Value<'table> for NP_Table<'table> {
    fn type_idx() -> (u8, String) { (NP_TypeKeys::Table as u8, "table".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Table as u8, "table".to_owned()) }


    fn schema_to_json(schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = NP_Table::get_schema_state(schema_ptr.copy());

        let columns: Vec<NP_JSON> = schema_state.columns.into_iter().map(|column| {
            let mut cols: Vec<NP_JSON> = Vec::new();
            cols.push(NP_JSON::String(column.1));
            cols.push(NP_Schema::_type_to_json(&column.2).unwrap());
            NP_JSON::Array(cols)
        }).collect();

        schema_json.insert("columns".to_owned(), NP_JSON::Array(columns));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Lite_Ptr<'table>) -> Result<Option<Box<Self>>, NP_Error> {

        match &ptr.memory.size {
            NP_Size::U8 => {
                let mut addr = ptr.kind.get_value_addr();

                let mut head: [u8; 1] = [0; 1];

                if addr == 0 {
                    // no table here, make one
                    addr = ptr.memory.malloc([0 as u8; 1].to_vec())?; // stores HEAD
                    ptr.memory.set_value_address(ptr.location, addr, &ptr.kind); // set pointer to new table
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = [ptr.memory.get_1_byte(a).unwrap_or(0)];
                }

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u8::from_be_bytes(head) as u32,
                    memory: Some(ptr.memory),
                    schema: Some(ptr.schema)
                })))
            },
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

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u16::from_be_bytes(head) as u32,
                    memory: Some(ptr.memory),
                    schema: Some(ptr.schema)
                })))
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

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u32::from_be_bytes(head) as u32,
                    memory: Some(ptr.memory),
                    schema: Some(ptr.schema)
                })))
            }
        }
    }
 
    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let base_size = match &ptr.memory.size {
            NP_Size::U8  => 1u32,
            NP_Size::U16 => 2u32,
            NP_Size::U32 => 4u32
        };

        let mut acc_size = 0u32;

        match NP_Table::into_value(ptr.clone())? {
            Some(mut real_table) => {

                let schema_state = NP_Table::get_schema_state(ptr.schema.copy());

                for c in schema_state.columns {
                    let has_value = real_table.has(&c.1)?;

                    if has_value.1 {
                        let col_ptr = real_table.select::<NP_Any>(&c.1)?;
                        let size = col_ptr.calc_size()?;
                        acc_size += size;
                    }
                }

            },
            None => {
                
            }
        }

        Ok(base_size + acc_size)
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let mut object = JSMAP::new();

        let table = NP_Table::into_value(ptr.clone());

        match table {
            Ok(good_table) => {
                match good_table {
                    Some(mut real_table) => {

                        let schema_state = NP_Table::get_schema_state(ptr.schema.copy());

                        for c in schema_state.columns {
                            let col_ptr = real_table.select::<NP_Any>(c.1.as_str());
                            match col_ptr {
                                Ok(ptr) => {
                                    object.insert(c.1.to_owned(), ptr.json_encode());
                                },
                                Err(_e) => {
                                    object.insert(c.1.to_owned(), NP_JSON::Null);
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
    }

    fn do_compact(from_ptr: NP_Lite_Ptr<'table>, to_ptr: NP_Lite_Ptr<'table>) -> Result<(), NP_Error> where Self: NP_Value<'table> + Default {

        if from_ptr.location == 0 {
            return Ok(());
        }

        let to_ptr_list = to_ptr.into::<Self>();

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

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "table" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Table as u8);

            let mut column_data: Vec<(String, Vec<u8>)> = Vec::new();

            match &json_schema["columns"] {
                NP_JSON::Array(cols) => {
                    for col in cols {
                        let column_name = match &col[0] {
                            NP_JSON::String(x) => x.clone(),
                            _ => "".to_owned()
                        };
                        if column_name.len() > 255 {
                            return Err(NP_Error::new("Table column names cannot be longer than 255 characters!"))
                        }
 
                        let column_type = NP_Schema::from_json(Box::new(col[1].clone()))?;
                        column_data.push((column_name, column_type.bytes));
                    }
                },
                _ => { 
                    return Err(NP_Error::new("Tables require a 'columns' property that is an array of schemas!"))
                }
            }

            if column_data.len() > 255 {
                return Err(NP_Error::new("Tables cannot have more than 255 columns!"))
            }

            // number of columns
            schema_data.push(column_data.len() as u8);

            for col in column_data {
                // colum name
                let bytes = col.0.as_bytes().to_vec();
                schema_data.push(bytes.len() as u8);
                schema_data.extend(bytes);

                if col.1.len() > u16::max as usize {
                    return Err(NP_Error::new("Schema overflow error!"))
                }
                
                // column type
                schema_data.extend((col.1.len() as u16).to_be_bytes().to_vec());
                schema_data.extend(col.1);
            }

            return Ok(Some(schema_data))
        }

        Ok(None)
    }
}

impl<'table> Default for NP_Table<'table> {

    fn default() -> Self {
        NP_Table { address: 0, head: 0, memory: None, schema: None}
    }
}

impl<'table> NP_Table<'table> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: &'table NP_Memory, schema: NP_Schema_Ptr<'table>) -> Self {
        NP_Table {
            address,
            head,
            memory: Some(memory),
            schema: Some(schema)
        }
    }

    /// Convert the table into an iterator.  Allows you to loop through all the values present in the table.
    /// 
    pub fn it(self) -> NP_Table_Iterator<'table> {
        NP_Table_Iterator::new(self.address, self.head, self.memory.unwrap(), self.schema.unwrap())
    }

    /// Convert schema bytes into Struct
    /// 
    #[doc(hidden)]
    pub fn get_schema_state(schema_ptr: NP_Schema_Ptr<'table>) -> NP_Table_Schema_State<'table> {

        let column_len = schema_ptr.schema.bytes[schema_ptr.address + 1];

        let mut columns: Vec<(u8, String, NP_Schema_Ptr)> = Vec::new();

        let mut offset = schema_ptr.address + 2;

        for x in 0..column_len as usize {
            let col_name_len = schema_ptr.schema.bytes[offset] as usize;
            let col_name_bytes = &schema_ptr.schema.bytes[(offset + 1)..(offset + 1 + col_name_len)];
            let col_name: String = String::from_utf8_lossy(col_name_bytes).into();

            offset += 1 + col_name_len;

            let schema_size = u16::from_be_bytes([
                schema_ptr.schema.bytes[offset],
                schema_ptr.schema.bytes[offset + 1]
            ]) as usize;
    
            columns.push((x as u8, col_name, schema_ptr.copy_with_addr(schema_ptr.address + offset + 2)));

            offset += schema_size + 2;
        }

        NP_Table_Schema_State { columns: columns }
    }

    /// Select a specific column from the table.  If there is no value for the column you selected, you'll get an empty pointer back.
    /// 
    /// If the column does not exist this operation will fail.
    /// 
    pub fn select<X: NP_Value<'table> + Default>(&mut self, column: &str) -> Result<NP_Ptr<'table, X>, NP_Error> {

        let mut column_schema: Option<(u8, String, NP_Schema_Ptr)> = None;

        let schema = self.schema.as_ref().unwrap().copy();

        for col in NP_Table::get_schema_state(schema.copy()).columns {
            if col.1 == column {
                column_schema = Some(col);
            }
        }

        match column_schema {
            Some(some_column_schema) => {

                let memory = self.memory.unwrap();
                let type_data = NP_TypeKeys::from(some_column_schema.2.schema.bytes[some_column_schema.2.address]);

                // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
                if X::type_idx().0 != NP_TypeKeys::Any as u8 && type_data.clone() as u8 != NP_TypeKeys::Any as u8 {

                    // not using any casting, check type
                    if type_data.clone() as u8 != X::type_idx().0 {
                        let mut err = "TypeError: Attempted to cast type (".to_owned();
                        err.push_str(X::type_idx().1.as_str());
                        err.push_str(") to schema of type (");
                        err.push_str(type_data.clone().into_type_idx().1.as_str());
                        err.push_str(")");
                        return Err(NP_Error::new(err));
                    }
                }

                if self.head == 0 { // no values, create one

                    let mut ptr_bytes: Vec<u8> = memory.blank_ptr_bytes(&NP_PtrKinds::TableItem { addr: 0, i: 0, next: 0 }); // Map item pointer
    
                    // set column index in pointer
                    match &memory.size {
                        NP_Size::U8 => { 
                            ptr_bytes[2] = some_column_schema.0;
                        },
                        NP_Size::U16 => { 
                            ptr_bytes[4] = some_column_schema.0;
                        },
                        NP_Size::U32 => {
                            ptr_bytes[8] = some_column_schema.0;
                        }
                    }
        
                    let addr = memory.malloc(ptr_bytes.to_vec())?;
                    
                    // update head to point to newly created TableItem pointer
                    self.set_head(addr);

                    // provide
                    return Ok(NP_Ptr::_new_table_item_ptr(self.head, some_column_schema.2, &memory));
                } else { // values exist, loop through them to see if we have an existing pointer for this column

                    let mut next_addr = self.head as usize;

                    let mut has_next = true;

                    while has_next {

                        let index = match &memory.size {
                            NP_Size::U8 => { memory.read_bytes()[(next_addr + 2)] },
                            NP_Size::U16 => { memory.read_bytes()[(next_addr + 4)] },
                            NP_Size::U32 => { memory.read_bytes()[(next_addr + 8)] }
                        };
                        
                        // found our value!
                        if index == some_column_schema.0 {
                            return Ok(NP_Ptr::_new_table_item_ptr(next_addr as u32, some_column_schema.2, &memory))
                        }
                        
                        // not found yet, get next address
                        let next_ptr = match memory.size {
                            NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(next_addr + 1).unwrap_or(0)]) as usize,
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
                        NP_Size::U8 => { 
                            ptr_bytes[2] = some_column_schema.0;
                        },
                        NP_Size::U16 => { 
                            ptr_bytes[4] = some_column_schema.0;
                        },
                        NP_Size::U32 => {
                            ptr_bytes[8] = some_column_schema.0;
                        }
                    }
            
                    let addr = memory.malloc(ptr_bytes.to_vec())?;

                    let write_bytes = memory.write_bytes();

                    // set previouse pointer's "next" value to this new pointer
                    match &memory.size {
                        NP_Size::U8 => { 
                            let addr_bytes = (addr as u8).to_be_bytes();
                            write_bytes[(next_addr + 1)] = addr_bytes[0];
                        },
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
                    return Ok(NP_Ptr::_new_table_item_ptr(addr, some_column_schema.2, memory));

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

        let memory = self.memory.unwrap();

        let schema = self.schema.as_ref().unwrap();
        let schema_state = NP_Table::get_schema_state(schema.copy());

        let column_index = schema_state.columns.iter().fold(0u8, |prev, cur| {
            if cur.1 == column {
                return cur.0; 
            }
            prev
        });

        if self.head == 0 { // no values, nothing to delete
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut has_next = true;

            while has_next {

                let index = match &memory.size {
                    NP_Size::U8  => { memory.read_bytes()[(curr_addr + 2)] },
                    NP_Size::U16 => { memory.read_bytes()[(curr_addr + 4)] },
                    NP_Size::U32 => { memory.read_bytes()[(curr_addr + 8)] }
                };
                
                // found our value!
                if index == column_index {

                    let next_pointer: u32 = match &memory.size {
                        NP_Size::U8 => { 
                            let next_pointer_bytes: [u8; 1];

                            match memory.get_1_byte(curr_addr + 1) {
                                Some(x) => {
                                    next_pointer_bytes = [x];
                                },
                                None => {
                                    return Err(NP_Error::new("Out of range request"));
                                }
                            }
                            u8::from_be_bytes(next_pointer_bytes) as u32
                        },
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
                            NP_Size::U8 => {
                                let next_pointer_bytes = (next_pointer as u8).to_be_bytes();
                                memory_bytes[(prev_addr + 1) as usize] = next_pointer_bytes[0];
                            },
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
                    NP_Size::U8 => u8::from_be_bytes([memory.get_1_byte(curr_addr + 1).unwrap_or(0)]) as usize,
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

        let memory = self.memory.unwrap();

        let memory_bytes = memory.write_bytes();
       
        let head_bytes = match memory.size {
            NP_Size::U8 => { 0u8.to_be_bytes().to_vec() },
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
            schema: self.schema
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let memory = self.memory.unwrap();

        let memory_bytes = memory.write_bytes();
       
        let addr_bytes = match memory.size {
            NP_Size::U8 => { (addr as u8).to_be_bytes().to_vec() },
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

        let memory = self.memory.unwrap();

        let schema = self.schema.as_ref().unwrap();
        let schema_state = NP_Table::get_schema_state(schema.copy());

        let column_index = schema_state.columns.iter().fold(0, |prev, cur| {
            if cur.1.as_str() == column { 
                found = true;
                return cur.0; 
            }
            prev
        });

        // no column with this name
        if found == false { return Ok((false, false));};

        // values exist, loop through values to see if we have a matching column

        let mut next_addr = self.head as usize;

        let mut has_next = true;

        while has_next {

            let index = match &memory.size {
                NP_Size::U8 => { memory.read_bytes()[(next_addr + 2)] },
                NP_Size::U16 => { memory.read_bytes()[(next_addr + 4)] },
                NP_Size::U32 => { memory.read_bytes()[(next_addr + 8)] }
            };

            // found our value!
            if index == column_index {
                let value_addr = match &memory.size {
                    NP_Size::U8 => { u8::from_be_bytes([memory.get_1_byte(next_addr).unwrap_or(0)]) as u32 },
                    NP_Size::U16 => { u16::from_be_bytes(*memory.get_2_bytes(next_addr).unwrap_or(&[0; 2])) as u32 },
                    NP_Size::U32 => { u32::from_be_bytes(*memory.get_4_bytes(next_addr).unwrap_or(&[0; 4])) }
                };
                return Ok((true, value_addr != 0));
            }

            
            // not found yet, get next address
            
            next_addr = match &memory.size {
                NP_Size::U8 => { u8::from_be_bytes([memory.get_1_byte(next_addr + 1).unwrap_or(0)]) as usize },
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
pub struct NP_Table_Iterator<'it> {
    address: u32, // pointer location
    head: u32,
    memory: &'it NP_Memory,
    schema: NP_Schema_Ptr<'it>,
    column_index: u8,
    table: NP_Table<'it>
}

impl<'it> NP_Table_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, memory: &'it NP_Memory, schema: NP_Schema_Ptr<'it>) -> Self {
        NP_Table_Iterator {
            address,
            head,
            memory: &memory,
            schema: schema.copy(),
            column_index: 0,
            table: NP_Table::new(address, head, memory, schema.copy())
        }
    }

    /// Convert the iterator back into a table.
    pub fn into_table(self) -> NP_Table<'it> {
        self.table
    }
}

impl<'it> Iterator for NP_Table_Iterator<'it> {
    type Item = NP_Table_Item<'it>;

    fn next(&mut self) -> Option<Self::Item> {
        let schema_state = NP_Table::get_schema_state(self.schema.copy());

        if (self.column_index as usize) < schema_state.columns.len() {

            loop {
                if (self.column_index as usize) >= schema_state.columns.len() {
                    return None;
                }

                let col_data = &schema_state.columns[self.column_index as usize];

                self.column_index += 1;
                let exists = self.table.has(col_data.1.as_str()).unwrap();
                return Some(NP_Table_Item {
                    index: col_data.0,
                    column: col_data.1.clone(),
                    has_value: exists,
                    schema: col_data.2.copy(),
                    table: NP_Table::new(self.address, self.head, &self.memory, self.schema.copy())
                });
                 
            }
        }

        None
    }
}

///  A single item of the table iterator.
#[derive(Debug)]
pub struct NP_Table_Item<'item> { 
    /// The index of this item in the table
    pub index: u8,
    /// The column string name for this index
    pub column: String,
    /// (has pointer at this index, his value at this index)
    pub has_value: (bool, bool),
    /// Schema pointer for this item
    pub schema: NP_Schema_Ptr<'item>,
    table: NP_Table<'item>
}

impl<'item> NP_Table_Item<'item> {
    /// Select the pointer at this iterator
    // TODO: Build a select statement that grabs the current index in place instead of seeking to it.
    pub fn select<X: NP_Value<'item> + Default>(&mut self) -> Result<NP_Ptr<'item, X>, NP_Error> {
        self.table.select(&self.column)
    }
    /// Delete the value and it's pointer at this iterator
    // TODO: Same as select for delete
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.table.delete(&self.column)
    }
}

