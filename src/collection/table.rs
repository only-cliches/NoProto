use alloc::rc::Rc;
use crate::{memory::{blank_ptr_u16_table_item, blank_ptr_u32_table_item, blank_ptr_u8_table_item}, pointer::{NP_Cursor, NP_Cursor_Addr, NP_Cursor_Kinds}, schema::{NP_Parsed_Schema}};
use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};
use core::{result::Result, hint::unreachable_unchecked};
use super::NP_Collection;

/// The data type for tables in NoProto buffers.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Table<'table> {
    remaining_cols: Vec<usize>,
    cursor: NP_Cursor_Addr,
    schema: &'table Box<NP_Parsed_Schema<'table>>,
    current: Option<NP_Cursor_Addr>,
    pub memory: &'table NP_Memory<'table>
}


impl<'table> NP_Table<'table> {

    pub fn cache_table_item<'commit>(cursor_addr: &'commit NP_Cursor_Addr, col_schemas: &'commit Vec<(u8, &str, Box<NP_Parsed_Schema>)>, parent: usize, memory: &'commit NP_Memory<'commit>) -> Result<(), NP_Error> {

        // should never attempt to cache a virtual cursor
        if cursor_addr.is_virtual { panic!() }

        // real table item, (maybe) needs to be cached
        match cursor_addr.get_data(&memory) {
            Ok(_x) => { /* already in cache */ },
            Err(_e) => {
                let mut new_cursor = NP_Cursor::new(cursor_addr.address, Some(parent), memory, &Box::new(NP_Parsed_Schema::None));

                let addr_size = memory.addr_size_bytes();

                new_cursor.parent_addr = Some(parent);
                new_cursor.address_value = memory.read_address(new_cursor.address);
                new_cursor.item_next_addr = Some(memory.read_address(new_cursor.address + addr_size));
                let index = memory.read_bytes()[cursor_addr.address + addr_size + addr_size];
                new_cursor.item_index = Some(index as usize);
                
                new_cursor.kind = NP_Cursor_Kinds::TableItem;

                for col in col_schemas {
                    if col.0 == index {
                        new_cursor.item_key = Some(col.1);
                        new_cursor.schema = &col.2;

                        memory.clone().insert_cache(new_cursor); 
                        return Ok(())
                    }
                }

                panic!()
            }
        }

        Ok(())
    }

    pub fn commit_or_cache_table(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<(), NP_Error> {
        let cursor = cursor_addr.get_data(&memory)?;

        if cursor_addr.is_virtual { // virtual cursor, just return blank details
            cursor.coll_head = Some(0);
            cursor.address_value = 0;
            cursor.kind = NP_Cursor_Kinds::None;
        } else if cursor.address_value == 0 { // real cursor but need to make table
            cursor.coll_head = Some(0);

            // stores HEAD
            cursor.address_value = match memory.size {
                NP_Size::U8 => { memory.malloc_borrow(&[0u8; 1])? },
                NP_Size::U16 => { memory.malloc_borrow(&[0u8; 2])? },
                NP_Size::U32 => { memory.malloc_borrow(&[0u8; 4])? }
            };
            memory.set_value_address(cursor.address, cursor.address_value);

            cursor.kind = NP_Cursor_Kinds::Table;
        } else if cursor.kind == NP_Cursor_Kinds::Standard { // real cursor with value, need to cache table data

            cursor.coll_head = Some(memory.read_address(cursor.address_value));
            cursor.kind = NP_Cursor_Kinds::Table;
        }
        
        Ok(())
    }

    /// Select into pointer
    pub fn select_to_ptr(cursor_addr: NP_Cursor_Addr, memory: &'table NP_Memory, key: &'table str, quick_select: Option<&str>) -> Result<Option<NP_Cursor_Addr>, NP_Error> {
        
        NP_Table::commit_or_cache_table(&cursor_addr, memory)?;

        let table_cursor = cursor_addr.get_data(&memory)?;

        let column_schema = match &**table_cursor.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                columns.iter().fold(None, |prev, item| {
                    if item.1 == key {
                        return Some(item);
                    }
                    return prev;
                })
            },
            _ => { unsafe { unreachable_unchecked() } }
        };


        let column_schema = match column_schema {
            Some(x) => x,
            None => return Ok(None)
        };
        

        let head = table_cursor.coll_head.unwrap();

        // table is empty, return virtual pointer
        if head == 0 {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = &column_schema.2;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::TableItem;
            virtual_cursor.item_index = Some(column_schema.0 as usize);
            virtual_cursor.item_prev_addr = None;
            virtual_cursor.item_next_addr = None;
            virtual_cursor.item_key = Some(column_schema.1);
            virtual_cursor.item_key_addr = None;
            return Ok(Some(NP_Cursor_Addr { address: 0, is_virtual: true}));
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate column
        if let Some(x) = quick_select {
            let virtual_cursor = memory.get_virt_cursor();
            virtual_cursor.address = 0;
            virtual_cursor.address_value = 0;
            virtual_cursor.schema = &column_schema.2;
            virtual_cursor.parent_addr = Some(cursor_addr.address);
            virtual_cursor.kind = NP_Cursor_Kinds::TableItem;
            virtual_cursor.item_index = Some(column_schema.0 as usize);
            virtual_cursor.item_prev_addr = Some(head);
            virtual_cursor.item_next_addr = None;
            virtual_cursor.item_key = Some(column_schema.1);
            virtual_cursor.item_key_addr = None;
            return Ok(Some(NP_Cursor_Addr { address: 0, is_virtual: true}));
        }

        let mut running_ptr: usize = 0;

        // key might be somewhere in existing records
        for item in NP_Table::start_iter(&cursor_addr, memory)? {

            let col_item = memory.get_cursor_data(&item)?;

            if col_item.item_key.unwrap() == key {
                return Ok(Some(item))
            }

            running_ptr = item.address;
        }

        // key not found, make a virutal pointer at the end of the table pointers
        let virtual_cursor = memory.get_virt_cursor();
        virtual_cursor.address = 0;
        virtual_cursor.address_value = 0;
        virtual_cursor.schema = &column_schema.2;
        virtual_cursor.parent_addr = Some(cursor_addr.address);
        virtual_cursor.kind = NP_Cursor_Kinds::TableItem;
        virtual_cursor.item_index = Some(column_schema.0 as usize);
        virtual_cursor.item_prev_addr = Some(running_ptr);
        virtual_cursor.item_next_addr = None;
        virtual_cursor.item_key = Some(column_schema.1);
        virtual_cursor.item_key_addr = None;
        return Ok(Some(NP_Cursor_Addr { address: 0, is_virtual: true}));
    }
}

impl<'value> NP_Value<'value> for NP_Table<'value> {
    fn type_idx() -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        let column_len = bytes[address + 1];

        let mut parsed_columns: Vec<(u8, &str, Box<NP_Parsed_Schema>)> = Vec::new();

        let mut offset = address + 2;

        for x in 0..column_len as usize {
            let col_name_len = bytes[offset] as usize;
            let col_name_bytes = &bytes[(offset + 1)..(offset + 1 + col_name_len)];
            let col_name = unsafe { core::str::from_utf8_unchecked(col_name_bytes) };

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

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let columns: Vec<NP_JSON> = match schema_ptr {
            NP_Parsed_Schema::Table { i: _, columns, sortable: _ } => {
                columns.into_iter().map(|column| {
                    let mut cols: Vec<NP_JSON> = Vec::new();
                    cols.push(NP_JSON::String(column.1.to_string()));
                    cols.push(NP_Schema::_type_to_json(&column.2).unwrap());
                    NP_JSON::Array(cols)
                }).collect()
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("columns".to_owned(), NP_JSON::Array(columns));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {
        Err(NP_Error::new("Type (table) doesn't support .set()!"))
    }
 
    fn get_size(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<usize, NP_Error> {

        if cursor_addr.is_virtual {
            return Ok(0);     
        }

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return Ok(0);
        }

        let base_size = match &memory.size {
            NP_Size::U8  => { 1usize }, // u8 head 
            NP_Size::U16 => { 2usize }, // u16 head 
            NP_Size::U32 => { 4usize }  // u32 head 
        };

        let mut acc_size = 0usize;

        for item in Self::start_iter(&cursor_addr, memory)? {
            acc_size += NP_Cursor::calc_size(item, memory).unwrap(); // item
        }
   
        Ok(base_size + acc_size)
    }

    fn to_json(cursor_addr: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {
        if cursor_addr.is_virtual {
            return NP_JSON::Null;
        }

        let cursor = cursor_addr.get_data(&memory).unwrap();

        if cursor.address_value == 0 {
            return NP_JSON::Null;
        }

        let mut json_map = JSMAP::new();

        for item in Self::start_iter(&cursor_addr, memory).unwrap() {
            let column_data = memory.get_cursor_data(&item).unwrap();
            let column = column_data.item_key.unwrap();
            json_map.insert(String::from(column), NP_Cursor::json_encode(item.clone(), memory));
        }

        NP_JSON::Dictionary(json_map)
    }

    fn do_compact(from_cursor_addr: NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor_addr: NP_Cursor_Addr, to_memory: &'value NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: NP_Value<'value> {

        if from_cursor_addr.address == 0 {
            return Ok(to_cursor_addr);
        }

        NP_Table::commit_or_cache_table(&from_cursor_addr, from_memory).unwrap();
        NP_Table::commit_or_cache_table(&to_cursor_addr, to_memory).unwrap();

        let from_cursor = from_memory.get_cursor_data(&from_cursor_addr)?;


        for old_item in NP_Table::start_iter(&from_cursor_addr, from_memory).unwrap() {
            if old_item.address != 0 { // pointer is not virutal

                let old_cursor = from_memory.get_cursor_data(&old_item)?;
                
                if old_cursor.address_value != 0 { // pointer has value
                    let index = old_cursor.item_key.unwrap();
                    let mut new_item = NP_Table::select_to_ptr(to_cursor_addr.clone(), to_memory, old_cursor.item_key.unwrap(), Some(index))?.unwrap();
                    NP_Table::commit_pointer(&new_item, to_memory)?;
                    NP_Cursor::compact(old_item, from_memory, new_item, to_memory)?;
                }
            }
        }

        Ok(to_cursor_addr)
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "table" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Table as u8);

            let mut column_data: Vec<(String, Vec<u8>)> = Vec::new();

            let mut column_schemas: Vec<(u8, &str, Box<NP_Parsed_Schema>)> = Vec::new();

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
                        column_schemas.push((x, column_name.as_str(), Box::new(column_type.1)));
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

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<&'value Self> {
        None
    }
}



impl<'collection> NP_Collection<'collection> for NP_Table<'collection> {

    fn start_iter(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<Self, NP_Error> {
        NP_Table::commit_or_cache_table(&cursor_addr, memory)?;

        let table_cursor = cursor_addr.get_data(&memory)?;

        let addr_size = memory.addr_size_bytes();

        let mut column_idxs = match &**table_cursor.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                let ids: Vec<usize> = Vec::with_capacity(columns.len());
                for c in columns {
                    ids.push(c.0 as usize);
                }
                ids
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let column_schemas = match &**table_cursor.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                columns
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let head_cursor = NP_Cursor_Addr { address: table_cursor.coll_head.unwrap(), is_virtual: true};

        // nothing in table
        if head_cursor.address == 0 {
            return Ok(NP_Table {
                remaining_cols: column_idxs,
                cursor: cursor_addr.clone(),
                current: None,
                schema: table_cursor.schema,
                memory: memory
            });
        }

        // return head
        NP_Table::cache_table_item(&head_cursor, &column_schemas, cursor_addr.address, memory);

        let head_cursor_data = memory.get_cursor_data(&head_cursor)?;

        column_idxs.retain(|x| { *x != head_cursor_data.item_index.unwrap() });

        return Ok(NP_Table {
            remaining_cols: column_idxs,
            cursor: cursor_addr.clone(),
            current: Some(head_cursor),
            schema: table_cursor.schema,
            memory: memory
        });
    }

    /// Step a pointer to the next item in the collection
    fn step_pointer(&self, cursor_addr: &NP_Cursor_Addr) -> Option<NP_Cursor_Addr> {
        // can't step with virtual pointer
        if cursor_addr.is_virtual {
            return None;
        }

        // save current pointer as previous pointer for next pointer
        let prev_addr = cursor_addr.address;

        let cursor = self.cursor_addr.get_data(&memory).unwrap();

        if cursor.item_next_addr.unwrap() == 0 { // no more pointers
            return None;
        }

        let column_schemas = match &**self.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                columns
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let mut next_cursor = NP_Cursor_Addr { address: cursor.item_next_addr.unwrap(), is_virtual: false};

        NP_Table::cache_table_item(&next_cursor, column_schemas, self.cursor.address, self.memory).unwrap();

        let next_cursor_item = self.memory.get_cursor_data(&next_cursor).unwrap();

        next_cursor_item.item_prev_addr = Some(prev_addr);

        return Some(next_cursor)
    }

    /// Commit a virtual pointer into the buffer
    fn commit_pointer<'mem>(cursor_addr: &NP_Cursor_Addr, memory: &'mem NP_Memory<'mem>) -> Result<NP_Cursor_Addr, NP_Error> {

        // pointer already committed
        if cursor_addr.address != 0 {
            return Ok(cursor_addr.clone());
        }

        if cursor_addr.is_virtual == false { panic!() }

        let cursor = memory.get_virt_cursor();

        let parent_addr = cursor.parent_addr.unwrap();

        let table_cursor = memory.get_cursor_data(&NP_Cursor_Addr { address: parent_addr, is_virtual: false})?;

        cursor.address = match &memory.size {
            NP_Size::U8 => {
                memory.malloc_borrow(&blank_ptr_u8_table_item())?
            },
            NP_Size::U16 => {
                memory.malloc_borrow(&blank_ptr_u16_table_item())?
            },
            NP_Size::U32 => {
                memory.malloc_borrow(&blank_ptr_u32_table_item())?
            }
        };

        let addr_size = memory.addr_size_bytes();

        let prev_addr = cursor.item_prev_addr.unwrap_or(0);

        if table_cursor.coll_head.unwrap() == 0 {
            // no head, make one
            memory.set_value_address(parent_addr, cursor.address);
            table_cursor.coll_head = Some(cursor.address);
        } else if table_cursor.coll_head.unwrap() ==  prev_addr { // inserting in beggining
            // update this pointer's next to old head
            memory.set_value_address(cursor.address + addr_size, table_cursor.coll_head.unwrap());
            cursor.item_next_addr = Some(table_cursor.coll_head.unwrap());

            // update head to this pointer
            table_cursor.coll_head = Some(cursor.address);
            memory.set_value_address(parent_addr, cursor.address);
        } else { // inserting at end
            memory.set_value_address(cursor.item_prev_addr.unwrap() + addr_size, cursor.address);
            memory.get_cursor_data(&NP_Cursor_Addr { address: cursor.item_prev_addr.unwrap(), is_virtual: false})?.item_next_addr = Some(cursor.address);
        }

        Ok(NP_Cursor_Addr { address: cursor.address, is_virtual: false})
    }
}






impl<'it> Iterator for NP_Table<'it> {
    type Item = NP_Cursor_Addr;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining_cols.len() == 0 {
            return None;
        }
        match &mut self.current {
            Some(x) => {
                
                let current = x.clone();
                let cursor_data = self.memory.get_cursor_data(&current).unwrap();
                self.remaining_cols.retain(|&x| x != cursor_data.item_index.unwrap());
                self.current = self.step_pointer(&current);
                Some(current)
            },
            None => {
                match self.remaining_cols.pop() {
                    Some(idx) => {
                        let column = match &**self.schema {
                            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                                
                                let mut col_value = "";
                                for x in columns {
                                    if x.0 as usize == idx {
                                        col_value = x.1;
                                    }
                                }
                                col_value
                            },
                            _ => panic!()
                        };
                        
                        self.current = NP_Table::select_to_ptr(self.cursor.clone(), self.memory, column, Some(column)).unwrap();

                        self.current

                    },
                    None => None
                }
            }
        }

    }

    fn count(self) -> usize where Self: Sized {
        match &**self.schema {
            NP_Parsed_Schema::Table { i: _, sortable: _, columns} => {
                columns.len()
            },
            _ => panic!()
        }
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
    buffer.set(&["name"], String::from("hello"))?;
    assert_eq!(buffer.get::<String>(&["name"])?, Some(Box::new(String::from("hello"))));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}