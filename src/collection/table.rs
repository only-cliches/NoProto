use crate::pointer::DEF_TABLE;
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
pub struct NP_Table {}


impl NP_Table {

    pub fn extend_vtables<'extend>(cursor: &NP_Cursor_Addr, memory: &'extend NP_Memory, col_index: usize) -> Result<&'extend [(usize, &'extend mut NP_Vtable); 64], NP_Error> {
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
                    bytes[index].1.set_next(new_vtable_addr as u16);
                    index +=1;
                    bytes[index] = (new_vtable_addr, unsafe { &mut *(memory.write_bytes().as_ptr().add(new_vtable_addr) as *mut NP_Vtable) })
                }

                Ok(bytes)
            },
            _ => unsafe { unreachable_unchecked() }
        }
    }

    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, parent_schema_addr: usize, memory: &NP_Memory<'parse>, columns: &Vec<(u8, String, usize)>) {

        let table_value = NP_Cursor::parse_cursor_value(buff_addr, parent_addr, parent_schema_addr, &memory);

        let mut new_cursor = NP_Cursor { 
            buff_addr: buff_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: table_value, 
            parent_addr: parent_addr
        };

        let table_addr = new_cursor.value.get_addr_value();

        if table_addr == 0 { // no table here
            memory.insert_parsed(buff_addr, new_cursor);
        } else { // table exists, parse it

            // parse vtables 
            let mut vtables: [(usize, &mut NP_Vtable); 64] = [(0, &mut DEF_TABLE); 64];

            vtables[0] = (table_addr as usize, unsafe { &mut *(memory.write_bytes().as_ptr().add(table_addr as usize) as *mut NP_Vtable) });

            let mut next_vtable = vtables[0].1.get_next();
            let mut index = 1;
            while next_vtable != 0 {
                vtables[index] = (next_vtable as usize, unsafe { &mut *(memory.write_bytes().as_ptr().add(next_vtable as usize) as *mut NP_Vtable) });
                next_vtable = vtables[index].1.get_next();
                index += 1;
            }

            // parse children
            match new_cursor.data {
                NP_Cursor_Data::Table { bytes} => {
                    let mut column_index = 0usize;
                    for vtable in &bytes { // each vtable holds 4 columns
                        if vtable.0 != 0 {
                            for (i, pointer) in vtable.1.values.iter().enumerate() {
                                let item_buff_addr = vtable.0 + (i * 2);
                                let schema_addr = columns[column_index].2;
                                NP_Cursor::parse(item_buff_addr, schema_addr, buff_addr, schema_addr, &memory);
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
 
    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

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

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        for (_i, col, item) in NP_Table::new(cursor.clone(), memory) {
            json_map.insert(String::from(col), NP_Cursor::json_encode(&item, memory));
        }

        NP_JSON::Dictionary(json_map)
    }

    fn do_compact(from_cursor: NP_Cursor_Addr, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor_Addr, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'value + Sized {

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
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // good values are preserved through compaction
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 18usize);

    Ok(())
}