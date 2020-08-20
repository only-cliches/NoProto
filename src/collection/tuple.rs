use crate::pointer::any::NP_Any;
use crate::pointer::NP_Ptr;
use crate::pointer::{NP_PtrKinds, NP_Value, NP_Lite_Ptr};
use crate::{memory::{NP_Size, NP_Memory}, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::{rc::Rc, boxed::Box};

/// Tuple data type [Using collections with pointers](../pointer/struct.NP_Ptr.html#using-collection-types-with-pointers).
#[derive(Debug)]
pub struct NP_Tuple {
    address: u32, // pointer location
    memory: Option<Rc<NP_Memory>>,
    schemas: Option<Rc<Vec<Rc<NP_Schema>>>>,
    values: Option<Vec<u32>>
}


impl NP_Tuple {

    #[doc(hidden)]
    pub fn new(address: u32, memory: Rc<NP_Memory>, schemas: Rc<Vec<Rc<NP_Schema>>>, values: Vec<u32>) -> Self {
        NP_Tuple {
            address,
            memory: Some(memory),
            schemas: Some(schemas),
            values: Some(values)
        }
    }

    /// Select a value at a given index in the tuple
    pub fn select<T: NP_Value + Default>(&self, index: u8) -> Result<NP_Ptr<T>, NP_Error> {

        let values = self.values.as_ref().unwrap();

        if index as usize > values.len() {
            return Err(NP_Error::new("Attempted to access tuple value outside length!"));
        }

        let schema_vec = self.schemas.as_ref().unwrap();
        let schema = Rc::clone(&schema_vec[index as usize]);

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

        let rc_memory = match &self.memory {
            Some(x) => {
                Rc::clone(x)
            },
            None => {
                unreachable!();
            }
        };

        Ok(NP_Ptr::_new_standard_ptr(values[index as usize], schema, rc_memory))
    }

    /// Convert the tuple into an iterator
    pub fn it(self) -> NP_Tuple_Iterator {
        NP_Tuple_Iterator::new(self.address, self.memory.unwrap(), self.schemas.unwrap(), self.values.unwrap())
    }

    /// Get the length of the tuple, includes empty items
    pub fn len(&self) -> u8 {
        match &self.schemas {
            Some(x) => {
                x.len() as u8
            },
            None => {
                0
            }
        }
    }

    /// Remove all values from the tuple
    pub fn clear(self) -> Self {

        let addr = self.address as u32;

        let length = self.values.unwrap().len();

        // let write_bytes = Rc::clone(&self.memory.unwrap()).write_bytes();
        let write_bytes = match &self.memory {
            Some(x) => {
                x.write_bytes()
            },
            None => unreachable!()
        };

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

impl NP_Value for NP_Tuple {
    fn is_type(_type_str: &str) -> bool {  // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::Tuple as i64, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Tuple as i64, "tuple".to_owned()) }
    fn set_value(_pointer: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()! Use .into() instead."))
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {

        match &*ptr.schema.kind {
            NP_SchemaKinds::Tuple { values, sorted } => {

                let mut addr = ptr.kind.get_value_addr();

                let mut values_vec: Vec<u32> = Vec::new();

                if addr == 0 {
                    let mut addresses = match &ptr.memory.size {
                        NP_Size::U16 => Vec::with_capacity(2 * values.len()),
                        NP_Size::U32 => Vec::with_capacity(4 * values.len())
                    };



                    for x in 0..addresses.len() {
                        addresses[x] = 0;
                    }

                    // no tuple here, make one
                    addr = ptr.memory.malloc(addresses)?; // stores value addresses
                    ptr.memory.set_value_address(ptr.location, addr, &ptr.kind);
                    for _x in 0..values.len() {
                        values_vec.push(0);
                    }

                    if *sorted { // write default values in sorted order
                        for x in 0..values_vec.len() as u32 {
                            let ptr = match &ptr.memory.size {
                                NP_Size::U16 => NP_Ptr::<NP_Any>::_new_standard_ptr(addr + (x * 2), Rc::clone(&ptr.schema), Rc::clone(&ptr.memory)),
                                NP_Size::U32 => NP_Ptr::<NP_Any>::_new_standard_ptr(addr + (x * 4), Rc::clone(&ptr.schema), Rc::clone(&ptr.memory))
                            };
                            ptr.set_default()?;
                            values_vec[x as usize] = ptr.location;
                            
                            match &ptr.memory.size {
                                NP_Size::U16 => {
                                    ptr.memory.set_value_address(addr + (x * 2), ptr.location, &ptr.kind);
                                },
                                NP_Size::U32 => {
                                    ptr.memory.set_value_address(addr + (x * 4), ptr.location, &ptr.kind);
                                }
                            };
                        }
                    }

                } else {
                    // existing head, read value
                    let a = addr as usize;
                    match &ptr.memory.size {
                        NP_Size::U16 => {
                            for x in 0..values.len() {
                                let value_address_bytes = *ptr.memory.get_2_bytes(a + (x * 2)).unwrap_or(&[0; 2]);
                                values_vec.push(u16::from_be_bytes(value_address_bytes) as u32);
                            }    
                        },
                        NP_Size::U32 => {
                            for x in 0..values.len() {
                                let value_address_bytes = *ptr.memory.get_4_bytes(a + (x * 4)).unwrap_or(&[0; 4]);
                                values_vec.push(u32::from_be_bytes(value_address_bytes));
                            }                            
                        }
                    };

                }

                Ok(Some(Box::new(NP_Tuple::new(addr, ptr.memory, Rc::clone(&values), values_vec))))
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        
        let base_size = 0u32;

        let addr = ptr.kind.get_value_addr();

        if addr == 0 {
            return Ok(0);
        }

        match &*ptr.schema.kind {
            NP_SchemaKinds::Tuple { values: _, sorted: _ } => {

                let tuple = NP_Tuple::into_value(ptr.clone())?.unwrap();

                let mut acc_size = 0u32;

                for mut l in tuple.it().into_iter() {
                    if l.has_value == true {
                        let ptr = l.select::<NP_Any>()?;
                        acc_size += ptr.calc_size()?;
                    } else {
                        // empty pointer
                        acc_size += match &ptr.memory.size {
                            NP_Size::U16 => 2,
                            NP_Size::U32 => 4
                        };
                    }
                }

                return Ok(base_size + acc_size);
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {

        match &*ptr.schema.kind {
            NP_SchemaKinds::Tuple { values: _, sorted: _ } => {

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
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn do_compact(from_ptr: NP_Lite_Ptr, to_ptr: NP_Lite_Ptr) -> Result<(), NP_Error> where Self: NP_Value + Default {

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
}

impl Default for NP_Tuple {

    fn default() -> Self {
        NP_Tuple { address: 0, memory: None, schemas: None, values: None}
    }
}

/// Tuple iterator data type
#[derive(Debug)]
pub struct NP_Tuple_Iterator {
    address: u32, // pointer location
    memory: Rc<NP_Memory>,
    current_index: u16,
    schemas: Rc<Vec<Rc<NP_Schema>>>,
    values: Vec<u32>
}

impl NP_Tuple_Iterator {

    #[doc(hidden)]
    pub fn new(address: u32, memory: Rc<NP_Memory>, schemas: Rc<Vec<Rc<NP_Schema>>>, values: Vec<u32>) -> Self {
        NP_Tuple_Iterator {
            address,
            memory,
            current_index: 0,
            schemas: schemas,
            values: values
        }
    }

    /// Convert the iterator back into a tuple
    pub fn into_tuple(self) -> NP_Tuple {
        NP_Tuple::new(self.address, self.memory, self.schemas, self.values)
    }
}

impl Iterator for NP_Tuple_Iterator {
    type Item = NP_Tuple_Item;

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
            memory: Rc::clone(&self.memory),
            schema: Rc::clone(&self.schemas[this_index as usize])
        })
    }
}

/// A single iterator item
#[derive(Debug)]
pub struct NP_Tuple_Item { 
    /// The index of this item in the list
    pub index: u16,
    /// If there is a value at this index
    pub has_value: bool,
    address: u32,
    memory: Rc<NP_Memory>,
    schema: Rc<NP_Schema>,
}

impl NP_Tuple_Item {

    /// Get the pointer at this iterator location
    pub fn select<T: NP_Value + Default>(&mut self) -> Result<NP_Ptr<T>, NP_Error> {

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

        Ok(NP_Ptr::_new_standard_ptr(self.address, Rc::clone(&self.schema), Rc::clone(&self.memory)))
    }
}
