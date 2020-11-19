use crate::{pointer::{NP_Iterator_Helper, NP_Ptr_Collection}, schema::{NP_Parsed_Schema}};
use crate::pointer::NP_PtrKinds;
use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value, NP_Ptr}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use core::{result::Result, hint::unreachable_unchecked};
use core::ops::Add;

use super::NP_Collection;

/// The data type for tables in NoProto buffers. [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug, Clone)]
pub struct NP_Table<'table> {
    address: usize, // pointer location
    head: usize,
    memory: &'table NP_Memory,
    schema: &'table Box<NP_Parsed_Schema>
}

impl<'table> NP_Value<'table> for NP_Table<'table> {
    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Table as u8, "table".to_owned(), NP_TypeKeys::Table) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Table as u8, "table".to_owned(), NP_TypeKeys::Table) }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
        let column_len = bytes[address + 1];

        let mut parsed_columns: Vec<(u8, String, Box<NP_Parsed_Schema>)> = Vec::new();

        let mut offset = address + 2;

        for x in 0..column_len as usize {
            let col_name_len = bytes[offset] as usize;
            let col_name_bytes = &bytes[(offset + 1)..(offset + 1 + col_name_len)];
            let col_name: String = String::from_utf8_lossy(col_name_bytes).into();

            offset += 1 + col_name_len;

            let schema_size = u16::from_be_bytes([
                bytes[offset],
                bytes[offset + 1]
            ]) as usize;

            parsed_columns.push((x as u8, col_name, Box::new(NP_Schema::from_bytes(offset + 2, bytes))));

            offset += schema_size + 2;
        }

        NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: parsed_columns
        }
    }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let columns: Vec<NP_JSON> = match schema_ptr {
            NP_Parsed_Schema::Table { i: _, columns, sortable: _ } => {
                columns.into_iter().map(|column| {
                    let mut cols: Vec<NP_JSON> = Vec::new();
                    cols.push(NP_JSON::String(column.1.clone()));
                    cols.push(NP_Schema::_type_to_json(&column.2).unwrap());
                    NP_JSON::Array(cols)
                }).collect()
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("columns".to_owned(), NP_JSON::Array(columns));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_pointer: &mut NP_Ptr<'table>, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Ptr<'table>) -> Result<Option<Box<Self>>, NP_Error> {

        match &ptr.memory.size {
            NP_Size::U8 => {
                let mut addr = ptr.kind.get_value_addr();

                let mut head: [u8; 1] = [0; 1];

                if addr == 0 {
                    // no table here, make one
                    addr = ptr.memory.malloc([0 as u8; 1].to_vec())?; // stores HEAD
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind); // set pointer to new table
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = [ptr.memory.get_1_byte(a).unwrap_or(0)];
                }

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u8::from_be_bytes(head) as usize,
                    memory: ptr.memory,
                    schema: ptr.schema
                })))
            },
            NP_Size::U16 => {
                let mut addr = ptr.kind.get_value_addr();

                let mut head: [u8; 2] = [0; 2];

                if addr == 0 {
                    // no table here, make one
                    addr = ptr.memory.malloc([0 as u8; 2].to_vec())?; // stores HEAD
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind); // set pointer to new table
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *ptr.memory.get_2_bytes(a).unwrap_or(&[0; 2]);
                }

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u16::from_be_bytes(head) as usize,
                    memory: ptr.memory,
                    schema: ptr.schema
                })))
            },
            NP_Size::U32 => {
                let mut addr = ptr.kind.get_value_addr();

                let mut head: [u8; 4] = [0; 4];

                if addr == 0 {
                    // no table here, make one
                    addr = ptr.memory.malloc([0 as u8; 4].to_vec())?; // stores HEAD
                    ptr.memory.set_value_address(ptr.address, addr, &ptr.kind); // set pointer to new table
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *ptr.memory.get_4_bytes(a).unwrap_or(&[0; 4]);
                }

                Ok(Some(Box::new(NP_Table {
                    address: addr,
                    head: u32::from_be_bytes(head) as usize,
                    memory: ptr.memory,
                    schema: ptr.schema
                })))
            }
        }
    }
 
    fn get_size(ptr: &'table NP_Ptr<'table>) -> Result<usize, NP_Error> {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let base_size = match &ptr.memory.size {
            NP_Size::U8  => 1usize,
            NP_Size::U16 => 2usize,
            NP_Size::U32 => 4usize
        };

        let mut acc_size = 0usize;

        match NP_Table::into_value(ptr.clone())? {
            Some(real_table) => {
                for item in real_table.it().into_iter() {
                    if item.has_value() {
                        acc_size += item.calc_size()?;
                    }
                }
            },
            None => {
                
            }
        }

        Ok(base_size + acc_size)
    }

    fn to_json(ptr: &'table NP_Ptr<'table>) -> NP_JSON {
        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let mut object = JSMAP::new();

        let real_table = NP_Table::into_value(ptr.clone()).unwrap().unwrap();

        for item in real_table.it().into_iter() {
            let column = match item.helper {
                NP_Iterator_Helper::Table {  index: _, column, prev_addr: _, skip_step: _ } => { column },
                _ => panic!()
            };
            object.insert(String::from(column), item.json_encode());
        }

        NP_JSON::Dictionary(object)
    }

    fn do_compact(from_ptr: NP_Ptr<'table>, to_ptr: &'table mut NP_Ptr<'table>) -> Result<(), NP_Error> where Self: NP_Value<'table> {

        if from_ptr.address == 0 {
            return Ok(());
        }

        let old_table = NP_Table::into_value(from_ptr)?.expect("Attempted to cast table from non table pointer!");
        let new_table = NP_Table::into_value(to_ptr.clone())?.expect("Attempted to cast table from non table pointer!");


        for old_item in old_table.it().into_iter() {
            if old_item.has_value() {
                let (idx, column) = match old_item.helper {
                    NP_Iterator_Helper::Table {  index, column, prev_addr: _, skip_step: _ } => { (index, column) },
                    _ => panic!()
                };
                let mut new_ptr = new_table.select(column, Some(idx as usize));
                new_ptr = NP_Table::commit_pointer(new_ptr)?;
                old_item.clone().compact(&mut new_ptr)?;
            }
        }

        Ok(())
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "table" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Table as u8);

            let mut column_data: Vec<(String, Vec<u8>)> = Vec::new();

            let mut column_schemas: Vec<(u8, String, Box<NP_Parsed_Schema>)> = Vec::new();

            match &json_schema["columns"] {
                NP_JSON::Array(cols) => {
                    let mut x: u8 = 0;
                    for col in cols {
                        let column_name = match &col[0] {
                            NP_JSON::String(x) => x.clone(),
                            _ => "".to_owned()
                        };
                        if column_name.len() > 255 {
                            return Err(NP_Error::new("Table column names cannot be longer than 255 characters!"))
                        }
 
                        let column_type = NP_Schema::from_json(Box::new(col[1].clone()))?;
                        column_data.push((column_name.clone(), column_type.0));
                        column_schemas.push((x, column_name, Box::new(column_type.1)));
                        x += 1;
                    }
                },
                _ => { 
                    return Err(NP_Error::new("Tables require a 'columns' property that is an array of schemas!"))
                }
            }

            if column_data.len() > 255 {
                return Err(NP_Error::new("Tables cannot have more than 255 columns!"))
            }

            if column_data.len() == 0 {
                return Err(NP_Error::new("Tables must have at least one column!"))
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


            return Ok(Some((schema_data, NP_Parsed_Schema::Table {
                i: NP_TypeKeys::Table,
                sortable: false,
                columns: column_schemas
            })))
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }
}

impl<'table> NP_Table<'table> {

    #[doc(hidden)]
    pub fn new(address: usize, head: usize, memory: &'table NP_Memory, schema: &'table Box<NP_Parsed_Schema>) -> Self {
        NP_Table {
            address,
            head,
            memory: memory,
            schema: schema
        }
    }

    /// read schema of table
    pub fn get_schema(&self) -> &'table Box<NP_Parsed_Schema> {
        self.schema
    }

    /// Convert the table into an iterator.  Allows you to loop through all the values present in the table.
    /// 
    pub fn it(self) -> NP_Table_Iterator<'table> {
        NP_Table_Iterator::new(self)
    }

    /// Select a specific value at the given key
    pub fn select(&'table self, column: &str, quick_select: Option<usize>) -> NP_Ptr<'table> {
        NP_Table::select_mv(self.clone(), column, quick_select)
    }

    /// Select a specific value at the given key
    pub fn select_mv(self, column: &str, quick_select: Option<usize>) -> NP_Ptr<'table> {
        let columns = match &**self.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns } => {
                columns
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let column_schema = columns.iter().fold(None, |prev, item| {
            if item.1 == column {
                Some(item)
            } else {
                prev
            }
        }).unwrap();

        // table is empty, return virtual pointer
        if self.head == 0 {
            return NP_Ptr::_new_collection_item_ptr(0, &column_schema.2, &self.memory, NP_Ptr_Collection::Table {
                address: self.address,
                head: self.head,
                schema: self.schema
            }, NP_Iterator_Helper::Table {
                index: column_schema.0,
                column: &column_schema.1,
                prev_addr: 0,
                skip_step: false
            })
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate column
        if let Some(x) = quick_select {
            return NP_Ptr::_new_collection_item_ptr(0, &columns[x].2, &self.memory, NP_Ptr_Collection::Table {
                address: self.address,
                head: self.head,
                schema: self.schema
            }, NP_Iterator_Helper::Table {
                index: columns[x].0,
                column: &columns[x].1,
                prev_addr: self.head,
                skip_step: false
            });
        }

        let mut running_ptr: usize = 0;

        // key might be somewhere in existing records
        for item in self.clone().it().into_iter() {

            let this_column = match item.helper {
                NP_Iterator_Helper::Table { index: _, column, prev_addr: _, skip_step: _} => column,
                _ => panic!()
            };
            
            // found matched key
            if this_column == column {
                return item.clone();
            }

            running_ptr = item.address;
        }


        // key not found, make a virutal pointer at the end of the table pointers
        return NP_Ptr::_new_collection_item_ptr(0, &column_schema.2, &self.memory, NP_Ptr_Collection::Table {
            address: self.address,
            head: self.head,
            schema: self.schema
        }, NP_Iterator_Helper::Table {
            index: column_schema.0,
            column: &column_schema.1,
            prev_addr: running_ptr,
            skip_step: false
        })
    }

    /// Check to see if a specific column value has been set in this table.
    /// 
    /// The first bool is if a pointer exists for this column, the second bool is if there is a value set on that pointer.
    /// 
    pub fn has(&self, column: &str) -> bool {
        if self.head == 0 { // empty list, nothing to delete
            false
        } else {
            self.select(column, None).has_value()
        }
    }

}


impl<'collection> NP_Collection<'collection> for NP_Table<'collection> {

    /// Get length of collection
    fn length(&self) -> usize {
        match &**self.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns } => {
                return columns.len()
            },
            _ => panic!()
        }
    }

    /// Step a pointer to the next item in the collection
    fn step_pointer(ptr: &mut NP_Ptr<'collection>) -> Option<NP_Ptr<'collection>> {
        // can't step with virtual pointer
        if ptr.address == 0 {
            return None;
        }

        let skip_step = match ptr.helper {
            NP_Iterator_Helper::Table { index: _, column: _, skip_step, prev_addr: _} => {
                skip_step
            },
            _ => panic!()
        };

        if skip_step == true {
            return None;
        }

        let addr_size = ptr.memory.addr_size_bytes();

        // save current pointer as previous pointer for next pointer
        let prev_addr = ptr.address;

        // get address for next pointer
        let curr_addr = ptr.memory.read_address(ptr.address + addr_size);

        if curr_addr == 0 { // no more pointers
            return None;
        }

        let current_index = ptr.memory.read_bytes()[curr_addr + addr_size + addr_size];

        let (column, schema) = match ptr.parent {
            NP_Ptr_Collection::Table { address: _, head: _, schema} => {
                match &**schema {
                    NP_Parsed_Schema::Table { i: _, sortable: _, columns } => {
                        let idx = current_index as usize;
                        (&columns[idx].1, &columns[idx].2)
                    },
                    _ => panic!()
                }
            },
            _ => panic!()
        };


        // provide next pointer
        Some(NP_Ptr::_new_collection_item_ptr(curr_addr, schema, ptr.memory, ptr.parent.clone(), NP_Iterator_Helper::Table {
            prev_addr,
            index: current_index,
            column: column,
            skip_step: false
        }))
    }

    /// Commit a virtual pointer into the buffer
    fn commit_pointer(ptr: NP_Ptr<'collection>) -> Result<NP_Ptr<'collection>, NP_Error> {

        // pointer already committed
        if ptr.address != 0 {
            return Ok(ptr);
        }

        match ptr.helper {
            NP_Iterator_Helper::Table { index, column, prev_addr, skip_step } => {
                let (mut head, table_address, schema) = match ptr.parent { NP_Ptr_Collection::Table { head, address, schema } => { (head, address, schema) }, _ => panic!()};

                let mut ptr_bytes: Vec<u8> = ptr.memory.blank_ptr_bytes(&NP_PtrKinds::TableItem { addr: 0, next: 0, i: 0}); 

                let addr_size = ptr.memory.addr_size_bytes();

                // set index in table item pointer
                ptr_bytes[addr_size + addr_size] = index;

                // write to buffer
                let new_addr = ptr.memory.malloc(ptr_bytes)?;
               

                if head == 0 { // empty map
                    // set head to this new pointer
                    head = new_addr;
                    ptr.memory.write_address(table_address, new_addr)?;
                } else { // table has existing values

                    if prev_addr == head { // inserting in beggining
                        // update this poitner's "next" to old head
                        ptr.memory.write_address(new_addr + addr_size, head)?;
                        // update head to this new pointer
                        head = new_addr;
                        ptr.memory.write_address(table_address, new_addr)?;
                    } else { // inserting at end
                        // update previous pointer "next" to this new pointer
                        ptr.memory.write_address(prev_addr + addr_size, new_addr)?;
                    }
                }

                Ok(NP_Ptr::_new_collection_item_ptr(new_addr, ptr.schema, ptr.memory, NP_Ptr_Collection::Table {
                    address: table_address,
                    head: head,
                    schema: schema
                }, NP_Iterator_Helper::Table {
                    index: index,
                    column: column,
                    prev_addr: prev_addr,
                    skip_step: skip_step
                }))
            },
            _ => panic!()
        }
    }
}




impl<'it> NP_Table_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(table: NP_Table<'it>) -> Self {
        let column_schemas = match &**table.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                columns
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let memory = table.memory;

        let addr_size = memory.addr_size_bytes();

        // Check if there's a pointer in the map, if so use it as the first element in the loop
        let (addr, prev_addr, head_idx) = if table.head != 0 { // tabl has items
            let head_idx = memory.read_bytes()[table.head + addr_size + addr_size];
            (table.head, 0, head_idx)
        } else { // empty table, everything is virtual
            (0, 0, 0)
        };

        let head_schema = &column_schemas[head_idx as usize];

        let remaining_idxs: Vec<&(u8, String, Box<NP_Parsed_Schema>)> = column_schemas.iter().filter_map(|x| {
            if x.0 == head_schema.0 {
                None
            } else {
                Some(x)
            }
        }).collect();

        // make first initial pointer
        NP_Table_Iterator {
            parent: NP_Ptr_Collection::Table {
                address: table.address,
                head: table.head,
                schema: table.schema
            },
            memory: &memory,
            remaining_idxs,
            current: Some(NP_Ptr::_new_collection_item_ptr(addr, &head_schema.2, &memory, NP_Ptr_Collection::Table {
                address: table.address,
                head: table.head,
                schema: table.schema
            }, NP_Iterator_Helper::Table {
                prev_addr,
                index: head_idx,
                column: &head_schema.1,
                skip_step: false
            }))
        }
    }
}

/// The iterator type for maps
#[derive(Debug)]
pub struct NP_Table_Iterator<'it> {
    parent: NP_Ptr_Collection<'it>,
    memory: &'it NP_Memory,
    remaining_idxs: Vec<&'it (u8, String, Box<NP_Parsed_Schema>)>,
    current: Option<NP_Ptr<'it>>
}

impl<'it> Iterator for NP_Table_Iterator<'it> {
    type Item = NP_Ptr<'it>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_idxs.len() == 0 {
            return None;
        }
        match &mut self.current {
            Some(x) => {
                let current = x.clone();
                let idx = match &x.helper {
                    NP_Iterator_Helper::Table {  index, column: _, prev_addr: _, skip_step: _ } => { index },
                    _ => panic!()
                };
                self.parent = current.parent.clone();
                self.remaining_idxs.retain(|&x| x.0 != *idx);
                self.current = NP_Table::step_pointer(x);
                Some(current)
            },
            None => {
                
                match self.remaining_idxs.pop() {
                    Some((idx, col, schema)) => {

                        let head = match self.parent { NP_Ptr_Collection::Table { head, address: _, schema: _ } => { head }, _ => panic!()};

                        let current = NP_Ptr::_new_collection_item_ptr(0, schema, self.memory, self.parent.clone(), NP_Iterator_Helper::Table {
                            prev_addr: head,
                            index: *idx,
                            column: col,
                            skip_step: true
                        });
                        self.current = Some(current.clone());
                        Some(current)
                    },
                    None => None
                }
            }
        }


    }

    fn count(self) -> usize where Self: Sized {
        #[inline]
        fn add1<T>(count: usize, _: T) -> usize {
            // Might overflow.
            Add::add(count, 1)
        }

        self.fold(0, add1)
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"table\",\"columns\":[[\"age\",{\"type\":\"uint8\"}],[\"name\",{\"type\":\"string\",\"size\":10}]]}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"table\",\"columns\":[[\"age\",{\"type\":\"uint8\"}],[\"name\",{\"type\":\"string\"}]]}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set("name", String::from("hello"))?;
    assert_eq!(buffer.get::<String>("name")?, Some(Box::new(String::from("hello"))));
    buffer.del("")?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}