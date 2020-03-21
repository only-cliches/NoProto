use crate::pointer::NP_PtrKinds;
use crate::pointer::{NP_ValueInto, NP_Value, NP_Ptr};
use crate::{memory::NP_Memory, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, error::NP_Error};

use alloc::string::FromUtf8Error;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::string::ToString;

pub struct NP_Map<'a, T> {
    address: u32, // pointer location
    head: u32,
    len: u16,
    memory: Option<&'a NP_Memory>,
    value: Option<&'a NP_Schema>,
    _val: T
}



impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_Value for NP_Map<'a, T> {
    fn new<X>() -> Self {
        unreachable!()
    }
    fn is_type( _type_str: &str) -> bool { 
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (-1, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "map".to_owned()) }
    fn buffer_get(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Type (map) doesn't support .get()! Use .into() instead."))
    }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: &NP_Schema, _buffer: &NP_Memory, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (map) doesn't support .set()! Use .into() instead."))
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_ValueInto<'a> for NP_Map<'a, T> {
    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<NP_Map<'a, T>>>, NP_Error> {
        
        match &*schema.kind {
            NP_SchemaKinds::Map { value } => {

                // make sure the type we're casting to isn't ANY or the cast itself isn't ANY
                if T::type_idx().0 != NP_TypeKeys::Any as i64 && value.type_data.0 != NP_TypeKeys::Any as i64  {

                    // not using ANY casting, check type
                    if value.type_data.0 != T::type_idx().0 {
                        let mut err = "TypeError: Attempted to cast type (".to_owned();
                        err.push_str(T::type_idx().1.as_str());
                        err.push_str(") to schema of type (");
                        err.push_str(value.type_data.1.as_str());
                        err.push_str(")");
                        return Err(NP_Error::new(err));
                    }
                }

                let mut addr = kind.get_value();

                let mut head: [u8; 4] = [0; 4];
                let mut size: [u8; 2] = [0; 2];

                if addr == 0 {
                    // no map here, make one
                    addr = buffer.malloc([0 as u8; 6].to_vec())?; // stores HEAD & LENGTH for map
                    buffer.set_value_address(address, addr, &kind);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                    size = *buffer.get_2_bytes(a + 4).unwrap_or(&[0; 2]);
                }

                Ok(Some(Box::new(NP_Map::new(addr, u32::from_le_bytes(head), u16::from_le_bytes(size), buffer, value))))
            },
            _ => {
                Err(NP_Error::new("unreachable"))
            }
        }
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> Default for NP_Map<'a, T> {

    fn default() -> Self {
        NP_Map { address: 0, head: 0, memory: None, value: None, _val: T::default(), len: 0 }
    }
}

impl<'a, T: NP_Value + Default + NP_ValueInto<'a>> NP_Map<'a, T> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, length: u16, memory: &'a NP_Memory, value: &'a NP_Schema) -> Self {
        NP_Map {
            address,
            head,
            memory: Some(memory),
            value: Some(value),
            _val: T::default(),
            len: length
        }
    }

    pub fn select(&mut self, key: &Vec<u8>) -> core::result::Result<NP_Ptr<'a, T>, NP_Error> {

        let memory = self.memory.unwrap();

        if self.head == 0 { // no values, create one

            let mut ptr_bytes: [u8; 12] = [0; 12]; // map item pointer

            // key length, then key data
            let key_addr = memory.malloc((key.len() as u32).to_le_bytes().to_vec())?;
            memory.malloc(key.clone())?;

            let key_addr_bytes = key_addr.to_le_bytes();

            for x in 0..key_addr_bytes.len() {
                ptr_bytes[8 + x] = key_addr_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;
            
            // update head to point to newly created Map Item pointer
            self.set_head(addr);
            self.set_len(1);

            // provide
            return Ok(NP_Ptr::new_map_item_ptr(self.head, self.value.unwrap(), &memory));
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut next_addr = self.head as usize;

            let mut has_next = true;

            while has_next {

                let key_bytes_addr = *memory.get_4_bytes(next_addr + 8).unwrap_or(&[0; 4]);

                let key_addr = u32::from_le_bytes(key_bytes_addr) as usize;

                let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

                let bytes_size = u32::from_le_bytes(key_bytes_length) as usize;

                let key_bytes: &[u8] = &memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

                let key_vec = key_bytes.to_vec();

                // found our value!
                if key_vec == *key {
                    return Ok(NP_Ptr::new_map_item_ptr(next_addr as u32, self.value.unwrap(), &memory));
                }
                
                // not found yet, get next address
                let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);

                let next_ptr = u32::from_le_bytes(next) as usize;
                if next_ptr == 0 {
                    has_next = false;
                } else {
                    next_addr = next_ptr;
                }
            }

            // ran out of pointers to check, make one!
            let mut ptr_bytes: [u8; 12] = [0; 12];

            // key length, then key data
            let key_addr = memory.malloc((key.len() as u32).to_le_bytes().to_vec())?;
            memory.malloc(key.clone())?;

            let key_addr_bytes = key_addr.to_le_bytes();

            for x in 0..key_addr_bytes.len() {
                ptr_bytes[8 + x] = key_addr_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // set previouse pointer's "next" value to this new pointer
            let addr_bytes = addr.to_le_bytes();
            let write_bytes = memory.write_bytes();
            for x in 0..addr_bytes.len() {
                write_bytes[(next_addr + 4 + x)] = addr_bytes[x];
            }

            self.set_len(self.len + 1);
            
            // provide 
            return Ok(NP_Ptr::new_map_item_ptr(addr, self.value.unwrap(), &memory));

        }
    }

    pub fn delete(&mut self, key: &Vec<u8>) -> core::result::Result<bool, NP_Error>{

        let memory = self.memory.unwrap();

        if self.head == 0 { // no values, nothing to delete
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut has_next = true;

            while has_next {

                let key_bytes_addr = *memory.get_4_bytes(curr_addr + 8).unwrap_or(&[0; 4]);

                let key_addr = u32::from_le_bytes(key_bytes_addr) as usize;

                let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

                let bytes_size = u32::from_le_bytes(key_bytes_length) as usize;

                let key_bytes: &[u8] = &memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

                let key_vec = key_bytes.to_vec();

                // found our value!
                if key_vec == *key {

                    let next_pointer_bytes: [u8; 4];

                    match memory.get_4_bytes(curr_addr + 4) {
                        Some(x) => {
                            next_pointer_bytes = *x;
                        },
                        None => {
                            return Err(NP_Error::new("Out of range request"));
                        }
                    }

                    if curr_addr == self.head as usize { // item is HEAD
                        self.set_head(u32::from_le_bytes(next_pointer_bytes));
                    } else { // item is NOT head
                
                        let memory_bytes = memory.write_bytes();
                
                        for x in 0..next_pointer_bytes.len() {
                            memory_bytes[(prev_addr + x as u32 + 4) as usize] = next_pointer_bytes[x as usize];
                        }
                    }

                    // set length
                    self.set_len(self.len - 1);

                    return Ok(true);
                }
                
                // not found yet, get next address
                let next: [u8; 4] = *memory.get_4_bytes(curr_addr + 4).unwrap_or(&[0; 4]);

                let next_ptr = u32::from_le_bytes(next) as usize;
                if next_ptr == 0 {
                    has_next = false;
                } else {
                    // store old value for next loop
                    prev_addr = curr_addr as u32;

                    // set next pointer for next loop
                    curr_addr = next_ptr;
                }
            }

            // out of pointers to check, nothing to delete
            Ok(false)
        }
    }

    pub fn len(&self) -> u16 {
        self.len
    }

    fn set_len(&mut self, len: u16) {
        self.len = len;

        let memory_bytes = self.memory.unwrap().write_bytes();
       
        let len_bytes = len.to_le_bytes();

        for x in 0..len_bytes.len() {
            memory_bytes[(self.address + (x as u32) + 4) as usize] = len_bytes[x as usize];
        }
    }

    pub fn empty(self) -> Self {

        let memory_bytes = self.memory.unwrap().write_bytes();

        for x in 0..6 {
            memory_bytes[(self.address + x as u32) as usize] = 0;
        }

        NP_Map {
            address: self.address,
            head: 0,
            memory: self.memory,
            value: self.value,
            _val: T::default(),
            len: 0
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let memory_bytes = self.memory.unwrap().write_bytes();
       
        let head_bytes = addr.to_le_bytes();

        for x in 0..head_bytes.len() {
            memory_bytes[(self.address + x as u32) as usize] = head_bytes[x as usize];
        }
      
    }

    pub fn has(&self, key: &Vec<u8>) -> core::result::Result<bool, NP_Error> {

        if self.head == 0 { // no values in this table
           return Ok(false);
        }

        let mut next_addr = self.head as usize;

        let mut has_next = true;

        let memory = self.memory.unwrap();

        while has_next {

            let key_bytes_addr = *memory.get_4_bytes(next_addr + 8).unwrap_or(&[0; 4]);

            let key_addr = u32::from_le_bytes(key_bytes_addr) as usize;

            let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

            let bytes_size = u32::from_le_bytes(key_bytes_length) as usize;

            let key_bytes: &[u8] = &memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

            let key_vec = key_bytes.to_vec();

            // found our value!
            if key_vec == *key {
                return Ok(true);
            }
            
            // not found yet, get next address
            let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);

            let next_ptr = u32::from_le_bytes(next) as usize;
            if next_ptr == 0 {
                has_next = false;
            } else {
                next_addr = next_ptr;
            }
        }

        // ran out of pointers, value doesn't exist!
        return Ok(false);
    }

}