use crate::{idl::AST_STR, schema::{NP_Struct_Field, NP_Value_Kind}};
use crate::{buffer::{VTABLE_BYTES, VTABLE_SIZE}, idl::{JS_AST, JS_Schema}};
use alloc::string::String;
use crate::pointer::{NP_Vtable};
use crate::{pointer::{NP_Cursor}, schema::{NP_Parsed_Schema}};
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
pub struct NP_Struct<'table> {
    index: usize,
    v_table: Option<&'table mut NP_Vtable>,
    v_table_addr: usize,
    v_table_index: usize,
    table: NP_Cursor
}

#[allow(missing_docs)]
impl<'table> NP_Struct<'table> {

    #[inline(always)]
    pub fn select<M: NP_Memory>(mut table_cursor: NP_Cursor, _empty: &Vec<u8>, fields: &Vec<NP_Struct_Field>,  key: &str, make_path: bool, schema_query: bool, memory: &M) -> Result<Option<NP_Cursor>, NP_Error> {
       
        match fields.iter().position(|val| { val.col == key }) {
            Some(x) => {

                if schema_query {
                    return Ok(Some(NP_Cursor::new(0, fields[x].schema, table_cursor.schema_addr)));
                }

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

                Ok(Some(NP_Cursor::new(item_address, fields[x].schema, table_cursor.schema_addr)))
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
            NP_Parsed_Schema::Struct { fields, .. } => {

                if fields.len() <= self.index {
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
                    Some((this_index, fields[this_index].col.as_str(), Some(NP_Cursor::new(item_address, fields[this_index].schema, self.table.schema_addr))))
                } else {
                    Some((this_index, fields[this_index].col.as_str(), None))
                }
            },
            _ => None
        }
    }
}

impl<'value> NP_Value<'value> for NP_Struct<'value> {

    fn to_json<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {
        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 { return NP_JSON::Null };

        let mut json_map = JSMAP::new();

        let mut struc = NP_Struct::new_iter(&cursor, memory);

        while let Some((_index, key, item)) = struc.step_iter(memory) {
            if let Some(real) = item {
                json_map.insert(String::from(key), NP_Cursor::json_encode(depth + 1, &real, memory));  
            } else {
                json_map.insert(String::from(key), NP_JSON::Null);  
            }            
        }

        NP_JSON::Dictionary(json_map)
    }

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("struct", NP_TypeKeys::Struct) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("struct", NP_TypeKeys::Struct) }

    fn set_from_json<'set, M: NP_Memory>(depth: usize, apply_null: bool, cursor: NP_Cursor, memory: &'set M, value: &Box<NP_JSON>) -> Result<(), NP_Error> where Self: 'set + Sized {
        
        match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Struct { fields, empty, .. } => {
                for col in fields.iter() {
                    let json_col = &value[col.col.as_str()];
                    match json_col {
                        NP_JSON::Null => {
                            if apply_null {
                                match NP_Struct::select(cursor, empty, fields, &col.col, false, false, memory)? {
                                    Some(x) => {
                                        NP_Cursor::delete(x, memory)?;
                                    },
                                    None => { }
                                }
                            }
                        },
                        _ => {
                            match NP_Struct::select(cursor, empty, fields, &col.col, true, false, memory)? {
                                Some(x) => {
                                    NP_Cursor::set_from_json(depth + 1, apply_null, x, memory, &Box::new(json_col.clone()))?;
                                },
                                None => { 
                                    return Err(NP_Error::new("Failed to find field value!"))
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        }

        
        Ok(())
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let fields_count = bytes[address + 1];

        let mut parsed_fields: Vec<NP_Struct_Field> = Vec::new();

        let table_schema_addr = schema.len();

        schema.push(NP_Parsed_Schema::Struct {
            val: NP_Value_Kind::Pointer,
            empty: Vec::new(),
            i: NP_TypeKeys::Struct,
            sortable: false,
            // fields_mapped: Vec::new(),
            fields: Vec::new()
        });

        let mut schema_parsed = schema;

        let mut offset = address + 2;

        let mut hash_map = Vec::new();

        for x in 0..fields_count as usize {
            let col_name_len = bytes[offset] as usize;
            let col_name_bytes = &bytes[(offset + 1)..(offset + 1 + col_name_len)];
            let col_name = unsafe { core::str::from_utf8_unchecked(col_name_bytes) };

            offset += 1 + col_name_len;

            let schema_size = u16::from_be_bytes([
                bytes[offset],
                bytes[offset + 1]
            ]) as usize;

            let field_addr = schema_parsed.len();
            let (_, schema) = NP_Schema::from_bytes(schema_parsed, offset + 2, bytes);
            schema_parsed = schema;
            // parsed_fields.push((x as u8, col_name.to_string(), field_addr));
            parsed_fields.push(NP_Struct_Field { idx: x as u8, col: col_name.to_string(), schema: field_addr, offset: 0});
            // hash_map.insert(col_name, x).unwrap_or_default();
            hash_map.push(col_name.to_string());
            offset += schema_size + 2;
        }

        // hash_map.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        schema_parsed[table_schema_addr] = NP_Parsed_Schema::Struct {
            empty: Vec::new(),
            val: NP_Value_Kind::Pointer,
            i: NP_TypeKeys::Struct,
            // fields_mapped: hash_map,
            sortable: false,
            fields: parsed_fields
        };

        (false, schema_parsed)
    }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let fields: Vec<NP_JSON> = match &schema[address] {
            NP_Parsed_Schema::Struct { fields, .. } => {
                fields.into_iter().map(|field| {
                    let mut cols: Vec<NP_JSON> = Vec::new();
                    cols.push(NP_JSON::String(field.col.to_string()));
                    cols.push(NP_Schema::_type_to_json(&schema, field.schema).unwrap_or(NP_JSON::Null));
                    NP_JSON::Array(cols)
                }).collect()
            },
            _ => Vec::new()
        };

        schema_json.insert("fields".to_owned(), NP_JSON::Array(fields));

        Ok(NP_JSON::Dictionary(schema_json))
    }
 
    fn get_size<M: NP_Memory>(depth:usize, cursor: &NP_Cursor, memory: &'value M) -> Result<usize, NP_Error> {

        let c_value = || { cursor.get_value(memory) };

        if c_value().get_addr_value() == 0 {
            return Ok(0) 
        }

        let mut acc_size = 0usize;

        let mut nex_vtable = c_value().get_addr_value() as usize;
        let mut loop_max = 65usize;
        while nex_vtable > 0 && loop_max > 0 {
            acc_size += 10;
            let vtable = Self::get_vtable(nex_vtable, memory);
            nex_vtable = vtable.get_next() as usize;
            loop_max -= 1;
        }

        let mut struc = Self::new_iter(&cursor, memory);

        while let Some((_index, _key, item)) = struc.step_iter(memory) {
            if let Some(real) = item {
                let add_size = NP_Cursor::calc_size(depth + 1, &real, memory)?;
                if add_size > 2 {
                    // scalar cursor is part of vtable
                    acc_size += add_size - 2;             
                }
            }         
        }
   
        Ok(acc_size)
    }



    fn do_compact<M: NP_Memory, M2: NP_Memory>(depth:usize, from_cursor: NP_Cursor, from_memory: &'value M, mut to_cursor: NP_Cursor, to_memory: &'value M2) -> Result<NP_Cursor, NP_Error> where Self: 'value + Sized {

        let from_value = from_cursor.get_value(from_memory);

        if from_value.get_addr_value() == 0 {
            return Ok(to_cursor) 
        }

        to_cursor = Self::make_first_vtable(to_cursor, to_memory)?;
        let to_cursor_value = to_cursor.get_value(to_memory);
        let mut last_real_vtable = to_cursor_value.get_addr_value() as usize;
        let mut last_vtable_idx = 0usize;

        let c: Vec<NP_Struct_Field>;
        let col_schemas = match &from_memory.get_schema(from_cursor.schema_addr) {
            NP_Parsed_Schema::Struct { fields, .. } => {
                fields
            },
            _ => { c = Vec::new(); &c }
        };

        let mut struc = Self::new_iter(&from_cursor, from_memory);

        while let Some((idx, _key, item)) = struc.step_iter(from_memory) {
           if let Some(real) = item {

                let v_table =  idx / VTABLE_SIZE; // which vtable
                let v_table_idx = idx % VTABLE_SIZE; // which index on the selected vtable
                
                if last_vtable_idx < v_table {
                    let vtable_data = Self::get_vtable(last_real_vtable, to_memory);
                    last_real_vtable = Self::make_next_vtable(vtable_data, to_memory)?;
                    last_vtable_idx += 1;
                }

                let item_addr = last_real_vtable + (v_table_idx * 2);
                NP_Cursor::compact(depth + 1, real.clone(), from_memory, NP_Cursor::new(item_addr, col_schemas[idx].schema, to_cursor.schema_addr), to_memory)?;
            }         
        }

        Ok(to_cursor)
    }

    fn schema_to_idl(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<String, NP_Error> {
        match &schema[address] {
            NP_Parsed_Schema::Struct { fields, .. } => {
                let mut result = String::from("struct({fields: {");

                let last_index = fields.len() - 1;
                for (idx, field) in fields.iter().enumerate() {
                    result.push_str(field.col.as_str());
                    result.push_str(": ");
                    result.push_str(NP_Schema::_type_to_idl(schema, field.schema)?.as_str());
                    if idx < last_index {
                        result.push_str(", ");
                    }
                }

                result.push_str("}})");
                Ok(result)
            },  
            _ => { Err(NP_Error::Unreachable) }
        }
    }

    fn from_idl_to_schema(mut schema: Vec<NP_Parsed_Schema>, _name: &str, idl: &JS_Schema, args: &Vec<JS_AST>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::Struct as u8);

        let schema_table_addr = schema.len();
        schema.push(NP_Parsed_Schema::Struct {
            val: NP_Value_Kind::Pointer,
            empty: Vec::new(),
            i: NP_TypeKeys::Struct,
            sortable: false,
            fields: Vec::new()
        });

        let mut fields: Vec<NP_Struct_Field> = Vec::new();

        let mut field_data: Vec<(String, Vec<u8>)> = Vec::new();

        let mut schema_parsed: Vec<NP_Parsed_Schema> = schema;

        let mut idl_fields: Option<&Vec<(AST_STR, JS_AST)>> = None;

        if args.len() > 0 {
            match &args[0] {
                JS_AST::object { properties } => {
                    for (key, value) in properties {
                        match idl.get_str(key).trim() {
                            "fields" => {
                                match value {
                                    JS_AST::object { properties } => {
                                        idl_fields = Some(properties);
                                    },
                                    _ => { }
                                }
                            },
                            "columns" => {
                                match value {
                                    JS_AST::object { properties } => {
                                        idl_fields = Some(properties);
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

        if let Some(ast_fields) = idl_fields {

            let mut x: u8 = 0;
            for col in ast_fields {
                let field_name = idl.get_str(&col.0).trim();
                if field_name.len() > 255 {
                    return Err(NP_Error::new("Struct field names cannot be longer than 255 characters!"))
                }
    
                let field_schema_addr = schema_parsed.len();
                // fields.push((x, String::from(field_name), field_schema_addr));
                fields.push(NP_Struct_Field { idx: x as u8, col: String::from(field_name), schema: field_schema_addr, offset: 0});
                let (_is_sortable, field_type, schema_p) = NP_Schema::from_idl(schema_parsed, idl, &col.1)?;
                schema_parsed = schema_p;
                field_data.push((String::from(field_name), field_type));
                x += 1;
            }
    
            schema_parsed[schema_table_addr] = NP_Parsed_Schema::Struct {
                val: NP_Value_Kind::Pointer,
                empty: Vec::new(),
                i: NP_TypeKeys::Struct,
                sortable: false,
                fields: fields,
            };
    
            if field_data.len() > 255 {
                return Err(NP_Error::new("Structs cannot have more than 255 fields!"))
            }
    
            if field_data.len() == 0 {
                return Err(NP_Error::new("Structs must have at least one field!"))
            }
    
            // number of fields
            schema_bytes.push(field_data.len() as u8);
    
            for col in field_data {
                // colum name
                let bytes = col.0.as_bytes().to_vec();
                schema_bytes.push(bytes.len() as u8);
                schema_bytes.extend(bytes);
    
                if col.1.len() > u16::MAX as usize {
                    return Err(NP_Error::new("Schema overflow error!"))
                }
                
                // field type
                schema_bytes.extend((col.1.len() as u16).to_be_bytes().to_vec());
                schema_bytes.extend(col.1);
            }
    
            Ok((false, schema_bytes, schema_parsed))
        } else {
            Err(NP_Error::new("Structs require a 'fields' property that is an array of schemas!"))
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_bytes: Vec<u8> = Vec::new();
        schema_bytes.push(NP_TypeKeys::Struct as u8);

        let schema_table_addr = schema.len();
        schema.push(NP_Parsed_Schema::Struct {
            val: NP_Value_Kind::Pointer,
            empty: Vec::new(),
            i: NP_TypeKeys::Struct,
            sortable: false,
            fields: Vec::new()
        });

        let mut fields: Vec<NP_Struct_Field> = Vec::new();

        let mut field_data: Vec<(String, Vec<u8>)> = Vec::new();

        let mut schema_parsed: Vec<NP_Parsed_Schema> = schema;

        let json_fields = if let NP_JSON::Array(fields) = &json_schema["fields"] {
            fields
        } else if let NP_JSON::Array(fields) = &json_schema["columns"] {
            fields
        } else {
            return Err(NP_Error::new("Structs require a 'fields' property that is an array of schemas!"))
        };

 
        let mut x: u8 = 0;
        for col in json_fields {
            let field_name = match &col[0] {
                NP_JSON::String(x) => x.clone(),
                _ => "".to_owned()
            };
            if field_name.len() > 255 {
                return Err(NP_Error::new("Struct field names cannot be longer than 255 characters!"))
            }

            let field_schema_addr = schema_parsed.len();
            // fields.push((x, field_name.clone(), field_schema_addr));
            fields.push(NP_Struct_Field { idx: x as u8, col: field_name.clone(), schema: field_schema_addr, offset: 0});
            let (_is_sortable, field_type, schema_p) = NP_Schema::from_json(schema_parsed, &Box::new(col[1].clone()))?;
            schema_parsed = schema_p;
            field_data.push((field_name, field_type));
            x += 1;
        }

        schema_parsed[schema_table_addr] = NP_Parsed_Schema::Struct {
            val: NP_Value_Kind::Pointer,
            empty: Vec::new(),
            i: NP_TypeKeys::Struct,
            sortable: false,
            fields: fields,
        };

        if field_data.len() > 255 {
            return Err(NP_Error::new("Structs cannot have more than 255 fields!"))
        }

        if field_data.len() == 0 {
            return Err(NP_Error::new("Structs must have at least one field!"))
        }

        // number of fields
        schema_bytes.push(field_data.len() as u8);

        for col in field_data {
            // colum name
            let bytes = col.0.as_bytes().to_vec();
            schema_bytes.push(bytes.len() as u8);
            schema_bytes.extend(bytes);

            if col.1.len() > u16::MAX as usize {
                return Err(NP_Error::new("Schema overflow error!"))
            }
            
            // field type
            schema_bytes.extend((col.1.len() as u16).to_be_bytes().to_vec());
            schema_bytes.extend(col.1);
        }

        return Ok((false, schema_bytes, schema_parsed))
   
    }

    fn default_value(_depth: usize, _addr: usize, _schema: &Vec<NP_Parsed_Schema>) -> Option<Self> {
        None
    }
}


#[test]
fn schema_parsing_works_idl() -> Result<(), NP_Error> {
    let schema = r#"struct({fields: {age: u8(), tags: list({of: string()}), name: string({size: 10})}})"#;
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_idl()?);
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_idl()?);
    Ok(())
}


#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"struct","fields":[["age",{"type":"uint8"}],["tags",{"type":"list","of":{"type":"string"}}],["name",{"type":"string","size":10}]]}"#;
    let factory = crate::NP_Factory::new_json(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    let factory2 = crate::NP_Factory::new_bytes(factory.export_schema_bytes())?;
    assert_eq!(schema, factory2.schema.to_json()?.stringify());
    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = r#"{"type":"struct","fields":[["age",{"type":"uint8"}],["name",{"type":"string"}]]}"#;
    let factory = crate::NP_Factory::new_json(schema)?;

    // compaction removes cleared values
    let mut buffer = factory.new_buffer(None);
    buffer.set(&["name"], "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.after_compaction, buffer.calc_bytes()?.current_buffer);
    assert_eq!(buffer.calc_bytes()?.after_compaction, 21usize);
    buffer.del(&[])?;
    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    // good values are preserved through compaction
    let mut buffer = factory.new_buffer(None);
    buffer.set(&crate::np_path!("name"), "hello")?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 21usize);
    buffer.compact(None)?;
    assert_eq!(buffer.get::<&str>(&["name"])?, Some("hello"));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 21usize);

    // println!("{:?}", buffer.read_bytes());
    // let packed = factory.pack_buffer(buffer);
    // println!("{:?}", packed.schema.to_json()?.stringify().len());
    // println!("{:?}", packed.export_schema_bytes().len());
    // let closed = packed.close_packed();

    // let opened = NP_Packed_Buffer::open(closed)?;
    // println!("{:?}", opened.get::<&str>(&["name"])?);

    Ok(())
}


#[test]
fn test_vtables() -> Result<(), NP_Error> {
    let factory = crate::NP_Factory::new_json(r#"{
        "type": "struct",
        "fields": [
            ["age",    {"type": "u8"}],
            ["name",   {"type": "string"}],
            ["color",  {"type": "string"}],
            ["car",    {"type": "string"}],
            ["rating", {"type": "u8"}]
        ]
    }"#)?;

    // compaction removes cleared values
    let mut buffer = factory.new_buffer(None);
    buffer.set(&["age"], 20u8)?;
    buffer.set(&["name"], "hello")?;
    buffer.set(&["color"], "blue")?;
    buffer.set(&["car"], "Chevy")?;
    buffer.set(&["rating"], 98u8)?;

    let mut new_buffer = factory.open_buffer(buffer.finish().bytes());
    assert_eq!(new_buffer.get::<u8>(&["age"])?.unwrap(), 20u8);
    assert_eq!(new_buffer.get::<&str>(&["name"])?.unwrap(), "hello");
    assert_eq!(new_buffer.get::<&str>(&["color"])?.unwrap(), "blue");
    assert_eq!(new_buffer.get::<&str>(&["car"])?.unwrap(), "Chevy");
    assert_eq!(new_buffer.get::<u8>(&["rating"])?.unwrap(), 98u8);

    new_buffer.set_with_json(&[], r#"{"value": {
        "age": 50, 
        "name": "Jimmy", 
        "color": "orange", 
        "car": "Audi", 
        "rating": 20
    }}"#)?;

    assert_eq!(new_buffer.get::<u8>(&["age"])?.unwrap(), 50u8);
    assert_eq!(new_buffer.get::<&str>(&["name"])?.unwrap(), "Jimmy");
    assert_eq!(new_buffer.get::<&str>(&["color"])?.unwrap(), "orange");
    assert_eq!(new_buffer.get::<&str>(&["car"])?.unwrap(), "Audi");
    assert_eq!(new_buffer.get::<u8>(&["rating"])?.unwrap(), 20u8);

    Ok(())
}