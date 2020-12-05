use alloc::string::String;
use crate::pointer::{NP_Cursor_Parent, NP_Cursor_Value};
use crate::{pointer::{NP_Cursor}, schema::{NP_Parsed_Schema, NP_Schema_Addr}};
use crate::{memory::{NP_Size, NP_Memory}, pointer::{NP_Value}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{result::Result, hint::unreachable_unchecked};

/// The data type for tables in NoProto buffers.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Table<'table> {
    cursor: NP_Cursor,
    table: NP_Cursor_Parent,
    current: Option<(usize, &'table String, NP_Cursor)>,
    pub memory: &'table NP_Memory<'table>,
    remaining_cols: [bool; 256],
    col_step: usize,
    col_length: usize,
}

fn pop_cols(cols: &[bool; 256], index: usize, length: usize) -> Option<usize> {
    // end of cols
    if length == index { return None }; 

    let value = cols[index];

    // already visited
    if value == true { return pop_cols(cols, index + 1, length) }; 

    return Some(index);
}

impl<'table> NP_Table<'table> {

    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, memory: &NP_Memory<'parse>) {

    }

    /// Create new table iterator
    ///
    #[inline(always)]
    pub fn new(mut cursor: NP_Cursor, memory: &'table NP_Memory<'table>) -> Self {
        let value_addr = if cursor.buff_addr != 0 { memory.read_address(cursor.buff_addr) } else { 0 };
        cursor.value = cursor.value.update_value_address(value_addr);

        let (cols_idx, cols_len) = match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Table { columns, .. } => {
                ([false; 256], columns.len())
            },
            _ => { unsafe {  unreachable_unchecked() } }
        };

        Self {
            cursor: cursor,
            table: NP_Cursor_Parent::Table {
                head: memory.read_address(value_addr),
                addr: value_addr,
                schema_addr: cursor.schema_addr
            },
            remaining_cols: cols_idx,
            current: None,
            col_length: cols_len,
            col_step: 0,
            memory: memory
        }
    }

    /// Read or save table into buffer
    /// 
    #[inline(always)]
    pub fn read_table(buff_addr: usize, schema_addr: usize, memory: &NP_Memory<'table>, create: bool) -> Result<(NP_Cursor, usize), NP_Error> {
        let mut cursor = NP_Cursor::new(buff_addr, schema_addr, &memory, NP_Cursor_Parent::None);
        let mut value_addr = cursor.value.get_value_address();
        
        if value_addr == 0 { // no table here
            if create { // please make one
                assert_ne!(cursor.buff_addr, 0); 
                value_addr = match memory.size { // stores HEAD for table
                    NP_Size::U8 => {  memory.malloc_borrow(&[0u8; 1])? },
                    NP_Size::U16 => { memory.malloc_borrow(&[0u8; 2])? },
                    NP_Size::U32 => { memory.malloc_borrow(&[0u8; 4])? }
                };
                // update buffer
                memory.write_address(cursor.buff_addr, value_addr);
                // update cursor
                cursor.value = cursor.value.update_value_address(value_addr);
                Ok((cursor, 0))
            } else { // no table and no need to make one, just pass empty data
                Ok((cursor, 0))       
            }
        } else { // table found, read info from buffer
            Ok((cursor, memory.read_address(value_addr)))
        }
    }


    /// Commit a virtual cursor into the buffer
    /// 
    #[inline(always)]
    pub fn commit_virtual_cursor<'commit>(mut cursor: NP_Cursor, memory: &'commit NP_Memory<'commit>) -> Result<NP_Cursor, NP_Error> {

        if cursor.buff_addr != 0 {
            return Ok(cursor)
        };

        let addr_size = memory.addr_size_bytes();

        cursor.buff_addr = memory.malloc_cursor(&cursor.value)?;

        let index = match cursor.value {
            NP_Cursor_Value::TableItem { index , .. } => { index },
            _ => { unsafe { unreachable_unchecked() }}
        };

        match cursor.parent {
            NP_Cursor_Parent::Table { head: _ , schema_addr: _, addr } => {

                if let Some(prev) = cursor.prev_cursor { // update previous cursor to point to this one
                    memory.write_address(prev + addr_size, cursor.buff_addr);
                } else { // update head to point to this cursor
                    // head = cursor.buff_addr;
                    memory.write_address(addr, cursor.buff_addr);
                }

                // write index in buffer
                memory.write_bytes()[cursor.buff_addr + addr_size + addr_size] = index as u8;

                return Ok(cursor);
            },
            _ => { unsafe { unreachable_unchecked() }}
        }
    }

    /// Select into pointer
    #[inline(always)]
    pub fn select_into(cursor: NP_Cursor, memory: &'table NP_Memory, col: &'table str, create_path: bool, quick_select: bool) -> Result<Option<NP_Cursor>, NP_Error> {

        let addr_size = memory.addr_size_bytes();

        let (table_cursor, mut head) = Self::read_table(cursor.buff_addr, cursor.schema_addr, &memory, create_path)?;

        let table_value_addr = table_cursor.value.get_value_address();

        let (col_schema_addr, col_index) = match &memory.schema[table_cursor.schema_addr] {
            NP_Parsed_Schema::Table { columns, ..} => {
                columns.iter().fold((0usize, 0usize), |prev, cur| {
                    if cur.1.as_str() == col {
                        (cur.2, cur.0.into())
                    } else {
                        prev
                    }
                })
            },
            _ => { unsafe { unreachable_unchecked()} }
        };

        if col_schema_addr == 0 {
            return Ok(None);
        }

        // table is empty
        if head == 0 {

            let mut virtual_cursor = NP_Cursor::new(0, col_schema_addr, memory, NP_Cursor_Parent::Table { head: head, addr: table_cursor.buff_addr, schema_addr: table_cursor.schema_addr });
            virtual_cursor.value = NP_Cursor_Value::TableItem { value_addr: 0, index: col_index, next: 0 };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // update head 
                head = virtual_cursor.buff_addr;
                memory.write_address(table_value_addr, head);

                // write index in buffer
                virtual_cursor.value = NP_Cursor_Value::TableItem { index: col_index,  value_addr: 0, next: 0 };
                memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size] = col_index as u8;
            }

            virtual_cursor.parent = NP_Cursor_Parent::Table { head: head, addr: table_cursor.buff_addr, schema_addr: table_cursor.schema_addr };
            return Ok(Some(virtual_cursor))
        }

        // this ONLY works if we KNOW FOR SURE that the key we're inserting is not a duplicate key
        if quick_select {
            let mut virtual_cursor = NP_Cursor::new(0, col_schema_addr, memory, NP_Cursor_Parent::Table { head, addr: table_cursor.buff_addr , schema_addr: table_cursor.schema_addr});
            virtual_cursor.value = NP_Cursor_Value::TableItem { value_addr: 0, index: col_index, next: 0 };

            if create_path {
                virtual_cursor.buff_addr = memory.malloc_cursor(&virtual_cursor.value)?;

                // update NEXT to old head
                memory.write_address(virtual_cursor.buff_addr + addr_size, head);
                let next_addr = head;

                // update head 
                head = virtual_cursor.buff_addr;
                memory.write_address(table_value_addr, head);

                // write index in buffer
                virtual_cursor.value = NP_Cursor_Value::TableItem { index: col_index,  value_addr: 0, next: next_addr };
                memory.write_bytes()[virtual_cursor.buff_addr + addr_size + addr_size] = col_index as u8;
            }

            return Ok(Some(virtual_cursor))
        }

        for (_ikey, icol, item) in NP_Table::new(table_cursor.clone(), memory) {
            if col == icol {
                if create_path {
                    return Ok(Some(NP_Table::commit_virtual_cursor(item.clone(), memory)?))
                } else {
                    return Ok(Some(item.clone()))
                }
            }
        }


       return Ok(None);
    }
}

impl<'value> NP_Value<'value> for NP_Table<'value> {
    fn type_idx() -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let column_len = bytes[address + 1];

        let mut parsed_columns: Vec<(u8, String,  NP_Schema_Addr)> = Vec::new();

        let table_schema_addr = schema.len();

        schema.push(NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: Vec::new()
        });

        let mut schema_parsed = schema;

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

            let column_addr = schema_parsed.len();
            let (_, schema) = NP_Schema::from_bytes(schema_parsed, offset + 2, bytes);
            schema_parsed = schema;
            parsed_columns.push((x as u8, col_name.to_string(), column_addr));

            offset += schema_size + 2;
        }

        schema_parsed[table_schema_addr] = NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: parsed_columns
        };

        (false, schema_parsed)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let columns: Vec<NP_JSON> = match &schema[address] {
            NP_Parsed_Schema::Table { i: _, columns, sortable: _ } => {
                columns.into_iter().map(|column| {
                    let mut cols: Vec<NP_JSON> = Vec::new();
                    cols.push(NP_JSON::String(column.1.to_string()));
                    cols.push(NP_Schema::_type_to_json(&schema, column.2).unwrap());
                    NP_JSON::Array(cols)
                }).collect()
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("columns".to_owned(), NP_JSON::Array(columns));

        Ok(NP_JSON::Dictionary(schema_json))
    }
 
    fn get_size(cursor: NP_Cursor, memory: &NP_Memory) -> Result<usize, NP_Error> {

        if cursor.value.get_value_address() == 0 {
            return Ok(0) 
        }


        let base_size = match &memory.size {
            NP_Size::U8  => { 1usize }, // u8 head 
            NP_Size::U16 => { 2usize }, // u16 head 
            NP_Size::U32 => { 4usize }  // u32 head 
        };

        let mut acc_size = 0usize;

        for (_i, _col, item) in NP_Table::new(cursor.clone(), memory) {
            acc_size += NP_Cursor::calc_size(item.clone(), memory).unwrap(); // item
        }
   
        Ok(base_size + acc_size)
    }

    fn to_json(cursor: &NP_Cursor, memory: &NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        for (_i, col, item) in NP_Table::new(cursor.clone(), memory) {
            json_map.insert(String::from(col), NP_Cursor::json_encode(&item, memory));
        }

        NP_JSON::Dictionary(json_map)
    }

    fn do_compact(from_cursor: &NP_Cursor, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor, NP_Error> where Self: 'value {

        if from_cursor.buff_addr == 0 {
            return Ok(to_cursor);
        }

        for (_i, col, old_item) in NP_Table::new(from_cursor.clone(), from_memory) {
            if old_item.buff_addr != 0 && old_item.value.get_value_address() != 0 { // pointer has value
 
                let new_item = NP_Table::select_into(to_cursor.clone(), to_memory, col, true, true)?.unwrap();
                NP_Cursor::compact(&old_item, from_memory, new_item, to_memory)?;
            }
        }

        Ok(to_cursor)
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::Table as u8);

        let schema_table_addr = schema.len();
        schema.push(NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: Vec::new()
        });

        let mut columns: Vec<(u8, String, NP_Schema_Addr)> = Vec::new();

        let mut column_data: Vec<(String, Vec<u8>)> = Vec::new();

        let mut schema_parsed: Vec<NP_Parsed_Schema> = schema;

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

                    let column_schema_addr = schema_parsed.len();
                    columns.push((x, column_name.clone(), column_schema_addr));
                    let (_is_sortable, column_type, schema_p) = NP_Schema::from_json(schema_parsed, &Box::new(col[1].clone()))?;
                    schema_parsed = schema_p;
                    column_data.push((column_name, column_type));
                    x += 1;
                }
            },
            _ => { 
                return Err(NP_Error::new("Tables require a 'columns' property that is an array of schemas!"))
            }
        }

        schema_parsed[schema_table_addr] = NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: columns
        };

        if column_data.len() > 255 {
            return Err(NP_Error::new("Tables cannot have more than 255 columns!"))
        }

        if column_data.len() == 0 {
            return Err(NP_Error::new("Tables must have at least one column!"))
        }

        // number of columns
        schema_bytes.push(column_data.len() as u8);

        for col in column_data {
            // colum name
            let bytes = col.0.as_bytes().to_vec();
            schema_bytes.push(bytes.len() as u8);
            schema_bytes.extend(bytes);

            if col.1.len() > u16::max as usize {
                return Err(NP_Error::new("Schema overflow error!"))
            }
            
            // column type
            schema_bytes.extend((col.1.len() as u16).to_be_bytes().to_vec());
            schema_bytes.extend(col.1);
        }

        return Ok((false, schema_bytes, schema_parsed))
   
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }
}


impl<'it> Iterator for NP_Table<'it> {
    type Item = (usize, &'it String, NP_Cursor);

    fn next(&mut self) -> Option<Self::Item> {

        let addr_size = self.memory.addr_size_bytes();

        if let Some(current) = self.current { // step pointer
    
            if let Some(next) = current.2.next_cursor {
                let mut next_cursor = NP_Cursor::new(next, current.2.schema_addr, &self.memory, current.2.parent.clone());

                let (next_next_addr, col_index) = match next_cursor.value {
                    NP_Cursor_Value::TableItem { next, index, ..} => { (next, index) },
                    _ => { unsafe { unreachable_unchecked() } }
                };

                let (col_key, col_schema_addr)= match &self.memory.schema[self.cursor.schema_addr] {
                    NP_Parsed_Schema::Table { columns, .. } => {
                        (&columns[col_index].1, &columns[col_index].2)
                    },
                    _ => { unsafe { unreachable_unchecked() } }
                };

                next_cursor.schema_addr = *col_schema_addr;

                next_cursor.prev_cursor = Some(current.2.buff_addr);

                if next_next_addr == 0 {
                    next_cursor.next_cursor = None;
                } else {
                    next_cursor.next_cursor = Some(next_next_addr);
                }

                self.remaining_cols[col_index] = true;
                
                self.current = Some((col_index, col_key, next_cursor));

                self.current
            } else { // no columns with pointers left in table, loop through virtual

                match pop_cols(&self.remaining_cols, self.col_step, self.col_length) {
                    Some(col_step) => {
                        self.col_step = col_step;

                        let (col_index, col_key, col_schema_addr) = match &self.memory.schema[self.cursor.schema_addr] {
                            NP_Parsed_Schema::Table { columns, .. } => {
                                &columns[self.col_step as usize]
                            },
                            _ => { unsafe { unreachable_unchecked() } }
                        };
                        

                        let mut virtual_cursor = NP_Cursor::new(0, *col_schema_addr, &self.memory, current.2.parent.clone());
                        virtual_cursor.value = NP_Cursor_Value::TableItem {value_addr: 0, index: *col_index as usize, next: 0 };
                        virtual_cursor.prev_cursor = if current.2.buff_addr != 0 {
                            Some(current.2.buff_addr)
                        } else {
                            current.2.prev_cursor
                        };

                        self.col_step += 1;
                        
                        self.current = Some(((*col_index) as usize, col_key, virtual_cursor));
                        
                        self.current
                    },
                    None => None
                }
            }

        } else { // make first pointer
            
            let (table_cursor, head) = Self::read_table(self.cursor.buff_addr, self.cursor.schema_addr, self.memory, true).unwrap();
            
            // nothing here bro
            if head == 0 {
                return None;
            }

            let first_cursor_index = self.memory.read_bytes()[head + addr_size + addr_size] as usize;

            let (col_index, col_key, col_schema_addr) = match &self.memory.schema[self.cursor.schema_addr] {
                NP_Parsed_Schema::Table { columns, .. } => {
                    &columns[first_cursor_index]
                },
                _ => { unsafe { unreachable_unchecked() } }
            };

            let mut first_cursor = NP_Cursor::new(head, *col_schema_addr, &self.memory, NP_Cursor_Parent::Table { addr: table_cursor.buff_addr, head, schema_addr: self.cursor.schema_addr });
       
            match first_cursor.value {
                NP_Cursor_Value::TableItem { next, ..} => {
                    if next != 0 {
                        first_cursor.next_cursor = Some(next);
                    }
                },
                _ => { unsafe { unreachable_unchecked() }}
            }

            self.remaining_cols[*col_index as usize] = true;
            
            self.current = Some(((*col_index).into(), col_key, first_cursor));

            self.current
        }
    }

    fn count(self) -> usize where Self: Sized {
        match &self.memory.schema[self.cursor.schema_addr] {
            NP_Parsed_Schema::Table { columns, ..} => {
                columns.len()
            },
            _ => { unsafe { unreachable_unchecked() }}
        }
    }
}


#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"table\",\"columns\":[[\"age\",{\"type\":\"uint8\"}],[\"tags\",{\"type\":\"list\",\"of\":{\"type\":\"string\"}}],[\"name\",{\"type\":\"string\",\"size\":10}]]}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"table\",\"columns\":[[\"age\",{\"type\":\"uint8\"}],[\"name\",{\"type\":\"string\"}]]}";
    let factory = crate::NP_Factory::new(schema)?;

    // compaction removes cleared values
    let mut buffer = factory.empty_buffer(None, None)?;
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // good values are preserved through compaction
    let mut buffer = factory.empty_buffer(None, None)?;
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);
    buffer.compact(None, None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);

    Ok(())
}