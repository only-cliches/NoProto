use crate::buffer::{VTABLE_BYTES, VTABLE_SIZE};
use alloc::string::String;
use crate::pointer::{NP_Vtable};
use crate::{pointer::{NP_Cursor}, schema::{NP_Parsed_Schema, NP_Schema_Addr}};
use crate::{memory::{NP_Memory}, pointer::{NP_Value}, error::NP_Error, schema::{NP_Schema, NP_TypeKeys}, json_flex::{JSMAP, NP_JSON}};

use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use core::{result::Result};

/// The data type for tables in NoProto buffers.
/// 
#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Table<'table> {
    index: usize,
    v_table: Option<&'table mut NP_Vtable>,
    v_table_addr: usize,
    v_table_index: usize,
    table: NP_Cursor
}

#[allow(missing_docs)]
impl<'table> NP_Table<'table> {

    #[inline(always)]
    pub fn select<M: NP_Memory>(mut table_cursor: NP_Cursor, columns: &Vec<(u8, String, usize)>,  key: &str, make_path: bool, memory: &M) -> Result<Option<NP_Cursor>, NP_Error> {
       
        match columns.iter().position(|val| { val.1 == key }) {
            Some(x) => {

                let v_table =  x / VTABLE_SIZE; // which vtable
                let v_table_idx = x % VTABLE_SIZE; // which index on the selected vtable

                let mut table_value = table_cursor.get_value(memory);

                if table_value.get_addr_value() == 0 {
                    if make_path {
                        table_cursor = Self::make_first_vtable(table_cursor, memory)?;

                        table_value = table_cursor.get_value(memory);
                    } else {
                        return Ok(None);
                    }
                }

                let mut seek_vtable = 0usize;
                let mut vtable_address = table_value.get_addr_value() as usize;

                if v_table > 0 {
                    let mut loop_max = 64usize;
                    while seek_vtable < v_table && loop_max > 0 {
                        let this_vtable = Self::get_vtable(vtable_address, memory);
                        let next_vtable = this_vtable.get_next();

                        if next_vtable == 0 {
                            vtable_address = Self::make_next_vtable(this_vtable, memory)?;
                        } else {
                            vtable_address = next_vtable as usize;
                        }

                        seek_vtable += 1;
                        loop_max -= 1;
                    }
                }

                let item_address = vtable_address + (v_table_idx * 2);

                Ok(Some(NP_Cursor::new(item_address, columns[x].2, table_cursor.schema_addr)))
            },
            None => Ok(None)
        }
    }

    #[inline(always)]
    pub fn make_first_vtable<'make, M: NP_Memory>(table_cursor: NP_Cursor, memory: &'make M) -> Result<NP_Cursor, NP_Error> {

        let first_vtable_addr = memory.malloc_borrow(&[0u8; VTABLE_BYTES])?;
        
        let table_value = table_cursor.get_value(memory);
        table_value.set_addr_value(first_vtable_addr as u16);

        Ok(table_cursor)
    }

    #[inline(always)]
    pub fn make_next_vtable<'make, M: NP_Memory>(prev_vtable: &'make mut NP_Vtable, memory: &'make M) -> Result<usize, NP_Error> {

        let vtable_addr = memory.malloc_borrow(&[0u8; VTABLE_BYTES])?;
        
        prev_vtable.set_next(vtable_addr as u16);

        Ok(vtable_addr)
    }

    #[inline(always)]
    pub fn new_iter<M: NP_Memory>(cursor: &NP_Cursor, memory: &'table M) -> Self {

        let table_value = cursor.get_value(memory);

        let addr_value = table_value.get_addr_value() as usize;

        Self {
            table: cursor.clone(),
            v_table: if addr_value == 0 {
                None
            } else {
                Some(Self::get_vtable(addr_value, memory))
            },
            v_table_addr: addr_value,
            v_table_index: 0,
            index: 0,
        }
    }

    #[inline(always)]
    pub fn get_vtable<'vtable, M: NP_Memory>(v_table_addr: usize, memory: &'vtable M) -> &'vtable mut NP_Vtable {
        if v_table_addr > memory.read_bytes().len() { // attack
            unsafe { &mut *(memory.write_bytes().as_ptr() as *mut NP_Vtable) }
        } else { // normal operation
            unsafe { &mut *(memory.write_bytes().as_ptr().add(v_table_addr) as *mut NP_Vtable) }
        }
    }

    #[inline(always)]
    pub fn step_iter<M: NP_Memory>(&mut self, memory: &'table M) -> Option<(usize, &'table str, Option<NP_Cursor>)> {

        match &memory.get_schema(self.table.schema_addr) {
            NP_Parsed_Schema::Table { columns, .. } => {

                if columns.len() <= self.index {
                    return None;
                }

                let v_table =  self.index / VTABLE_SIZE; // which vtable
                let v_table_idx = self.index % VTABLE_SIZE; // which index on the selected vtable

                if self.v_table_index > v_table {
                    self.v_table_index = v_table;
                    match &self.v_table {
                        Some(vtable) => {
                            let next_vtable = vtable.get_next() as usize;
                            if next_vtable > 0 {
                                self.v_table = Some(Self::get_vtable(next_vtable, memory));
                                self.v_table_addr = next_vtable;
                            } else {
                                self.v_table = None;
                                self.v_table_addr = 0;
                            }
                        },
                        _ => {}
                    }
                }

                let this_index = self.index;
                self.index += 1;

                if self.v_table_addr != 0 {
                    let item_address = self.v_table_addr + (v_table_idx * 2);
                    Some((this_index, columns[this_index].1.as_str(), Some(NP_Cursor::new(item_address, columns[this_index].2, self.table.schema_addr))))
                } else {
                    Some((this_index, columns[this_index].1.as_str(), None))
                }
            },
            _ => None
        }
    }
}

impl<'value> NP_Value<'value> for NP_Table<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("table", NP_TypeKeys::Table) }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let column_len = bytes[address + 1];

        let mut parsed_columns: Vec<(u8, String,  NP_Schema_Addr)> = Vec::new();

        let table_schema_addr = schema.len();

        schema.push(NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            // columns_mapped: Vec::new(),
            columns: Vec::new()
        });

        let mut schema_parsed = schema;

        let mut offset = address + 2;

        let mut hash_map = Vec::new();

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
            // hash_map.insert(col_name, x).unwrap_or_default();
            hash_map.push(col_name.to_string());
            offset += schema_size + 2;
        }

        // hash_map.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        schema_parsed[table_schema_addr] = NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            // columns_mapped: hash_map,
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
                    cols.push(NP_Schema::_type_to_json(&schema, column.2).unwrap_or(NP_JSON::Null));
                    NP_JSON::Array(cols)
                }).collect()
            },
            _ => Vec::new()
        };

        schema_json.insert("columns".to_owned(), NP_JSON::Array(columns));

        Ok(NP_JSON::Dictionary(schema_json))
    }
 
    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        let mut nex_vtable = c_value.get_addr_value() as usize;
        let mut loop_max = 65usize;
        while nex_vtable > 0 && loop_max > 0 {
            acc_size += 10;
            let vtable = Self::get_vtable(nex_vtable, memory);
            nex_vtable = vtable.get_next() as usize;
            loop_max -= 1;
        }

        let mut table = Self::new_iter(&cursor, memory);

        while let Some((_index, _key, item)) = table.step_iter(memory) {
            if let Some(real) = item {
                let add_size = NP_Cursor::calc_size(&real, memory)?;
                if add_size > 2 {
                    // scalar cursor is part of vtable
                    acc_size += add_size - 2;             
                }
            }         
        }
   
        Ok(acc_size)
    }

    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        let mut table = Self::new_iter(&cursor, memory);

        while let Some((_index, key, item)) = table.step_iter(memory) {
            if let Some(real) = item {
                json_map.insert(String::from(key), NP_Cursor::json_encode(&real, memory));  
            } else {
                json_map.insert(String::from(key), NP_JSON::Null);  
            }            
        }

        NP_JSON::Dictionary(json_map)
    }

    fn do_compact<M: NP_Memory, M2: NP_Memory>(from_cursor: NP_Cursor, from_memory: &'value M, mut to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        to_cursor = Self::make_first_vtable(to_cursor, to_memory)?;
        let to_cursor_value = to_cursor.get_value(to_memory);
        let mut last_real_vtable = to_cursor_value.get_addr_value() as usize;
        let mut last_vtable_idx = 0usize;

        let c: Vec<(u8, String, usize)>;
        let col_schemas = match &from_memory.get_schema(from_cursor.schema_addr) {
            NP_Parsed_Schema::Table { columns, .. } => {
                columns
            },
            _ => { c = Vec::new(); &c }
        };

        let mut table = Self::new_iter(&from_cursor, from_memory);

        while let Some((idx, _key, item)) = table.step_iter(from_memory) {
           if let Some(real) = item {

                let v_table =  idx / VTABLE_SIZE; // which vtable
                let v_table_idx = idx % VTABLE_SIZE; // which index on the selected vtable
                
                if last_vtable_idx < v_table {
                    let vtable_data = Self::get_vtable(last_real_vtable, to_memory);
                    last_real_vtable = Self::make_next_vtable(vtable_data, to_memory)?;
                    last_vtable_idx += 1;
                }

                let item_addr = last_real_vtable + (v_table_idx * 2);
                NP_Cursor::compact(real.clone(), from_memory, NP_Cursor::new(item_addr, col_schemas[idx].2, to_cursor.schema_addr), to_memory)?;
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
            columns: Vec::new(),
           //  columns_mapped: Vec::new()
        });

        let mut columns_mapped = Vec::new();

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
                    // columns_mapped.insert(column_name.as_str(), x as usize)?;
                    columns_mapped.push(column_name.to_string());
                    column_data.push((column_name, column_type));
                    x += 1;
                }
            },
            _ => { 
                return Err(NP_Error::new("Tables require a 'columns' property that is an array of schemas!"))
            }
        }

        // columns_mapped.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        schema_parsed[schema_table_addr] = NP_Parsed_Schema::Table {
            i: NP_TypeKeys::Table,
            sortable: false,
            columns: columns,
            // columns_mapped
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

            if col.1.len() > u16::MAX as usize {
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
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.after_compaction, 20usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    // good values are preserved through compaction
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 20usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 20usize);

    Ok(())
}


#[test]
fn test_vtables() -> Result<(), NP_Error> {
    let factory = crate::NP_Factory::new(r#"{
        "type": "table",
        "columns": [
            ["age",    {"type": "u8"}],
            ["name",   {"type": "string"}],
            ["color",  {"type": "string"}],
            ["car",    {"type": "string"}],
            ["rating", {"type": "u8"}]
        ]
    }"#)?;

    // compaction removes cleared values
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["age"], 20u8)?;
    buffer.set(&["name"], "hello")?;
    buffer.set(&["color"], "blue")?;
    buffer.set(&["car"], "Chevy")?;
    buffer.set(&["rating"], 98u8)?;

    let new_buffer = factory.open_buffer(buffer.close());
    assert_eq!(new_buffer.get::<u8>(&["age"])?.unwrap(), 20u8);
    assert_eq!(new_buffer.get::<&str>(&["name"])?.unwrap(), "hello");
    assert_eq!(new_buffer.get::<&str>(&["color"])?.unwrap(), "blue");
    assert_eq!(new_buffer.get::<&str>(&["car"])?.unwrap(), "Chevy");
    assert_eq!(new_buffer.get::<u8>(&["rating"])?.unwrap(), 98u8);

    Ok(())
}