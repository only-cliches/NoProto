use crate::{hashmap::NP_HashMap, utils::{opt_out, opt_out_mut}};
use alloc::string::String;
use crate::pointer::{NP_Cursor_Addr, NP_Cursor_Data, NP_Vtable};
use crate::{pointer::{NP_Cursor}, schema::{NP_Parsed_Schema, NP_Schema_Addr}};
use crate::{memory::{NP_Memory}, pointer::{NP_Value}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{result::Result, hint::unreachable_unchecked};

/// The data type for tables in NoProto buffers.
/// 
#[doc(hidden)]
pub struct NP_Table {
    cursor: NP_Cursor_Addr,
    index: usize
}



impl NP_Table {

    pub fn make_table<'make>(table_cursor_addr: &NP_Cursor_Addr, memory: &'make NP_Memory) -> Result<&'make [(usize, Option<&'make mut NP_Vtable>); 64], NP_Error> {

        let cursor = memory.get_parsed(table_cursor_addr);

        match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Table { columns: column_schemas, ..} => {
                let first_vtable_addr = memory.malloc_borrow(&[0u8; 10])?;
                
                cursor.value.set_addr_value(first_vtable_addr as u16);
                let mut vtables: [(usize, Option<&mut NP_Vtable>); 64] = NP_Vtable::new_empty();
                vtables[0] = (first_vtable_addr as usize, Some(unsafe { &mut *(memory.write_bytes().as_ptr().add(first_vtable_addr as usize) as *mut NP_Vtable) }));
                
                // create cached pointers for vtable
                for x in 0..4usize {
                    if x < column_schemas.len() {
                        let item_addr = vtables[0].0 + (x * 2);
                        memory.insert_parsed(item_addr, NP_Cursor {
                            buff_addr: item_addr, 
                            schema_addr: column_schemas[x].2, 
                            data: NP_Cursor_Data::Empty,
                            temp_bytes: None,
                            value: NP_Cursor::parse_cursor_value(item_addr, cursor.buff_addr, cursor.schema_addr, &memory), 
                            parent_addr: cursor.buff_addr,
                            index: x
                        });
                    }
                }

                cursor.data = NP_Cursor_Data::Table { bytes: vtables };

                Ok(match &cursor.data {
                    NP_Cursor_Data::Table { bytes } => bytes,
                    _ => unsafe { unreachable_unchecked() }
                })
            }
            _ => unsafe { unreachable_unchecked() }
        }
    }

    pub fn new_iter(cursor_addr: &NP_Cursor_Addr) -> Self {

        Self {
            cursor: cursor_addr.clone(),
            index: 0,
        }

    }

    pub fn step_iter<'step>(table: &mut Self, memory: &'step NP_Memory) -> Option<(&'step str, NP_Cursor_Addr)> {
        let table_cursor = memory.get_parsed(&table.cursor);

        match &mut table_cursor.data {
            NP_Cursor_Data::Table { bytes } => {
                match &memory.schema[table_cursor.schema_addr] {
                    NP_Parsed_Schema::Table { columns, .. } => {

                        if columns.len() <= table.index {
                            return None;
                        }

                        let v_table =  table.index / 4; // which vtable
                        let v_table_idx = table.index % 4; // which index on the selected vtable
        
                        if bytes[v_table].0 == 0 { // vtable doesn't exist
                            let virtual_cursor = memory.get_parsed(&NP_Cursor_Addr::Virtual);
                            virtual_cursor.reset();
                            virtual_cursor.parent_addr = table_cursor.buff_addr;
                            virtual_cursor.schema_addr = columns[table.index].2;
                            virtual_cursor.index = table.index;
                            table.index += 1;
                            return Some((columns[table.index - 1].1.as_str(), NP_Cursor_Addr::Virtual))
                        } else { // vtable exists
                            let item_address = bytes[v_table].0 + (v_table_idx * 2);
                            table.index += 1;
                            return Some((columns[table.index - 1].1.as_str(), NP_Cursor_Addr::Real(item_address)))
                        }
                    },
                    _ => unsafe { unreachable_unchecked() }
                }
            },
            _ => unsafe { unreachable_unchecked() }
        }
    }

    pub fn for_each<'each, F>(cursor_addr: &NP_Cursor_Addr, memory: &'each NP_Memory, callback: &mut F) where F: FnMut((&'each str, NP_Cursor_Addr)) {

        let mut list_iter = Self::new_iter(cursor_addr);

        while let Some((index, item)) = Self::step_iter(&mut list_iter, memory) {
            callback((index, item))
        }

    }

    pub fn extend_vtables<'extend>(cursor: &NP_Cursor_Addr, memory: &'extend NP_Memory, col_index: usize) -> Result<&'extend [(usize, Option<&'extend mut NP_Vtable>); 64], NP_Error> {
        let cursor = memory.get_parsed(cursor);

        let desired_v_table =  col_index / 4;

        match &mut cursor.data {
            NP_Cursor_Data::Table { bytes } => {

                let mut index = 0usize;

                // find the last virtual table that has been saved in the buffer
                loop {
                    if bytes[index].0 == 0 {
                        index += 1;
                    } else {
                        break;
                    }
                }

                // extend it
                while index <= desired_v_table {
                    let new_vtable_addr = memory.malloc_borrow(&[0u8; 10])?;
                    opt_out_mut(&mut bytes[index].1).set_next(new_vtable_addr as u16);
                    index +=1;
                    bytes[index] = (new_vtable_addr, Some(unsafe { &mut *(memory.write_bytes().as_ptr().add(new_vtable_addr) as *mut NP_Vtable) }))
                }

                Ok(bytes)
            },
            _ => unsafe { unreachable_unchecked() }
        }
    }

    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, parent_schema_addr: usize, memory: &NP_Memory<'parse>, columns: &Vec<(u8, String, usize)>, index: usize) {

        let table_value = NP_Cursor::parse_cursor_value(buff_addr, parent_addr, parent_schema_addr, &memory);

        let mut new_cursor = NP_Cursor { 
            buff_addr: buff_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: table_value, 
            parent_addr: parent_addr,
            index
        };

        let table_addr = new_cursor.value.get_addr_value();

        if table_addr == 0 { // no table here
            memory.insert_parsed(buff_addr, new_cursor);
        } else { // table exists, parse it

            // parse vtables 
            let mut vtables: [(usize, Option<&mut NP_Vtable>); 64] = NP_Vtable::new_empty();

            vtables[0] = (table_addr as usize, Some(unsafe { &mut *(memory.write_bytes().as_ptr().add(table_addr as usize) as *mut NP_Vtable) }));

            let mut next_vtable = opt_out(&mut vtables[0].1).get_next();
            let mut index = 1;
            while next_vtable != 0 {
                vtables[index] = (next_vtable as usize, Some(unsafe { &mut *(memory.write_bytes().as_ptr().add(next_vtable as usize) as *mut NP_Vtable) }));
                next_vtable = opt_out(&mut vtables[index].1).get_next();
                index += 1;
            }

            // parse children
            match new_cursor.data {
                NP_Cursor_Data::Table { bytes} => {
                    let mut column_index = 0usize;
                    for vtable in &bytes { // each vtable holds 4 columns
                        if vtable.0 != 0 {
                            for (i, pointer) in opt_out(&vtable.1).values.iter().enumerate() {
                                let item_buff_addr = vtable.0 + (i * 2);
                                let schema_addr = columns[column_index].2;
                                NP_Cursor::parse(item_buff_addr, schema_addr, buff_addr, schema_addr, &memory, column_index);
                                column_index += 1;
                            }
                        } else {
                            column_index += 4;
                        }
                    }
                },
                _ => { unsafe { unreachable_unchecked() }}
            }

            // set table data 
            new_cursor.data = NP_Cursor_Data::Table { bytes: vtables };
            memory.insert_parsed(buff_addr, new_cursor);

        }
    }
}

impl<'value> NP_Value<'value> for NP_Table {
    fn type_idx() -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let column_len = bytes[address + 1];

        let mut parsed_columns: Vec<(u8, String,  NP_Schema_Addr)> = Vec::new();

        let table_schema_addr = schema.len();

        schema.push(NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns_mapped: NP_HashMap::new(),
            columns: Vec::new()
        });

        let mut schema_parsed = schema;

        let mut offset = address + 2;

        let mut hash_map = NP_HashMap::new();

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
            hash_map.insert(col_name, x);
            offset += schema_size + 2;
        }

        schema_parsed[table_schema_addr] = NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            columns_mapped: hash_map,
            sortable: false,
            columns: parsed_columns
        };

        (false, schema_parsed)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let columns: Vec<NP_JSON> = match &schema[address] {
            NP_Parsed_Schema::Table { columns, .. } => {
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
 
    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 {
            return Ok(0) 
        }

        let base_size = 0; // head is stored in pointer as value

        let mut acc_size = 0usize;

        Self::for_each(&cursor, memory, &mut |(_i, item)| {
            acc_size += NP_Cursor::calc_size(item.clone(), memory).unwrap();
        });
   
        Ok(base_size + acc_size)
    }

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        Self::for_each(&cursor, memory, &mut |(key, item)| {
            json_map.insert(String::from(key), NP_Cursor::json_encode(item.clone(), memory));     
        });

        NP_JSON::Dictionary(json_map)
    }

    fn do_compact(from_cursor: &NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: Sized {

        let from_c = from_memory.get_parsed(from_cursor);
        
        if from_c.value.get_addr_value() == 0 {
            return Ok(to_cursor);
        }

        Self::make_table(&to_cursor, &to_memory)?;

        let to_c = to_memory.get_parsed(&to_cursor);

        Self::for_each(from_cursor, from_memory, &mut |(key, item)| {
            let old_item = from_memory.get_parsed(&item);

            if old_item.buff_addr != 0 && old_item.value.get_addr_value() != 0 { // pointer has value
                let v_table =  old_item.index / 4; // which vtable
                let v_table_idx = old_item.index % 4; // which index on the selected vtable

                match &to_c.data {
                    NP_Cursor_Data::Table { bytes } => {
                        let mut sel_v_table = &bytes[v_table];

                        if sel_v_table.0 == 0 { // no vtable here, need to make one
                            sel_v_table = &Self::extend_vtables(&to_cursor, to_memory, old_item.index).unwrap()[v_table];
                        }

                        let item_addr = sel_v_table.0 + (v_table_idx * 2);
                        NP_Cursor::compact(&item, from_memory, NP_Cursor_Addr::Real(item_addr), to_memory).unwrap();
                    }
                    _ => unsafe { unreachable_unchecked() }
                }

                
            }    
        });

        Ok(to_cursor)
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::Table as u8);

        let schema_table_addr = schema.len();
        schema.push(NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: Vec::new(),
            columns_mapped: NP_HashMap::new()
        });

        let mut columns_mapped = NP_HashMap::new();

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
                    columns_mapped.insert(column_name.as_str(), x as usize);
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
            columns: columns,
            columns_mapped
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
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 19usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 2usize);

    // good values are preserved through compaction
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 19usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 19usize);

    Ok(())
}