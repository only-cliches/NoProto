use core::hint::unreachable_unchecked;

use crate::{json_flex::JSMAP};
use crate::pointer::NP_Ptr;
use crate::pointer::{NP_Value};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Parsed_Schema}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};

/// Tuple data type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug, Clone)]
pub struct NP_Tuple<'tuple> {
    address: usize, // pointer location
    memory: &'tuple NP_Memory,
    schema: &'tuple Box<NP_Parsed_Schema>,
    value_addrs: Vec<usize>
}

impl<'tuple> NP_Tuple<'tuple> {

    #[doc(hidden)]
    pub fn new(address: usize, memory: &'tuple NP_Memory, schema: &'tuple Box<NP_Parsed_Schema>, value_addrs: Vec<usize>) -> Self {
        NP_Tuple {
            address,
            memory: memory,
            schema: schema,
            value_addrs: value_addrs
        }
    }

    /// read schema of tuple
    pub fn get_schema(&self) -> &'tuple Box<NP_Parsed_Schema> {
        self.schema
    }

    /// Select a value at the given index
    pub fn select(&self, index: u8) -> Result<NP_Ptr<'tuple>, NP_Error> {
        NP_Tuple::select_mv(self.clone(), index)
    }

    /// Select a value at a given index in the tuple
    pub fn select_mv(self, index: u8) -> Result<NP_Ptr<'tuple>, NP_Error> {

        let values = &self.value_addrs;

        let indexu = index as usize;

        let addr = self.address;

        if index as usize > values.len() {
            return Err(NP_Error::new("Attempted to access tuple value outside length!"));
        }

        let rc_memory = self.memory;

        let location = match rc_memory.size {
            NP_Size::U8 => {addr + (indexu * 1) },
            NP_Size::U16 => {addr + (indexu * 2) },
            NP_Size::U32 => {addr + (indexu * 4) } 
        };

        let object_schema = match &**self.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values } => {
                &values[indexu]
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        Ok(NP_Ptr::_new_standard_ptr(location, object_schema, rc_memory))
    }

    /// Convert the tuple into an iterator
    pub fn it(self) -> NP_Tuple_Iterator<'tuple> {
        NP_Tuple_Iterator::new(self)
    }

    /// Get the length of the tuple, includes empty items
    pub fn len(&self) -> u8 {
        match &**self.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values } => {
                values.len() as u8
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    /// Remove all values from the tuple
    pub fn clear(self) -> Self {

        let addr = self.address as u32;

        let length = self.value_addrs.len();

        let memory = self.memory;

        let write_bytes = memory.write_bytes();

        let byte_count = match memory.size {
            NP_Size::U32 => length * 4,
            NP_Size::U16 => length * 2,
            NP_Size::U8 => length * 1
        };

        for x in 0..byte_count {
            write_bytes[(addr + x as u32) as usize] = 0;
        }

        NP_Tuple {
            address: self.address,
            memory: self.memory,
            schema: self.schema,
            value_addrs: self.value_addrs
        }
    }

}

impl<'tuple> NP_Value<'tuple> for NP_Tuple<'tuple> {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Tuple as u8, "tuple".to_owned(), NP_TypeKeys::Tuple) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Tuple as u8, "tuple".to_owned(), NP_TypeKeys::Tuple) }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

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

    fn set_value(_pointer: &mut NP_Ptr<'tuple>, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Ptr<'tuple>) -> Result<Option<Box<Self>>, NP_Error> {

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
                    let mut ptr = NP_Ptr::_new_standard_ptr(value_addrs[x], ptr.schema, &ptr.memory);
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

        Ok(Some(Box::new(NP_Tuple {
            address: addr,
            memory: ptr.memory,
            schema: ptr.schema,
            value_addrs: value_addrs
        })))
    }

    fn get_size(ptr: &'tuple NP_Ptr<'tuple>) -> Result<usize, NP_Error> {
        
        let base_size = 0usize;

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let tuple = NP_Tuple::into_value(ptr.clone())?.expect("Tried to convert non Tuple into tuple!");

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

    fn to_json(ptr: &'tuple NP_Ptr<'tuple>) -> NP_JSON {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let tuple = NP_Tuple::into_value(ptr.clone()).unwrap().expect("Tried to conver tuple into non tuple!");

        let mut json_list = Vec::new();

        for mut l in tuple.it().into_iter() {
            let ptr = l.select().unwrap();
            json_list.push(ptr.json_encode());
        }

        NP_JSON::Array(json_list)
        
    }

    fn do_compact(from_ptr: NP_Ptr<'tuple>, to_ptr: &'tuple mut NP_Ptr<'tuple>) -> Result<(), NP_Error> where Self: NP_Value<'tuple> {

        if from_ptr.address == 0 {
            return Ok(());
        }

        let old_tuple = Self::into_value(from_ptr)?.unwrap();
        let new_tuple = Self::into_value(to_ptr.clone())?.unwrap();
 
        for mut item in old_tuple.it().into_iter() {

            if item.has_value {
                let mut new_ptr = new_tuple.select(item.index as u8)?;
                let old_ptr = item.select()?;
                old_ptr.clone().compact(&mut new_ptr)?;
            }
        }

        Ok(())
    }


    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

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

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
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


/// Tuple iterator data type
#[derive(Debug)]
pub struct NP_Tuple_Iterator<'it> {
    tuple: NP_Tuple<'it>,
    current_index: u16,
}

impl<'it> NP_Tuple_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(tuple: NP_Tuple<'it>) -> Self {
        NP_Tuple_Iterator {
            tuple: tuple,
            current_index: 0
        }
    }

    /// Convert the iterator back into a tuple
    pub fn into_tuple(self) -> NP_Tuple<'it> {
        self.tuple
    }
}

impl<'it> Iterator for NP_Tuple_Iterator<'it> {
    type Item = NP_Tuple_Item<'it>;

    fn next(&mut self) -> Option<Self::Item> {

        if (self.current_index as usize) >= self.tuple.value_addrs.len() || self.tuple.value_addrs.len() == 0 {
            return None;
        }

        let this_index = self.current_index as usize;
        self.current_index += 1;

        let value_schema = match &**self.tuple.schema {
            NP_Parsed_Schema::Tuple { i: _, sortable: _, values } => {
                &values[this_index as usize]
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let values = &self.tuple.value_addrs;
        
        Some(NP_Tuple_Item {
            index: this_index as u16,
            has_value: if values.len() > this_index { values[this_index] != 0 } else { false },
            address: if values.len() > this_index { values[this_index] } else { 0 },
            memory: &self.tuple.memory,
            schema: value_schema
        })
    }
}

/// A single iterator item
#[derive(Debug)]
pub struct NP_Tuple_Item<'item> { 
    /// The index of this item in the list
    pub index: u16,
    /// If there is a value at this index
    pub has_value: bool,
    address: usize,
    memory: &'item NP_Memory,
    schema: &'item Box<NP_Parsed_Schema>,
}

impl<'item> NP_Tuple_Item<'item> {

    /// Get the pointer at this iterator location
    pub fn select(&mut self) -> Result<NP_Ptr<'item>, NP_Error> {

        Ok(NP_Ptr::_new_standard_ptr(self.address, self.schema, &self.memory))
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
    buffer.set("0", String::from("hello"))?;
    assert_eq!(buffer.get::<String>("0")?, Some(Box::new(String::from("hello"))));
    buffer.del("")?;
    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}