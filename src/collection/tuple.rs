use crate::pointer::any::NP_Any;
use crate::pointer::NP_Ptr;
use crate::pointer::NP_ValueInto;
use crate::pointer::{NP_PtrKinds, NP_Value};
use crate::{memory::NP_Memory, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, error::NP_Error, json_flex::JFObject};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::boxed::Box;

pub struct NP_Tuple<'a> {
    address: u32, // pointer location
    memory: Option<&'a NP_Memory>,
    schemas: Option<&'a Vec<NP_Schema>>,
    values: Option<Vec<u32>>
}


impl<'a> NP_Tuple<'a> {

    #[doc(hidden)]
    pub fn new(address: u32, memory: &'a NP_Memory, schemas: &'a Vec<NP_Schema>, values: Vec<u32>) -> Self {
        NP_Tuple {
            address,
            memory: Some(memory),
            schemas: Some(schemas),
            values: Some(values)
        }
    }

    pub fn select<T: NP_Value + Default + NP_ValueInto<'a>>(&self, index: u8) -> Result<NP_Ptr<'a, T>, NP_Error> {

        let values = self.values.as_ref().unwrap();

        if index as usize > values.len() {
            return Err(NP_Error::new("Attempted to access tuple value outside length!"));
        }

        let schema_vec = *self.schemas.as_ref().unwrap();
        let schema: &NP_Schema = &schema_vec[index as usize];

        // match type casting
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && schema.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using ANY casting, check type
            if schema.type_data.0 != T::type_idx().0 {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(T::type_idx().1.as_str());
                err.push_str(") to schema of type (");
                err.push_str(schema.type_data.1.as_str());
                err.push_str(")");
                return Err(NP_Error::new(err));
            }
        }

        Ok(NP_Ptr::new_standard_ptr(values[index as usize], schema, self.memory.unwrap()))
    }

    pub fn it(self) -> NP_Tuple_Iterator<'a> {
        NP_Tuple_Iterator::new(self.address, self.memory.unwrap(), self.schemas.unwrap(), self.values.unwrap())
    }
/*
    pub fn delete(&mut self, index: u8) -> bool {
        match &mut self.values {
            Some(values) => {

                if index as usize > values.len() {
                    return false;
                }

                if values[index as usize] == 0 {
                    return false;
                }

                values[index as usize] = 0;

                let value_address = (self.address as u32 + (4u32 * index as u32)) as usize;
                let write_bytes = self.memory.unwrap().write_bytes();

                for x in 0..4 {
                    write_bytes[value_address + x] = 0;
                }

                true
            },
            None => { false }
        }
    }
*/
    pub fn len(&self) -> u8 {
        self.schemas.unwrap().len() as u8
    }

    pub fn clear(self) -> Self {

        let addr = self.address as u32;

        let length = self.values.unwrap().len();

        let write_bytes = self.memory.unwrap().write_bytes();

        let byte_count = (length * 4) as usize;

        for x in 0..byte_count {
            write_bytes[(addr + x as u32) as usize] = 0;
        }

        // create new empty addresses
        let mut addresses = Vec::with_capacity(4 * length);

        for x in 0..addresses.len() {
            addresses[x] = 0;
        }
        

        NP_Tuple {
            address: self.address,
            memory: self.memory,
            schemas: self.schemas,
            values: Some(addresses)
        }
    }

}

impl<'a> NP_Value for NP_Tuple<'a> {
    fn is_type(_type_str: &str) -> bool {  // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::Tuple as i64, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Tuple as i64, "tuple".to_owned()) }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a> NP_ValueInto<'a> for NP_Tuple<'a> {
    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<NP_Tuple<'a>>>, NP_Error> {
        
        match &*schema.kind {
            NP_SchemaKinds::Tuple { values, sorted } => {

                let mut addr = kind.get_value();

                let mut values_vec: Vec<u32> = Vec::new();

                if addr == 0 {
                    let mut addresses = Vec::with_capacity(4 * values.len());

                    for x in 0..addresses.len() {
                        addresses[x] = 0;
                    }

                    // no tuple here, make one
                    addr = buffer.malloc(addresses)?; // stores value addresses
                    buffer.set_value_address(address, addr, &kind);
                    for _x in 0..values.len() {
                        values_vec.push(0);
                    }

                    if *sorted { // write default values in sorted order
                        for x in 0..values_vec.len() as u32 {
                            let ptr = NP_Ptr::<NP_Any>::new_standard_ptr(addr + (x * 4), schema, buffer);
                            ptr.set_default()?;
                            values_vec[x as usize] = ptr.location;
                            buffer.set_value_address(addr + (x * 4), ptr.location, &kind);
                        }
                    }

                } else {
                    // existing head, read value
                    let a = addr as usize;
                    for x in 0..values.len() {
                        let value_address_bytes = *buffer.get_4_bytes(a + (x * 4)).unwrap_or(&[0; 4]);
                        values_vec.push(u32::from_be_bytes(value_address_bytes));
                    }
                }

                Ok(Some(Box::new(NP_Tuple::new(addr, buffer, values, values_vec))))
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> Result<u32, NP_Error> {
        
        let base_size = 0u32;

        let addr = kind.get_value();

        if addr == 0 {
            return Ok(0);
        }

        match &*schema.kind {
            NP_SchemaKinds::Tuple { values: _, sorted: _ } => {

                let tuple = NP_Tuple::buffer_into(addr, *kind, schema, buffer)?.unwrap();

                let mut acc_size = 0u32;

                for mut l in tuple.it().into_iter() {
                    if l.has_value == true {
                        let ptr = l.select::<NP_Any>()?;
                        acc_size += ptr.calc_size()?;
                    } else {
                        acc_size += 4; // empty pointer
                    }
                }

                return Ok(base_size + acc_size);
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_to_json(_address: u32, kind: &NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> JFObject {

        match &*schema.kind {
            NP_SchemaKinds::Tuple { values: _, sorted: _ } => {

                let addr = kind.get_value();

                if addr == 0 {
                    return JFObject::Null;
                }

                let tuple = NP_Tuple::buffer_into(addr, *kind, schema, buffer).unwrap().unwrap();

                let mut json_list = Vec::new();

                for mut l in tuple.it().into_iter() {
                    if l.has_value == true {
                        let ptr = l.select::<NP_Any>();
                        match ptr {
                            Ok(p) => {
                                json_list.push(p.json_encode());
                            },
                            Err (_e) => {
                                json_list.push(JFObject::Null);
                            }
                        }
                    } else {
                        json_list.push(JFObject::Null);
                    }
                }

                JFObject::Array(json_list)
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_do_compact<X: NP_Value + Default + NP_ValueInto<'a>>(from_ptr: &NP_Ptr<'a, X>, to_ptr: NP_Ptr<'a, NP_Any>) -> Result<(u32, NP_PtrKinds, &'a NP_Schema), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok((0, from_ptr.kind, from_ptr.schema));
        }

        let to_ptr_list = NP_Any::cast::<NP_Tuple>(to_ptr)?;

        let new_address = to_ptr_list.location;

        match Self::buffer_into(from_ptr.location, from_ptr.kind, from_ptr.schema, from_ptr.memory)? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(new_tuple) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value {
                                let new_ptr = new_tuple.select(item.index as u8)?;
                                let old_ptr = item.select::<NP_Any>()?;
                                old_ptr.compact(new_ptr)?;
                            }

                        }

                        return Ok((new_address, from_ptr.kind, from_ptr.schema));
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok((0, from_ptr.kind, from_ptr.schema))
    }
}

impl<'a> Default for NP_Tuple<'a> {

    fn default() -> Self {
        NP_Tuple { address: 0, memory: None, schemas: None, values: None}
    }
}


pub struct NP_Tuple_Iterator<'a> {
    address: u32, // pointer location
    memory: &'a NP_Memory,
    current_index: u16,
    schemas: &'a Vec<NP_Schema>,
    values: Vec<u32>
}

impl<'a> NP_Tuple_Iterator<'a> {

    pub fn new(address: u32, memory: &'a NP_Memory, schemas: &'a Vec<NP_Schema>, values: Vec<u32>) -> Self {
        NP_Tuple_Iterator {
            address,
            memory,
            current_index: 0,
            schemas: schemas,
            values: values
        }
    }

    pub fn into_tuple(self) -> NP_Tuple<'a> {
        NP_Tuple::new(self.address, self.memory, self.schemas, self.values)
    }
}

impl<'a> Iterator for NP_Tuple_Iterator<'a> {
    type Item = NP_Tuple_Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {

        if (self.current_index as usize) > self.values.len() {
            return None;
        }

        let this_index = self.current_index;
        self.current_index += 1;
        
        Some(NP_Tuple_Item {
            index: this_index,
            has_value: self.values[this_index as usize] != 0,
            address: self.values[this_index as usize],
            memory: self.memory,
            schema: &self.schemas[this_index as usize]
        })
    }
}

pub struct NP_Tuple_Item<'a> { 
    pub index: u16,
    pub has_value: bool,
    pub address: u32,
    pub memory: &'a NP_Memory,
    pub schema: &'a NP_Schema,
}

impl<'a> NP_Tuple_Item<'a> {

    pub fn select<T: NP_Value + Default + NP_ValueInto<'a>>(&mut self) -> Result<NP_Ptr<'a, T>, NP_Error> {

        // match type casting
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && self.schema.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using ANY casting, check type
            if self.schema.type_data.0 != T::type_idx().0 {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str(T::type_idx().1.as_str());
                err.push_str(") to schema of type (");
                err.push_str(self.schema.type_data.1.as_str());
                err.push_str(")");
                return Err(NP_Error::new(err));
            }
        }

        Ok(NP_Ptr::new_standard_ptr(self.address, self.schema, self.memory))
    }
}
