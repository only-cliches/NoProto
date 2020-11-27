use crate::collection::NP_Collection;
use alloc::rc::Rc;
use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP, pointer::{NP_Cursor_Addr}};
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};

/// Tuple data type.
/// 
#[doc(hidden)]
#[derive(Debug, Clone)]
pub struct NP_Tuple<'tuple> {
    tuple_cursor: NP_Cursor_Addr,
    schema: &'tuple Vec<Box<NP_Parsed_Schema<'tuple>>>,
    current_index: usize,
    current: Option<NP_Cursor_Addr>,
    pub memory: &'tuple NP_Memory<'tuple>
}

impl<'tuple> NP_Tuple<'tuple> {

    pub fn cache_tuple_item<'cache>(item_addr: &NP_Cursor_Addr, index: usize, item_schema: &Box<NP_Parsed_Schema>, memory: &NP_Memory) -> Result<(), NP_Error> {

    }

    pub fn commit_or_cache_tuple(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<(), NP_Error> {
        let mut addr = ptr.kind.get_value_addr();

        let tuple_data = match &**ptr.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable, values } => {
                (*sortable, values.len())
            },
            _ => { panic!() }
        };

        let mut value_addrs: Vec<usize> = Vec::new();

        if addr == 0 { // no tuple yet, make one
    
            let length = match &ptr.memory.size {
                NP_Size::U8  => 1 * tuple_data.1,
                NP_Size::U16 => 2 * tuple_data.1,
                NP_Size::U32 => 4 * tuple_data.1
            };

            let mut addresses = Vec::with_capacity(length);

            for _x in 0..length {
                addresses.push(0);
            }

            addr = ptr.memory.malloc(addresses)?; // stores value addresses
            ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);

            if tuple_data.0 { // write default values in sorted order
                for x in 0..value_addrs.len() as usize {
                    let mut ptr = NP_Ptr::_new_standard_ptr(value_addrs[x], ptr.schema, (&ptr.memory));
                    ptr.set_default()?;
                }
            }

        } else { // tuple exists
            for x in 0..tuple_data.1 as usize {
                match &ptr.memory.size {
                    NP_Size::U8 => {
                        value_addrs.push(addr + (x * 1));
                    },
                    NP_Size::U16 => {
                        value_addrs.push(addr + (x * 2));
                    },
                    NP_Size::U32 => {
                        value_addrs.push(addr + (x * 4));
                    }
                };
            }
        }

        Ok(NP_Tuple {
            address: addr,
            memory: (&ptr.memory),
            schema: ptr.schema,
            value_addrs: value_addrs
        })
    }

    /// Select into pointer
    pub fn select_to_ptr(cursor_addr: NP_Cursor_Addr, memory: &'tuple NP_Memory, index: usize) -> Result<Option<NP_Cursor_Addr>, NP_Error> {

        let tuple = Self::ptr_to_self(target_ptr)?;

        let values = &tuple.value_addrs;

        let indexu = index as usize;

        if indexu > values.len() {
            return Err(NP_Error::new("Attempted to access tuple value outside length!"));
        }

        let location = values[indexu];

        let object_schema = match &**target_ptr.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values } => {
                &values[indexu]
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        target_ptr.address = location;
        target_ptr.schema = object_schema;
        target_ptr.helper = NP_Iterator_Helper::None;
        target_ptr.parent = NP_Ptr_Collection::Tuple { address: tuple.address, length: tuple.len() as usize, schema: tuple.schema};
        target_ptr.kind = NP_Ptr::read_kind(target_ptr.address, (&target_ptr.memory), &target_ptr.parent);
        Ok(())
    }

}

impl<'value> NP_Value<'value> for NP_Tuple<'value> {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("tuple", NP_TypeKeys::Tuple) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema<'value>>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        let schema_state: (bool, Vec<NP_JSON>) = match schema_ptr {
            NP_Parsed_Schema::Tuple { i: _, sortable, values } => {
                (*sortable, values.into_iter().map(|column| {
                    NP_Schema::_type_to_json(column).unwrap()
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

    fn set_value(cursor_addr: NP_Cursor_Addr, memory: NP_Memory, value: &Self) -> Result<NP_Cursor_Addr, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()!"))
    }

    fn get_size(cursor_addr: NP_Cursor_Addr, _memory: &'value NP_Memory) -> Result<usize, NP_Error> {
        
        let base_size = 0usize;

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let tuple = NP_Tuple::ptr_to_self(&ptr).expect("Tried to convert non Tuple into tuple!");

        let mut acc_size = 0usize;

        for mut l in tuple.it().into_iter() {
            if l.has_value == true {
                let ptr = l.select()?;
                acc_size += ptr.calc_size()?;
            } else {
                // empty pointer
                acc_size += match &ptr.memory.size {
                    NP_Size::U8  => 1,
                    NP_Size::U16 => 2,
                    NP_Size::U32 => 4
                };
            }
        }

        return Ok(base_size + acc_size);
    }

    fn to_json(cursor_addr: NP_Cursor_Addr, _memory: &'value NP_Memory) -> NP_JSON {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let tuple = NP_Tuple::ptr_to_self(&ptr).expect("Tried to convert tuple into non tuple!");

        let mut json_list = Vec::new();

        for mut l in tuple.it().into_iter() {
            let ptr = l.select().unwrap();
            json_list.push(ptr.json_encode());
        }

        NP_JSON::Array(json_list)
        
    }

    fn do_compact(from_cursor: NP_Cursor_Addr, from_memory: &'value NP_Memory, to_cursor: NP_Cursor_Addr, to_memory: &'value NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> where Self: NP_Value<'value> {

        if from_ptr.address == 0 {
            return Ok(());
        }

        let old_tuple = Self::ptr_to_self(&from_ptr)?;
        let new_tuple = Self::ptr_to_self(to_ptr)?;
 
        for mut item in old_tuple.it().into_iter() {

            if item.has_value {
                let mut new_ptr = new_tuple.select(item.index as u8)?;
                let old_ptr = item.select()?;
                old_ptr.compact(&mut new_ptr)?;
            }
        }

        Ok(())
    }


    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, json_schema: &'value NP_JSON) -> Result<Option<(Vec<u8>, Vec<NP_Parsed_Schema<'value>>)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "tuple" == type_str {
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

            let mut schemas: Vec<Vec<u8>> = Vec::new();
            let mut parsed_schemas: Vec<Box<NP_Parsed_Schema>> = Vec::new();

            match &json_schema["values"] {
                NP_JSON::Array(cols) => {
                    for col in cols {
                        let column_type = NP_Schema::from_json(Box::new(col.clone()))?;
                        if sorted && column_type.1.is_sortable() == false {
                            return Err(NP_Error::new("All children of a sorted tuple must be sortable items!"))
                        }
                        schemas.push(column_type.0);
                        parsed_schemas.push(Box::new(column_type.1));
                    }
                },
                _ => { 
                    return Err(NP_Error::new("Tuples require a 'values' property that is an array of schemas!"))
                }
            }

            if schemas.len() > 255 {
                return Err(NP_Error::new("Tuples cannot have more than 255 values!"))
            }

            // number of schema values
            schema_data.push(schemas.len() as u8);

            for col in schemas {

                if col.len() > u16::max as usize {
                    return Err(NP_Error::new("Schema overflow error!"))
                }
                
                // column type
                schema_data.extend((col.len() as u16).to_be_bytes().to_vec());
                schema_data.extend(col);
            }

            return Ok(Some((schema_data, NP_Parsed_Schema::Tuple {
                i: NP_TypeKeys::Tuple,
                sortable: sorted,
                values: parsed_schemas
            })))
        }

        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<&'value Self> {
        None
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema<'value>>, address: usize, bytes: &'value Vec<u8>) -> Vec<NP_Parsed_Schema<'value>> {
        let is_sorted = bytes[address + 1];

        let column_len = bytes[address + 2];

        let mut schemas: Vec<Box<NP_Parsed_Schema>> = Vec::new();

        let mut offset = address + 3;

        for _x in 0..column_len as usize {

            let schema_size = u16::from_be_bytes([
                bytes[offset],
                bytes[offset + 1]
            ]) as usize;

            schemas.push(Box::new(NP_Schema::from_bytes(offset + 2, bytes)));

            offset += schema_size + 2;
        }

        NP_Parsed_Schema::Tuple {
            i: NP_TypeKeys::Tuple,
            values: schemas, 
            sortable: is_sorted != 0 
        }
    }
}

impl<'collection> NP_Collection<'collection> for NP_Tuple<'collection> {
    fn start_iter(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<Self, NP_Error> {
        
        NP_Tuple::commit_or_cache_tuple(&cursor_addr, memory)?;

        let tuple_cursor = cursor_addr.get_data(&memory)?;

        let schema_values = match &**tuple_cursor.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values} => values,
            _ => panic!()
        };

        if cursor_addr.is_virtual || tuple_cursor.address == 0 { 
            return Ok(NP_Tuple {
                tuple_cursor: cursor_addr.clone(),
                schema: schema_values,
                current_index: 0,
                current: None,
                memory: memory
            });
        } else {
            return Ok(NP_Tuple {
                tuple_cursor: cursor_addr.clone(),
                schema: schema_values,
                current_index: 0,
                current: Some(NP_Cursor_Addr { address: tuple_cursor.address, is_virtual: false}),
                memory: memory
            });
        }
    }

    fn step_pointer(&self, cursor_addr: &NP_Cursor_Addr) -> Option<NP_Cursor_Addr> { panic!() }
    fn commit_pointer(cursor_addr: &NP_Cursor_Addr, memory: &NP_Memory) -> Result<NP_Cursor_Addr, NP_Error> { panic!() }
}

impl<'it> Iterator for NP_Tuple<'it> {
    type Item = NP_Cursor_Addr;

    fn next(&mut self) -> Option<Self::Item> {

        if (self.current_index as usize) >= self.schema.len() || self.schema.len() == 0 || self.tuple_cursor.is_virtual == true {
            return None;
        }

        let addr_size = self.memory.addr_size_bytes();

        let this_index = self.current_index as usize;
        self.current_index += 1;

        NP_Tuple::commit_or_cache_tuple(&self.tuple_cursor, &self.memory).unwrap();
        let tuple = self.memory.get_cursor_data(&self.tuple_cursor).unwrap();

        let current_addr = tuple.address_value + (self.current_index * addr_size);
        let current_cursor = NP_Cursor_Addr { address: current_addr, is_virtual: false};
        NP_Tuple::cache_tuple_item(&current_cursor, self.current_index, &self.schema[self.current_index], self.memory).unwrap();

        Some(current_cursor)
    }

    fn count(self) -> usize where Self: Sized {
        self.schema.len()
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
    buffer.set(&["0"], String::from("hello"))?;
    assert_eq!(buffer.get::<String>(&["0"])?, Some(Box::new(String::from("hello"))));
    assert_eq!(buffer.calc_bytes()?.current_buffer, 17usize);
    buffer.del(&[])?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}