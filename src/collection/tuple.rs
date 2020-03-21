use crate::pointer::NP_Ptr;
use crate::pointer::NP_ValueInto;
use crate::pointer::{NP_PtrKinds, NP_Value};
use crate::{memory::NP_Memory, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, error::NP_Error};

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
            return Err(NP_Error::new("Attempted to access tuple value outside index!"));
        }

        let schema_vec = *self.schemas.as_ref().unwrap();
        let schema: &NP_Schema = &schema_vec[index as usize];

        // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
        if T::type_idx().0 != NP_TypeKeys::Any as i64 && schema.type_data.0 != NP_TypeKeys::Any as i64  {

            // not using any casting, check type
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

    pub fn delete(&mut self, index: u8) -> bool {
        match &mut self.values {
            Some(x) => {

                if index as usize > x.len() {
                    return false;
                }

                if x[index as usize] == 0 {
                    return false;
                }

                x[index as usize] = 0;

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
    fn new<T: NP_Value + Default>() -> Self {
        unreachable!()
    }
    fn is_type(_type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "tuple".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "tuple".to_owned()) }
    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .get()! Use .into() instead."))
    }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (tuple) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a> NP_ValueInto<'a> for NP_Tuple<'a> {
    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<NP_Tuple<'a>>>, NP_Error> {
        
        match &*schema.kind {
            NP_SchemaKinds::Tuple { values } => {

                let mut addr = kind.get_value();

                let mut addresses = Vec::with_capacity(4 * values.len());

                for x in 0..addresses.len() {
                    addresses[x] = 0;
                }

                let mut values_vec: Vec<u32> = Vec::new();

                if addr == 0 {
                    // no tuple here, make one
                    addr = buffer.malloc(addresses)?; // stores value addresses
                    buffer.set_value_address(address, addr, &kind);
                    for _x in 0..values.len() {
                        values_vec.push(0);
                    }

                } else {
                    // existing head, read value
                    let a = addr as usize;
                    for x in 0..values.len() {
                        let value_address_bytes = *buffer.get_4_bytes(a + (x * 4)).unwrap_or(&[0; 4]);
                        values_vec.push(u32::from_le_bytes(value_address_bytes));
                    }
                }

                Ok(Some(Box::new(NP_Tuple::new(addr, buffer, values, values_vec))))
            },
            _ => {
                Err(NP_Error::new("unreachable"))
            }
        }
    }
}

impl<'a> Default for NP_Tuple<'a> {

    fn default() -> Self {
        NP_Tuple { address: 0, memory: None, schemas: None, values: None}
    }
}