use crate::{json_flex::JSMAP, pointer::any::NP_Any};
use crate::pointer::NP_Ptr;
use crate::pointer::{NP_PtrKinds, NP_Value, NP_Lite_Ptr};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_Schema, NP_TypeKeys, NP_Schema_Ptr}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::{boxed::Box};

/// Tuple data type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug)]
pub struct NP_Tuple<'tuple> {
    address: u32, // pointer location
    memory: Option<&'tuple NP_Memory>,
    schema: Option<NP_Schema_Ptr<'tuple>>,
    value_addrs: Option<Vec<u32>>
}
/// The struct that represents the schema for Tuples
#[derive(Debug)]
pub struct NP_Tuple_Schema_State<'tuple> {
    sorted: bool,
    schemas: Vec<(u8, NP_Schema_Ptr<'tuple>)>
}

impl<'tuple> NP_Tuple<'tuple> {

    #[doc(hidden)]
    pub fn new(address: u32, memory: &'tuple NP_Memory, schema: NP_Schema_Ptr<'tuple>, value_addrs: Vec<u32>) -> Self {
        NP_Tuple {
            address,
            memory: Some(memory),
            schema: Some(schema),
            value_addrs: Some(value_addrs)
        }
    }

    /// Convert schema bytes into Struct
    /// 
    #[doc(hidden)]
    pub fn get_schema_state(schema_ptr: NP_Schema_Ptr<'tuple>) -> NP_Tuple_Schema_State<'tuple> {

        let is_sorted = schema_ptr.schema.bytes[schema_ptr.address + 1];

        let column_len = schema_ptr.schema.bytes[schema_ptr.address + 2];

        let mut schemas: Vec<(u8, NP_Schema_Ptr)> = Vec::new();

        let mut offset = schema_ptr.address + 3;

        for x in 0..column_len as usize {

            let schema_size = u16::from_be_bytes([
                schema_ptr.schema.bytes[offset],
                schema_ptr.schema.bytes[offset + 1]
            ]) as usize;

            schemas.push((x as u8, schema_ptr.copy_with_addr(schema_ptr.address + offset + 2)));

            offset += schema_size + 2;
        }

        NP_Tuple_Schema_State { schemas: schemas, sorted: is_sorted != 0 }
    }

    /// Select a value at a given index in the tuple
    pub fn select<T: NP_Value<'tuple> + Default>(&self, index: u8) -> Result<NP_Ptr<'tuple, T>, NP_Error> {

        let values = self.value_addrs.as_ref().unwrap();

        let addr = self.address;

        if index as usize > values.len() {
            return Err(NP_Error::new("Attempted to access tuple value outside length!"));
        }

        let schema_ptr = self.schema.as_ref().unwrap().copy();
        let schema_type = schema_ptr.schema.bytes[schema_ptr.address];
    

        // match type casting
        if T::type_idx().0 != NP_TypeKeys::Any as u8 && schema_type != NP_TypeKeys::Any as u8  {

            // not using ANY casting, check type
            if schema_type != T::type_idx().0 {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(T::type_idx().1.as_str());
                err.push_str(") to schema of type (");
                err.push_str(NP_TypeKeys::from(schema_type).into_type_idx().1.as_str());
                err.push_str(")");
                return Err(NP_Error::new(err));
            }
        }

        let rc_memory = self.memory.unwrap();

        let location = match rc_memory.size {
            NP_Size::U8 => {addr + (index as u32 * 1) },
            NP_Size::U16 => {addr + (index as u32 * 2) },
            NP_Size::U32 => {addr + (index as u32 * 4) } 
        };

        let schema_state = NP_Tuple::get_schema_state(schema_ptr.copy());

        Ok(NP_Ptr::_new_standard_ptr(location, schema_state.schemas[index as usize].1.copy(), rc_memory))
    }

    /// Convert the tuple into an iterator
    pub fn it(self) -> NP_Tuple_Iterator<'tuple> {
        NP_Tuple_Iterator::new(self.address, self.memory.unwrap(), self.schema.unwrap(), self.value_addrs.unwrap())
    }

    /// Get the length of the tuple, includes empty items
    pub fn len(&self) -> u8 {
        let schema_vec = self.schema.as_ref().unwrap();
        let schema_len = schema_vec.schema.bytes[schema_vec.address + 1];

        schema_len
    }

    /// Remove all values from the tuple
    pub fn clear(self) -> Self {

        let addr = self.address as u32;

        let length = match &self.value_addrs {
            Some(x) => x.len(),
            None => 0
        };

        let memory = self.memory.unwrap();

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

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Tuple as u8, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Tuple as u8, "tuple".to_owned()) }

    fn schema_to_json(schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = NP_Tuple::get_schema_state(schema_ptr.copy());

        let columns: Vec<NP_JSON> = schema_state.schemas.into_iter().map(|column| {
            NP_Schema::_type_to_json(&column.1).unwrap()
        }).collect();

        schema_json.insert("values".to_owned(), NP_JSON::Array(columns));

        if schema_state.sorted {
            schema_json.insert("sorted".to_owned(), NP_JSON::True);
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Lite_Ptr<'tuple>) -> Result<Option<Box<Self>>, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let tuple_schema = NP_Tuple::get_schema_state(ptr.schema.copy());

        let tuple_size = tuple_schema.schemas.len();

        let mut value_addrs: Vec<u32> = Vec::new();

        if addr == 0 { // no tuple yet, make one
    
            let length = match &ptr.memory.size {
                NP_Size::U8  => 1 * tuple_size,
                NP_Size::U16 => 2 * tuple_size,
                NP_Size::U32 => 4 * tuple_size
            };

            let mut addresses = Vec::with_capacity(length);

            for _x in 0..length {
                addresses.push(0);
            }

            addr = ptr.memory.malloc(addresses)?; // stores value addresses
            ptr.memory.set_value_address(ptr.location, addr, &ptr.kind);

            if tuple_schema.sorted { // write default values in sorted order
                for x in 0..value_addrs.len() as usize {
                    let ptr = NP_Ptr::<NP_Any>::_new_standard_ptr(value_addrs[x], ptr.schema.copy(), &ptr.memory);
                    ptr.set_default()?;
                }
            }

        } else { // tuple exists
            for x in 0..tuple_size as u32 {
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
            memory: Some(ptr.memory),
            schema: Some(ptr.schema.copy()),
            value_addrs: Some(value_addrs)
        })))
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        
        let base_size = 0u32;

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        let tuple = NP_Tuple::into_value(ptr.clone())?.unwrap();

        let mut acc_size = 0u32;

        for mut l in tuple.it().into_iter() {
            if l.has_value == true {
                let ptr = l.select::<NP_Any>()?;
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

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return NP_JSON::Null;
        }

        let tuple = NP_Tuple::into_value(ptr).unwrap_or(Some(Box::new(NP_Tuple::default()))).unwrap_or(Box::new(NP_Tuple::default()));

        let mut json_list = Vec::new();

        for mut l in tuple.it().into_iter() {
            if l.has_value == true {
                let ptr = l.select::<NP_Any>();
                match ptr {
                    Ok(p) => {
                        json_list.push(p.json_encode());
                    },
                    Err (_e) => {
                        json_list.push(NP_JSON::Null);
                    }
                }
            } else {
                json_list.push(NP_JSON::Null);
            }
        }

        NP_JSON::Array(json_list)
        
    }

    fn do_compact(from_ptr: NP_Lite_Ptr<'tuple>, to_ptr: NP_Lite_Ptr<'tuple>) -> Result<(), NP_Error> where Self: NP_Value<'tuple> + Default {

        if from_ptr.location == 0 {
            return Ok(());
        }

        let to_ptr_list = to_ptr.into::<Self>();

        match Self::into_value(from_ptr)? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(new_tuple) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value {
                                let new_ptr = NP_Lite_Ptr::from(new_tuple.select::<NP_Any>(item.index as u8)?);
                                let old_ptr = NP_Lite_Ptr::from(item.select::<NP_Any>()?);
                                old_ptr.compact(new_ptr)?;
                            }

                        }
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok(())
    }


    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "tuple" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Tuple as u8);

            match json_schema["sorted"] {
                NP_JSON::True => {
                    schema_data.push(1);
                },
                _ => {
                    schema_data.push(0);
                }
            }

            let mut schemas: Vec<Vec<u8>> = Vec::new();

            match &json_schema["values"] {
                NP_JSON::Array(cols) => {
                    for col in cols {
                        let column_type = NP_Schema::from_json(Box::new(col.clone()))?;
                        schemas.push(column_type.bytes);
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

            return Ok(Some(schema_data))
        }

        Ok(None)
    }
}

impl<'tuple> Default for NP_Tuple<'tuple> {

    fn default() -> Self {
        NP_Tuple { address: 0, memory: None, schema: None, value_addrs: None}
    }
}

/// Tuple iterator data type
#[derive(Debug)]
pub struct NP_Tuple_Iterator<'it> {
    address: u32, // pointer location
    memory: &'it NP_Memory,
    current_index: u16,
    schemas: NP_Schema_Ptr<'it>,
    values: Vec<u32>
}

impl<'it> NP_Tuple_Iterator<'it> {

    #[doc(hidden)]
    pub fn new(address: u32, memory: &'it NP_Memory, schema: NP_Schema_Ptr<'it>, values: Vec<u32>) -> Self {
        NP_Tuple_Iterator {
            address,
            memory,
            current_index: 0,
            schemas: schema,
            values: values
        }
    }

    /// Convert the iterator back into a tuple
    pub fn into_tuple(self) -> NP_Tuple<'it> {
        NP_Tuple::new(self.address, self.memory, self.schemas, self.values)
    }
}

impl<'it> Iterator for NP_Tuple_Iterator<'it> {
    type Item = NP_Tuple_Item<'it>;

    fn next(&mut self) -> Option<Self::Item> {

        if (self.current_index as usize) > self.values.len() {
            return None;
        }

        let this_index = self.current_index;
        self.current_index += 1;

        let tuple_schema = NP_Tuple::get_schema_state(self.schemas.copy());
        
        Some(NP_Tuple_Item {
            index: this_index,
            has_value: self.values[this_index as usize] != 0,
            address: self.values[this_index as usize],
            memory: &self.memory,
            schema: tuple_schema.schemas[this_index as usize].1.copy()
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
    address: u32,
    memory: &'item NP_Memory,
    schema: NP_Schema_Ptr<'item>,
}

impl<'item> NP_Tuple_Item<'item> {

    /// Get the pointer at this iterator location
    pub fn select<T: NP_Value<'item> + Default>(&mut self) -> Result<NP_Ptr<'item, T>, NP_Error> {

        Ok(NP_Ptr::_new_standard_ptr(self.address, self.schema.copy(), &self.memory))
    }
}
