use crate::pointer::{NP_Cursor_Parent};
use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP, pointer::{NP_Cursor}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};
use alloc::string::ToString;

/// Tuple data type.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Tuple<'tuple> {
    cursor: NP_Cursor,
    tuple: NP_Cursor_Parent,
    current: Option<(usize, NP_Cursor)>,
    pub memory: &'tuple NP_Memory<'tuple>
}

impl<'tuple> NP_Tuple<'tuple> {

    /// Generate a new tuple iterator
    #[inline(always)]
    pub fn new(mut cursor: NP_Cursor, memory: &'tuple NP_Memory<'tuple>) -> Self {
        let value_addr = if cursor.buff_addr != 0 { memory.read_address(cursor.buff_addr) } else { 0 };
        cursor.value = cursor.value.update_value_address(value_addr);
        Self {
            cursor: cursor,
            tuple: NP_Cursor_Parent::Tuple {
                addr: value_addr,
                schema_addr: cursor.schema_addr
            },
            current: None,
            memory: memory
        }
    }

    /// Read or save a list into the buffer
    /// 
    #[inline(always)]
    pub fn read_tuple(buff_addr: usize, schema_addr: usize, memory: &NP_Memory<'tuple>, create: bool) -> Result<NP_Cursor, NP_Error> {

        let mut cursor = NP_Cursor::new(buff_addr, schema_addr, &memory, NP_Cursor_Parent::None);
        let mut value_addr = cursor.value.get_value_address();
        let addr_size = memory.addr_size_bytes();

        let tuple_size = match &memory.schema[schema_addr] {
            NP_Parsed_Schema::Tuple { values, .. } => values.len(),
            _ => unsafe { unreachable_unchecked() }
        };
        
        if value_addr == 0 { // no tuple here
            if create { // please make one
                assert_ne!(cursor.buff_addr, 0); 

                let tuple_size = addr_size * tuple_size;

                let mut empty_bytes = Vec::with_capacity(tuple_size);
                for _x in 0..tuple_size {
                    empty_bytes.push(0);
                }

                value_addr = memory.malloc_borrow(&empty_bytes)?;
                // update buffer
                memory.write_address(cursor.buff_addr, value_addr);
                // update cursor
                cursor.value = cursor.value.update_value_address(value_addr);
                Ok(cursor)
            } else { // no tuple and no need to make one, just pass empty data
                Ok(cursor)       
            }
        } else { // tuple found, read info from buffer
            Ok(cursor)
        }
    }

    /// Select into pointer
    #[inline(always)]
    pub fn select_into(cursor: NP_Cursor, memory: &'tuple NP_Memory<'tuple>, col: usize, create_path: bool) -> Result<Option<NP_Cursor>, NP_Error> {

        let addr_size = memory.addr_size_bytes();

        let tuple_cursor = Self::read_tuple(cursor.buff_addr, cursor.schema_addr, memory, create_path)?;

        let value_schema_addr = match &memory.schema[cursor.schema_addr] {
            NP_Parsed_Schema::Tuple { values, .. } => {
                if values.len() <= col {
                    return Ok(None)
                }
                values[col]
            },
            _ => unsafe { unreachable_unchecked() }
        };

        let value_buff_addr = if tuple_cursor.value.get_value_address() > 0 {
            tuple_cursor.value.get_value_address() + (addr_size * col)
        } else {
            0
        };

        let virtual_cursor = NP_Cursor::new(value_buff_addr, value_schema_addr, memory, NP_Cursor_Parent::Tuple { addr: tuple_cursor.value.get_value_address(), schema_addr: cursor.schema_addr });

        Ok(Some(virtual_cursor))
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
            _ => { unsafe { unreachable_unchecked() } }
        };

        schema_json.insert("values".to_owned(), NP_JSON::Array(schema_state.1));

        if schema_state.0 {
            schema_json.insert("sorted".to_owned(), NP_JSON::True);
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_cursor_addr: NP_Cursor, _memory: &NP_Memory, _value: Self) -> Result<NP_Cursor, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()!"))
    }

    fn get_size(cursor: NP_Cursor, memory: &NP_Memory) -> Result<usize, NP_Error> {
        
        let base_size = 0usize;

        if cursor.value.get_value_address() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        for (_index, item ) in NP_Tuple::new(cursor.clone(), memory) {
            if item.buff_addr != 0 && item.value.get_value_address() != 0 {
                acc_size += NP_Cursor::calc_size(item.clone(), memory)?; // item
            } else {
                // empty pointer
                acc_size += match memory.size {
                    NP_Size::U8  => 1,
                    NP_Size::U16 => 2,
                    NP_Size::U32 => 4
                };
            }
        }

        return Ok(base_size + acc_size);
    }

    fn to_json(cursor: &NP_Cursor, memory: &NP_Memory) -> NP_JSON {

        if cursor.buff_addr == 0 { return NP_JSON::Null };

        let mut json_list = Vec::new();

        for (_index, item ) in NP_Tuple::new(cursor.clone(), memory) {
            json_list.push(NP_Cursor::json_encode(&item, memory));
        }

        NP_JSON::Array(json_list)
    }

    fn do_compact(from_cursor: &NP_Cursor, from_memory: &NP_Memory<'value>, to_cursor: NP_Cursor, to_memory: &NP_Memory<'value>) -> Result<NP_Cursor, NP_Error> where Self: 'value {

        if from_cursor.buff_addr == 0 || from_cursor.value.get_value_address() == 0 {
            return Ok(to_cursor);
        }

        for (index, old_item) in NP_Tuple::new(from_cursor.clone(), from_memory) {
            if old_item.buff_addr != 0 && old_item.value.get_value_address() != 0 { // pointer has value
                let new_item = NP_Tuple::select_into(to_cursor.clone(), &to_memory, index, true)?.unwrap();
                NP_Cursor::compact(&old_item, from_memory, new_item, to_memory)?;
            } 
        }

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


impl<'it> Iterator for NP_Tuple<'it> {
    type Item = (usize, NP_Cursor);

    fn next(&mut self) -> Option<Self::Item> {

        if let Some((index, _current)) = self.current { // go to next one
            let values_len = match &self.memory.schema[self.cursor.schema_addr] {
                NP_Parsed_Schema::Tuple { values, ..} => {
                    values.len()
                },
                _ => { unsafe { unreachable_unchecked() }}
            };

            if values_len >= index + 1 {
                self.current = None;
            } else {
                self.current = Some((index + 1, NP_Tuple::select_into(self.cursor.clone(), self.memory, index + 1, true).unwrap().unwrap()))
            }
        } else { // nothing in loop yet, make first loop item
            self.current = Some((0, NP_Tuple::select_into(self.cursor.clone(), self.memory, 0, true).unwrap().unwrap()))
        }

        self.current
    }

    fn count(self) -> usize where Self: Sized {
        match &self.memory.schema[self.cursor.schema_addr] {
            NP_Parsed_Schema::Tuple { values, ..} => {
                values.len()
            },
            _ => { unsafe { unreachable_unchecked() }}
        }
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
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(&["0"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 17usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}