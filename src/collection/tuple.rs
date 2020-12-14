use crate::{ pointer::NP_Vtable};
use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP, pointer::{NP_Cursor}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};
use alloc::string::ToString;

use super::table::NP_Table;

/// Tuple data type.
/// 
#[doc(hidden)]
pub struct NP_Tuple<'tuple> {
    index: usize,
    v_table: Option<&'tuple mut NP_Vtable>,
    v_table_addr: usize,
    v_table_index: usize,
    table: NP_Cursor
}

impl<'tuple> NP_Tuple<'tuple> {


    #[inline(always)]
    pub fn select(mut tuple_cursor: NP_Cursor, index: usize, schema_only: bool, memory: &NP_Memory) -> Result<Option<NP_Cursor>, NP_Error> {
        match &memory.schema[tuple_cursor.schema_addr] {
            NP_Parsed_Schema::Tuple { values, .. } => {

                if index >= values.len() {
                    return Ok(None)
                }
        
                let column_schema_data = values[index];

                if schema_only {
                    return Ok(Some(NP_Cursor::new(0, column_schema_data, tuple_cursor.schema_addr)));
                }

                let v_table =  index / 4; // which vtable
                let v_table_idx = index % 4; // which index on the selected vtable

                let mut table_value = tuple_cursor.get_value(memory);
                if table_value.get_addr_value() == 0 {
                    tuple_cursor = Self::make_first_vtable(tuple_cursor, memory)?;
                }
                table_value = tuple_cursor.get_value(memory);

                let mut seek_vtable = 0usize;
                let mut vtable_address = table_value.get_addr_value() as usize;

                while seek_vtable < v_table {
                    let this_vtable = Self::get_vtable(vtable_address, memory);
                    let next_vtable = this_vtable.get_next();

                    if next_vtable == 0 {
                        vtable_address = Self::make_next_vtable(this_vtable, memory)?;
                    } else {
                        vtable_address = next_vtable as usize;
                    }

                    seek_vtable += 1;
                }

                let item_address = vtable_address + (v_table_idx * 2);

                Ok(Some(NP_Cursor::new(item_address, column_schema_data, tuple_cursor.schema_addr)))
             
            },
            _ => unsafe { panic!() }
        }
    }

    pub fn make_first_vtable<'make>(table_cursor: NP_Cursor, memory: &'make NP_Memory) -> Result<NP_Cursor, NP_Error> {

        let first_vtable_addr = memory.malloc_borrow(&[0u8; 10])?;
        
        let table_value = table_cursor.get_value(memory);
        table_value.set_addr_value(first_vtable_addr as u16);

        Ok(table_cursor)
    }

    pub fn make_next_vtable<'make>(prev_vtable: &'make mut NP_Vtable, memory: &'make NP_Memory) -> Result<usize, NP_Error> {

        let vtable_addr = memory.malloc_borrow(&[0u8; 10])?;
        
        prev_vtable.set_next(vtable_addr as u16);

        Ok(vtable_addr)
    }

    pub fn new_iter(cursor: &NP_Cursor, memory: &'tuple NP_Memory) -> Self {

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

    pub fn get_vtable<'vtable>(v_table_addr: usize, memory: &'vtable NP_Memory) -> &'vtable mut NP_Vtable {
        unsafe { &mut *(memory.write_bytes().as_ptr().add(v_table_addr) as *mut NP_Vtable) }
    }

    pub fn step_iter(&mut self, memory: &'tuple NP_Memory) -> Option<(usize, Option<NP_Cursor>)> {

        match &memory.schema[self.table.schema_addr] {
            NP_Parsed_Schema::Tuple { values, .. } => {

                if values.len() <= self.index {
                    return None;
                }

                let v_table =  self.index / 4; // which vtable
                let v_table_idx = self.index % 4; // which index on the selected vtable

                if self.v_table_index > v_table_idx {
                    self.v_table_index = v_table_idx;
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
                    Some((this_index, Some(NP_Cursor::new(item_address, values[this_index], self.table.schema_addr))))
                } else {
                    Some((this_index, None))
                }
            },
            _ => unsafe { panic!() }
        }

        
    }

    pub fn for_each<F>(cursor_addr: &NP_Cursor, memory: &'tuple NP_Memory, callback: &mut F) where F: FnMut((usize, Option<NP_Cursor>)) {

        let mut table = Self::new_iter(cursor_addr, memory);

        while let Some((index, item)) = table.step_iter(memory) {
            callback((index, item))
        }

    }

}

impl<'value> NP_Value<'value> for NP_Tuple<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let schema_state: (bool, Vec<NP_JSON>) = match &schema[address] {
            NP_Parsed_Schema::Tuple { i: _, sortable, values } => {
                (*sortable, values.into_iter().map(|column| {
                    NP_Schema::_type_to_json(schema, *column).unwrap()
                }).collect())
            },
            _ => { unsafe { panic!() } }
        };

        schema_json.insert("values".to_owned(), NP_JSON::Array(schema_state.1));

        if schema_state.0 {
            schema_json.insert("sorted".to_owned(), NP_JSON::True);
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn get_size(cursor: &NP_Cursor, memory: &'value NP_Memory<'value>) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        let mut nex_vtable = c_value.get_addr_value() as usize;

        while nex_vtable > 0 {
            acc_size += 10;
            let vtable = Self::get_vtable(nex_vtable, memory);
            nex_vtable = vtable.get_next() as usize;
        }

        // loop through values in table
        Self::for_each(&cursor, memory, &mut |(_i, item)| {
            if let Some(real) = item {
                let add_size = NP_Cursor::calc_size(&real, memory).unwrap();
                if add_size > 2 {
                    // scalar cursor is part of vtable
                    acc_size += add_size - 2;             
                }
            }
        });
   
        Ok(acc_size)
    }

    fn to_json(cursor: &NP_Cursor, memory: &'value NP_Memory) -> NP_JSON {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_list = Vec::new();

        Self::for_each(&cursor, memory, &mut |(idx, item)| {
            if let Some(real) = item {
                json_list.push(NP_Cursor::json_encode(&real, memory));  
            } else {
                json_list.push(NP_JSON::Null);  
            }
        });

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_cursor: NP_Cursor, from_memory: &'value NP_Memory, mut to_cursor: NP_Cursor, to_memory: &'value NP_Memory) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        to_cursor = Self::make_first_vtable(to_cursor, to_memory)?;
        let to_cursor_value = to_cursor.get_value(to_memory);
        let mut last_real_vtable = to_cursor_value.get_addr_value() as usize;
        let mut last_vtable_idx = 0usize;

        let col_schemas = match &from_memory.schema[from_cursor.schema_addr] {
            NP_Parsed_Schema::Table { columns, .. } => {
                columns
            },
            _ => unsafe { panic!() }
        };

        Self::for_each(&from_cursor, from_memory, &mut |(idx, item)| {

            if let Some(real) = item {

                let v_table =  idx / 4; // which vtable
                let v_table_idx = idx % 4; // which index on the selected vtable
                
                if last_vtable_idx < v_table {
                    let vtable_data = Self::get_vtable(last_real_vtable, to_memory);
                    last_real_vtable = Self::make_next_vtable(vtable_data, to_memory).unwrap();
                    last_vtable_idx += 1;
                }

                let item_addr = last_real_vtable + (v_table_idx * 2);
                NP_Cursor::compact(real.clone(), from_memory, NP_Cursor::new(item_addr, col_schemas[idx].2, to_cursor.schema_addr), to_memory).unwrap();
            }
        });

        Ok(to_cursor)
    }


    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

    
        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Tuple as u8);

        let mut sorted = false;

        match json_schema["sorted"] {
            NP_JSON::True => {
                sorted = true;
                schema_data.push(1);
            },
            _ => {
                schema_data.push(0);
            }
        }

        let mut column_schemas: Vec<Vec<u8>> = Vec::new();
        let tuple_addr = schema.len();
        schema.push(NP_Parsed_Schema::Tuple {
            i: NP_TypeKeys::Tuple,
            sortable: sorted,
            values: Vec::new()
        });

        let mut tuple_values = Vec::new();

        let mut working_schema = schema;

        match &json_schema["values"] {
            NP_JSON::Array(cols) => {
                for col in cols {
                    tuple_values.push(working_schema.len());
                    let (is_sortable, schema_bytes, _schema ) = NP_Schema::from_json(working_schema, &Box::new(col.clone()))?;
                    working_schema = _schema;
                    if sorted && is_sortable == false {
                        return Err(NP_Error::new("All children of a sorted tuple must be sortable items!"))
                    }
                    column_schemas.push(schema_bytes);
                }
            },
            _ => { 
                return Err(NP_Error::new("Tuples require a 'values' property that is an array of schemas!"))
            }
        }
        
        working_schema[tuple_addr] = NP_Parsed_Schema::Tuple {
            i: NP_TypeKeys::Tuple,
            sortable: sorted,
            values: tuple_values
        };

        if column_schemas.len() > 255 {
            return Err(NP_Error::new("Tuples cannot have more than 255 values!"))
        }

        // number of schema values
        schema_data.push(column_schemas.len() as u8);

        for col in column_schemas {

            if col.len() > u16::max as usize {
                return Err(NP_Error::new("Schema overflow error!"))
            }
            
            // column type
            schema_data.extend((col.len() as u16).to_be_bytes().to_vec());
            schema_data.extend(col);
        }

        return Ok((sorted, schema_data, working_schema))
     
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let is_sorted = bytes[address + 1];

        let column_len = bytes[address + 2];

        let mut working_schema = schema;

        let tuple_schema_addr = working_schema.len();
        working_schema.push(NP_Parsed_Schema::Tuple {
            i: NP_TypeKeys::Tuple,
            values: Vec::new(), 
            sortable: is_sorted != 0 
        });

        let mut tuple_values: Vec<usize> = Vec::new();

        let mut offset = address + 3;

        for _x in 0..column_len as usize {

            let schema_size = u16::from_be_bytes([
                bytes[offset],
                bytes[offset + 1]
            ]) as usize;

            tuple_values.push(working_schema.len());
            let (_sortable, schema_) = NP_Schema::from_bytes(working_schema, offset + 2, bytes);
            working_schema = schema_;

            offset += schema_size + 2;
        }

        working_schema[tuple_schema_addr] = NP_Parsed_Schema::Tuple {
            i: NP_TypeKeys::Tuple,
            values: tuple_values, 
            sortable: is_sorted != 0 
        };

        (is_sorted != 0, working_schema)
    }
}




#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"tuple\",\"values\":[{\"type\":\"string\"},{\"type\":\"uuid\"},{\"type\":\"uint8\"}]}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"tuple\",\"values\":[{\"type\":\"string\",\"size\":10},{\"type\":\"uuid\"},{\"type\":\"uint8\"}],\"sorted\":true}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"tuple\",\"values\":[{\"type\":\"string\"},{\"type\":\"uuid\"},{\"type\":\"uint8\"}]}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&["0"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 19usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 2usize);

    Ok(())
}