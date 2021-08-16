use crate::{idl::JS_AST, pointer::NP_Cursor_Parent, schema::{NP_Tuple_Data, NP_Tuple_Field, NP_Value_Kind}};
use alloc::{string::String, sync::Arc};
use crate::{idl::JS_Schema};

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
#[derive(Debug)]
pub struct NP_Tuple {
    index: usize,
    table: NP_Cursor
}

#[allow(missing_docs)]
impl NP_Tuple {

    #[inline(always)]
    pub fn select(mut tuple_cursor: NP_Cursor, schema: &NP_Parsed_Schema, index: usize, make_path: bool, schema_query: bool, memory: &NP_Memory) -> Result<Option<NP_Cursor>, NP_Error> {
    // pub fn select(mut tuple_cursor: NP_Cursor, empty: &Vec<u8>, values: &Vec<NP_Tuple_Field>, index: usize, make_path: bool, schema_query: bool, memory: &NP_Memory) -> Result<Option<NP_Cursor>, NP_Error> {

        let data = unsafe { &*(*schema.data as *const NP_Tuple_Data) };

        if index >= data.values.len() {
            return Ok(None)
        }

        if schema_query {
            return Ok(Some(NP_Cursor::new(0, data.values[index].schema, tuple_cursor.schema_addr)));
        }

        let value_schema_data = data.values[index].schema;

        let mut tuple = tuple_cursor.get_value(memory);
        if tuple.get_addr_value() == 0 {
            if make_path {
                tuple_cursor = Self::alloc_tuple(tuple_cursor, &data.empty, memory)?;

                tuple = tuple_cursor.get_value(memory);
            } else {
                return Ok(None);
            }
        }
        
        let item_address = tuple.get_addr_value() as usize + data.values[index].offset;

        let mut cursor = NP_Cursor::new(item_address, value_schema_data, tuple_cursor.schema_addr);

        cursor.parent_type = NP_Cursor_Parent::Tuple;

        if data.values[index].fixed {
            cursor.value_bytes = Some((item_address as u32).to_be_bytes()); 
        }

        if memory.read_bytes()[item_address - 1] == 0 && make_path == false {
            Ok(None)
        } else {
            Ok(Some(cursor))
        }
    

    }

    #[inline(always)]
    pub fn alloc_tuple<'make>(tuple_cursor: NP_Cursor, empty: &Vec<u8>, memory: &'make NP_Memory) -> Result<NP_Cursor, NP_Error> {

        let new_addr = memory.malloc_borrow(empty)?;
        
        tuple_cursor.get_value_mut(memory).set_addr_value(new_addr as u32);

        Ok(tuple_cursor)
    }

    pub fn new_iter(cursor: &NP_Cursor, _memory: &NP_Memory) -> Self {

        Self {
            table: cursor.clone(),
            index: 0,
        }
    }

    pub fn step_iter(&mut self, memory: &NP_Memory, show_empty: bool) -> Option<(usize, Option<NP_Cursor>)> {

        let data = unsafe { &*(*memory.get_schema(self.table.schema_addr).data as *const NP_Tuple_Data) };

        if data.values.len() <= self.index {
            return None;
        }

        let this_index = self.index;
        self.index += 1;

        let next_cursor = Self::select(self.table, memory.get_schema(self.table.schema_addr), this_index, true, false, memory);

        match next_cursor {
            Ok(next) => {
                match next {
                    Some(cursor) => {
                        if memory.read_bytes()[cursor.buff_addr - 1] == 0 && show_empty {
                            Some((this_index, None))
                        } else {
                            Some((this_index, Some(cursor)))
                        }
                    },
                    None => None
                }
            },
            Err(_e) => { None }
        }
   
    }

}

impl<'value> NP_Value<'value> for NP_Tuple {

    fn to_json(depth:usize, cursor: &NP_Cursor, memory: &'value NP_Memory) -> NP_JSON {
        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_list = Vec::new();

        let mut tuple = NP_Tuple::new_iter(&cursor, memory);

        while let Some((_idx, item)) = tuple.step_iter(memory, false) {
            if let Some(x) = item {
                json_list.push(NP_Cursor::json_encode(depth + 1, &x, memory)); 
            } else {
                json_list.push(NP_JSON::Null);
            }
        }


        NP_JSON::Array(json_list)
    }

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let data = unsafe { &*(*schema[address].data as *const NP_Tuple_Data) };

        let schema_state: (bool, Vec<NP_JSON>) = (schema[address].sortable, data.values.iter().map(|column| {
            NP_Schema::_type_to_json(schema, column.schema).unwrap_or(NP_JSON::Null)
        }).collect());

        schema_json.insert("values".to_owned(), NP_JSON::Array(schema_state.1));

        if schema_state.0 {
            schema_json.insert("sorted".to_owned(), NP_JSON::True);
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_from_json<'set>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &'set NP_Memory, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {
        
        match &**value {
            NP_JSON::Array(list) => {
                for (idx, tuple_item) in list.iter().enumerate() {
                    match NP_Tuple::select(cursor, memory.get_schema(cursor.schema_addr), idx, true, false, memory)? {
                        Some(x) => {
                            NP_Cursor::set_from_json(depth + 1, apply_null, x, memory, &Box::new(tuple_item.clone()))?;
                        },
                        None => { 
                            return Err(NP_Error::new("Failed to find column value!"))
                        }
                    }
                }
            },
            _ => { }
        }
        
        Ok(())
    }

    fn get_size(depth:usize, cursor: &NP_Cursor, memory: &'value NP_Memory) -> Result<usize, NP_Error> {

        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        let mut tuple = Self::new_iter(&cursor, memory);

        let data = unsafe { &*(*memory.get_schema(cursor.schema_addr).data as *const NP_Tuple_Data) };

        while let Some((index, item)) = tuple.step_iter(memory, false) {
            if let Some(cursor) = item {
                acc_size += 1;
                let schema_value = &data.values[index];
                if schema_value.fixed {
                    acc_size += schema_value.size;
                } else {
                    acc_size += NP_Cursor::calc_size(depth + 1, &cursor, memory)?;
                }   
            }   
        }
    
        Ok(acc_size)
       
    }

    fn do_compact(depth:usize, from_cursor: NP_Cursor, from_memory: &'value NP_Memory, mut to_cursor: NP_Cursor, to_memory: &'value NP_Memory) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        let data = unsafe { &*(*from_memory.get_schema(from_cursor.schema_addr).data as *const NP_Tuple_Data) };

        let (col_schemas, _empty) = (&data.values, &data.empty);

        to_cursor = Self::alloc_tuple(to_cursor, &data.empty, to_memory)?;

        let mut tuple = Self::new_iter(&from_cursor, from_memory);

        while let Some((idx, item)) = tuple.step_iter(from_memory, false) {
            if let Some(old_cursor) = item {
                to_memory.write_bytes()[old_cursor.buff_addr - 1] = 1;
                NP_Cursor::compact(depth + 1, old_cursor.clone(), from_memory, NP_Cursor::new(old_cursor.buff_addr, col_schemas[idx].schema, to_cursor.schema_addr), to_memory)?;
            }
        }

        Ok(to_cursor)
    }

    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error> {
        let data = unsafe { &*(*schema[address].data as *const NP_Tuple_Data) };

        let mut result = String::from("tuple({values: [");

        let last_index = data.values.len() - 1;
        for (idx, field) in data.values.iter().enumerate() {
            result.push_str(NP_Schema::_type_to_idl(schema, field.schema)?.as_str());
            if idx < last_index {
                result.push_str(", ");
            }
        }

        result.push_str("]");
        if schema[address].sortable == true {
            result.push_str(", sorted: true");
        }
        result.push_str("})");
        Ok(result)
         
    }

    fn from_idl_to_schema(mut schema: Vec<NP_Parsed_Schema>, _name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Tuple as u8);

        let mut sorted = false;
        let mut tuple_values: Option<&Vec<JS_AST>> = None;

        if args.len() > 0 {
            match &args[0] {
                JS_AST::object { properties } => {
                    for (key, value) in properties {
                        match idl.get_str(key).trim() {
                            "sorted" => {
                                sorted = true;
                            },
                            "values" => {
                                match value {
                                    JS_AST::array { values } => {
                                        tuple_values = Some(values);
                                    },
                                    _ => { }
                                }
                            },
                            _ => { }
                        }
                    }
                },
                _ => { }
            }
        }

        if sorted {
            schema_data.push(1);
        } else {
            schema_data.push(0);
        }

        if let Some(tuple_vals) = tuple_values {

            let mut column_schemas: Vec<Vec<u8>> = Vec::new();
            let tuple_addr = schema.len();
            schema.push(NP_Parsed_Schema {
                val: NP_Value_Kind::Pointer,
                i: NP_TypeKeys::Tuple,
                sortable: sorted,
                data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: Vec::new(), empty: Vec::new() })) as *const u8)
            });
    
            let mut tuple_values: Vec<NP_Tuple_Field> = Vec::new();
    
            let mut working_schema = schema;

            let mut data_offset = 1usize;
    
            for col in tuple_vals {
                let schema_len = working_schema.len();
                let (is_sortable, schema_bytes, schema ) = NP_Schema::from_idl(working_schema, idl, &col)?;
                match schema[schema_len].val {
                    NP_Value_Kind::Pointer => {
                        tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: 0, fixed: false });
                        data_offset += 2;
                    },
                    NP_Value_Kind::Fixed(x) => {
                        tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: x as usize, fixed: true });
                        data_offset += x as usize;
                    }
                }
                data_offset += 1;
                working_schema = schema;
                if sorted && is_sortable == false {
                    return Err(NP_Error::new("All children of a sorted tuple must be sortable items!"))
                }
                column_schemas.push(schema_bytes);
            }
            
            working_schema[tuple_addr] = NP_Parsed_Schema {
                val: NP_Value_Kind::Pointer,
                i: NP_TypeKeys::Tuple,
                sortable: sorted,
                data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: tuple_values, empty: vec![0; data_offset - 1] })) as *const u8)
            };

            if column_schemas.len() > 255 {
                return Err(NP_Error::new("Tuples cannot have more than 255 values!"))
            }
    
            // number of schema values
            schema_data.push(column_schemas.len() as u8);
    
            for col in column_schemas {
    
                if col.len() > u16::MAX as usize {
                    return Err(NP_Error::new("Schema overflow error!"))
                }
                
                // column type
                schema_data.extend((col.len() as u16).to_be_bytes().to_vec());
                schema_data.extend(col);
            }
    
            Ok((sorted, schema_data, working_schema))
        } else {
            Err(NP_Error::new("Tuples require a 'values' property that is an array of schemas!"))
        }
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
        schema.push(NP_Parsed_Schema {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Tuple,
            sortable: sorted,
            data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: Vec::new(), empty: Vec::new() })) as *const u8)
        });

        let mut tuple_values: Vec<NP_Tuple_Field> = Vec::new();

        let mut working_schema = schema;

        let mut data_offset = 1usize;

        match &json_schema["values"] {
            NP_JSON::Array(cols) => {
                for col in cols {
                    let schema_len = working_schema.len();
                    let (is_sortable, schema_bytes, schema ) = NP_Schema::from_json(working_schema, &Box::new(col.clone()))?;
                    
                    match schema[schema_len].val {
                        NP_Value_Kind::Pointer => {
                            tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: 0, fixed: false });
                            data_offset += 4;
                        },
                        NP_Value_Kind::Fixed(x) => {
                            tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: x as usize, fixed: true });
                            data_offset += x as usize;
                        }
                    }
                    data_offset += 1;
                    working_schema = schema;
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
        
        working_schema[tuple_addr] = NP_Parsed_Schema {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Tuple,
            sortable: sorted,
            data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: tuple_values, empty: vec![0; data_offset - 1] })) as *const u8)
        };

        if column_schemas.len() > 255 {
            return Err(NP_Error::new("Tuples cannot have more than 255 values!"))
        }

        // number of schema values
        schema_data.push(column_schemas.len() as u8);

        for col in column_schemas {

            if col.len() > u16::MAX as usize {
                return Err(NP_Error::new("Schema overflow error!"))
            }
            
            // column type
            schema_data.extend((col.len() as u16).to_be_bytes().to_vec());
            schema_data.extend(col);
        }

        return Ok((sorted, schema_data, working_schema))
     
    }

    fn default_value(_depth: usize, _addr: usize, _schema: &Vec<NP_Parsed_Schema>) -> Option<Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let is_sorted = bytes[address + 1];

        let column_len = bytes[address + 2];

        let mut working_schema = schema;

        let tuple_schema_addr = working_schema.len();
        working_schema.push(NP_Parsed_Schema {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Tuple,
            sortable: is_sorted != 0,
            data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: Vec::new(), empty: Vec::new() })) as *const u8)
        });

        let mut tuple_values: Vec<NP_Tuple_Field> = Vec::new();

        let mut offset = address + 3;

        let mut data_offset = 1usize;

        for _x in 0..column_len as usize {

            let schema_size = u16::from_be_bytes([
                bytes[offset],
                bytes[offset + 1]
            ]) as usize;
            let schema_len = working_schema.len();
            let (_sortable, schema) = NP_Schema::from_bytes(working_schema, offset + 2, bytes);
            match schema[schema_len].val {
                NP_Value_Kind::Pointer => {
                    tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: 0, fixed: false });
                    data_offset += 2;
                },
                NP_Value_Kind::Fixed(x) => {
                    tuple_values.push(NP_Tuple_Field { schema: schema_len, offset: data_offset, size: x as usize, fixed: true });
                    data_offset += x as usize;
                }
            }
            data_offset += 1;
            working_schema = schema;

            offset += schema_size + 2;
        }

        working_schema[tuple_schema_addr] = NP_Parsed_Schema {
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Tuple,
            sortable: is_sorted != 0,
            data: Arc::new(Box::into_raw(Box::new(NP_Tuple_Data { values: tuple_values, empty: vec![0; data_offset - 1] })) as *const u8)
        };

        (is_sorted != 0, working_schema)
    }
}



#[test]
fn schema_parsing_works_idl() -> Result<(), NP_Error> {
    let schema = "tuple({values: [string(), uuid(), u8()]})";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl()?);
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl()?);

    let schema = "tuple({values: [string({size: 10}), uuid(), u8()], sorted: true})";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl()?);
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl()?);
    Ok(())
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"tuple\",\"values\":[{\"type\":\"string\"},{\"type\":\"uuid\"},{\"type\":\"uint8\"}]}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());

    let schema = "{\"type\":\"tuple\",\"values\":[{\"type\":\"string\",\"size\":10},{\"type\":\"uuid\"},{\"type\":\"uint8\"}],\"sorted\":true}";
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());
    Ok(())
}


#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"tuple","values":[{"type":"string"},{"type":"uuid"},{"type":"uint8"}]}"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
    buffer.set(&["0"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.current_buffer, 39usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 6usize);

    buffer.set_with_json(&[], r#"{"value": ["bar", "1ED3C129-2943-4CCE-8904-53C0487FF18E", 50]}"#)?;
    assert_eq!(buffer.get::<&str>(&["0"])?, Some("bar"));
    assert_eq!(buffer.get::<crate::pointer::uuid::NP_UUID>(&["1"])?, Some(crate::pointer::uuid::NP_UUID::from_string("1ED3C129-2943-4CCE-8904-53C0487FF18E")));
    assert_eq!(buffer.get::<u8>(&["2"])?, Some(50u8));

    Ok(())
}

#[test]
fn sorting_tuples_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"tuple","values":[{"type":"string","size":10},{"type":"uuid"},{"type":"uint8"}],"sorted":true}"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    let mut buffer = factory.new_buffer(None);
    buffer.set_min(&[])?;
    assert_eq!(buffer.read_bytes(), &[0, 0, 0, 0, 0, 6, 1, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
    buffer.set(&["0"], "hello")?;
    let uuid = crate::pointer::uuid::NP_UUID::generate(22);
    buffer.set(&["1"], &uuid)?;
    buffer.set(&["2"], 20u8)?;
    assert_eq!(buffer.read_bytes(), &[0, 0, 0, 0, 0, 6, 1, 104, 101, 108, 108, 111, 32, 32, 32, 32, 32, 1, 76, 230, 170, 176, 120, 208, 69, 186, 109, 122, 100, 179, 210, 224, 68, 195, 1, 20]);

    Ok(())
}