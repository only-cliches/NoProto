use crate::pointer::DEF_TABLE;
use crate::pointer::NP_Cursor_Addr;
use crate::{pointer::NP_Cursor_Data, schema::NP_Schema_Addr, pointer::NP_Vtable};
use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP, pointer::{NP_Cursor}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};
use alloc::string::ToString;

/// Tuple data type.
/// 
#[doc(hidden)]
pub struct NP_Tuple {}

impl NP_Tuple {

    pub fn extend_vtables<'extend>(cursor: &NP_Cursor_Addr, memory: &'extend NP_Memory, col_index: usize) -> Result<&'extend [(usize, &'extend mut NP_Vtable); 64], NP_Error> {
        let cursor = memory.get_parsed(cursor);

        let desired_v_table =  col_index / 4;

        match &mut cursor.data {
            NP_Cursor_Data::Tuple { bytes } => {

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

    pub fn parse<'parse>(buff_addr: usize, schema_addr: NP_Schema_Addr, parent_addr: usize, parent_schema_addr: usize, memory: &NP_Memory<'parse>, values: &Vec<usize>) {

        let tuple_value = NP_Cursor::parse_cursor_value(buff_addr, parent_addr, parent_schema_addr, &memory);

        let mut new_cursor = NP_Cursor { 
            buff_addr: buff_addr, 
            schema_addr: schema_addr, 
            data: NP_Cursor_Data::Empty,
            temp_bytes: None,
            value: tuple_value, 
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
                NP_Cursor_Data::Tuple { bytes} => {
                    let mut column_index = 0usize;
                    for vtable in &bytes { // each vtable holds 4 columns
                        if vtable.0 != 0 {
                            for (i, pointer) in vtable.1.values.iter().enumerate() {
                                let item_buff_addr = vtable.0 + (i * 2);
                                let schema_addr = values[column_index];
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
            new_cursor.data = NP_Cursor_Data::Tuple { bytes: vtables };
            memory.insert_parsed(buff_addr, new_cursor);

        }
    }

}

impl<'value> NP_Value<'value> for NP_Tuple {

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

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

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
    let mut buffer = factory.empty_buffer(None)?;
    buffer.set(&["0"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 17usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}