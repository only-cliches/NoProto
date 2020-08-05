use alloc::rc::Rc;
use crate::pointer::NP_PtrKinds;
use crate::pointer::{NP_Value, NP_Ptr, any::NP_Any};
use crate::{memory::NP_Memory, schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys}, error::NP_Error, json_flex::NP_JSON};

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;

pub struct NP_Map<T> {
    address: u32, // pointer location
    head: u32,
    len: u16,
    memory: Option<Rc<NP_Memory>>,
    value: Option<Rc<NP_Schema>>,
    _val: T
}

impl<T: NP_Value + Default> NP_Value for NP_Map<T> {
    fn is_type( _type_str: &str) -> bool {  // not needed for collection types
        unreachable!()
    }
    fn type_idx() -> (i64, String) { (NP_TypeKeys::Map as i64, "map".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Map as i64, "map".to_owned()) }
    fn buffer_set(_address: u32, _kind: &NP_PtrKinds, _schema: Rc<NP_Schema>, _buffer: Rc<NP_Memory>, _value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Type (map) doesn't support .set()! Use .into() instead."))
    }

    fn buffer_into(address: u32, kind: NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> core::result::Result<Option<Box<NP_Map<T>>>, NP_Error> {
        
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

                Ok(Some(Box::new(NP_Map::new(addr, u32::from_be_bytes(head), u16::from_be_bytes(size), buffer, Rc::clone(value)))))
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> Result<u32, NP_Error> {
        let base_size = 6u32; // head + length

        match &*schema.kind {
            NP_SchemaKinds::Map { value } => {

                let addr = kind.get_value();

                let head: [u8; 4];
                let size: [u8; 2];

                if addr == 0 {
                    return Ok(0);
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                    size = *buffer.get_2_bytes(a + 4).unwrap_or(&[0; 2]);
                }

                let list = NP_Map::<T>::new(addr, u32::from_be_bytes(head), u16::from_be_bytes(size), buffer, Rc::clone(value));

                let mut acc_size = 0u32;

                for mut l in list.it().into_iter() {

                    if l.has_value == true {
                        let ptr = l.select()?;
                        acc_size += ptr.calc_size()?;
                        acc_size += l.key.len() as u32 + 4u32; // key + key length bytes
                    }

                };

                Ok(acc_size + base_size)
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_to_json(_address: u32, kind: &NP_PtrKinds, schema: Rc<NP_Schema>, buffer: Rc<NP_Memory>) -> NP_JSON {

        match &*schema.kind {
            NP_SchemaKinds::Map { value } => {

                let addr = kind.get_value();

                let head: [u8; 4];
                let size: [u8; 2];

                if addr == 0 {
                    return NP_JSON::Null;
                } else {
                    // existing head, read value
                    let a = addr as usize;
                    head = *buffer.get_4_bytes(a).unwrap_or(&[0; 4]);
                    size = *buffer.get_2_bytes(a + 4).unwrap_or(&[0; 2]);
                }

                let list = NP_Map::<T>::new(addr, u32::from_be_bytes(head), u16::from_be_bytes(size), buffer, Rc::clone(value));

                let mut json_list = Vec::new();

                for mut l in list.it().into_iter() {

                    let value: NP_JSON;

                    if l.has_value == true {
                        let ptr = l.select();
                        match ptr {
                            Ok(p) => {
                                value = p.json_encode();
                            },
                            Err (_e) => {
                                value = NP_JSON::Null;
                            }
                        }
                    } else {
                        value = NP_JSON::Null;
                    }

                    let mut kv = Vec::new();
                    kv.push(NP_JSON::Array(l.key.into_iter().map(|k| NP_JSON::Integer(k as i64)).collect()));
                    kv.push(value);

                    json_list.push(NP_JSON::Array(kv));
                }

                NP_JSON::Array(json_list)
            },
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_do_compact<X: NP_Value + Default>(from_ptr: &NP_Ptr<X>, to_ptr: NP_Ptr<NP_Any>) -> Result<(u32, NP_PtrKinds, Rc<NP_Schema>), NP_Error> where Self: NP_Value + Default {

        if from_ptr.location == 0 {
            return Ok((0, from_ptr.kind, Rc::clone(&from_ptr.schema)));
        }

        let to_ptr_list = NP_Any::cast::<NP_Map<NP_Any>>(to_ptr)?;

        let new_address = to_ptr_list.location;

        match Self::buffer_into(from_ptr.location, from_ptr.kind, Rc::clone(&from_ptr.schema), Rc::clone(&from_ptr.memory))? {
            Some(old_list) => {

                match to_ptr_list.into()? {
                    Some(mut new_list) => {

                        for mut item in old_list.it().into_iter() {

                            if item.has_value {
                                let new_ptr = new_list.select(&item.key)?;
                                let old_ptr = item.select()?;
                                old_ptr._compact(new_ptr)?;
                            }

                        }

                        return Ok((new_address, from_ptr.kind, Rc::clone(&from_ptr.schema)));
                    },
                    None => {}
                }
            },
            None => { }
        }

        Ok((0, from_ptr.kind, Rc::clone(&from_ptr.schema)))
    }
}

impl<'a, T: NP_Value + Default> Default for NP_Map<T> {

    fn default() -> Self {
        NP_Map { address: 0, head: 0, memory: None, value: None, _val: T::default(), len: 0 }
    }
}

impl<'a, T: NP_Value + Default> NP_Map<T> {

    #[doc(hidden)]
    pub fn new(address: u32, head: u32, length: u16, memory: Rc<NP_Memory>, value: Rc<NP_Schema>) -> Self {
        NP_Map {
            address,
            head,
            memory: Some(memory),
            value: Some(value),
            _val: T::default(),
            len: length
        }
    }

    pub fn it(self) -> NP_Map_Iterator<T> {
        NP_Map_Iterator::new(self.address, self.head, self.len, self.memory.unwrap(), self.value.unwrap())
    }

    pub fn select(&mut self, key: &Vec<u8>) -> core::result::Result<NP_Ptr<T>, NP_Error> {

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        let self_schema = match &self.value {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        if self.head == 0 { // no values, create one

            let mut ptr_bytes: [u8; 12] = [0; 12]; // map item pointer

            // key length, then key data
            let key_addr = memory.malloc((key.len() as u32).to_be_bytes().to_vec())?;
            memory.malloc(key.clone())?;

            let key_addr_bytes = key_addr.to_be_bytes();

            for x in 0..key_addr_bytes.len() {
                ptr_bytes[8 + x] = key_addr_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;
            
            // update head to point to newly created Map Item pointer
            self.set_head(addr);
            self.set_len(1);

            // provide
            return Ok(NP_Ptr::new_map_item_ptr(self.head, self_schema, memory));
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut next_addr = self.head as usize;

            let mut has_next = true;

            while has_next {

                let key_bytes_addr = *memory.get_4_bytes(next_addr + 8).unwrap_or(&[0; 4]);

                let key_addr = u32::from_be_bytes(key_bytes_addr) as usize;

                let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

                let bytes_size = u32::from_be_bytes(key_bytes_length) as usize;

                let key_bytes: &[u8] = &memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

                let key_vec = key_bytes.to_vec();

                // found our value!
                if key_vec == *key {
                    return Ok(NP_Ptr::new_map_item_ptr(next_addr as u32, self_schema, memory));
                }
                
                // not found yet, get next address
                let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);

                let next_ptr = u32::from_be_bytes(next) as usize;
                if next_ptr == 0 {
                    has_next = false;
                } else {
                    next_addr = next_ptr;
                }
            }

            // ran out of pointers to check, make one!
            let mut ptr_bytes: [u8; 12] = [0; 12];

            // key length, then key data
            let key_addr = memory.malloc((key.len() as u32).to_be_bytes().to_vec())?;
            memory.malloc(key.clone())?;

            let key_addr_bytes = key_addr.to_be_bytes();

            for x in 0..key_addr_bytes.len() {
                ptr_bytes[8 + x] = key_addr_bytes[x];
            }

            let addr = memory.malloc(ptr_bytes.to_vec())?;

            // set previouse pointer's "next" value to this new pointer
            let addr_bytes = addr.to_be_bytes();
            let write_bytes = memory.write_bytes();
            for x in 0..addr_bytes.len() {
                write_bytes[(next_addr + 4 + x)] = addr_bytes[x];
            }

            self.set_len(self.len + 1);
            
            // provide 
            return Ok(NP_Ptr::new_map_item_ptr(addr, self_schema, memory));

        }
    }

    pub fn delete(&mut self, key: &Vec<u8>) -> core::result::Result<bool, NP_Error>{

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };


        if self.head == 0 { // no values, nothing to delete
            Ok(false)
        } else { // values exist, loop through them to see if we have an existing pointer for this column

            let mut curr_addr = self.head as usize;
            let mut prev_addr = 0u32;

            let mut has_next = true;

            while has_next {

                let key_bytes_addr = *memory.get_4_bytes(curr_addr + 8).unwrap_or(&[0; 4]);

                let key_addr = u32::from_be_bytes(key_bytes_addr) as usize;

                let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

                let bytes_size = u32::from_be_bytes(key_bytes_length) as usize;

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
                        self.set_head(u32::from_be_bytes(next_pointer_bytes));
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

                let next_ptr = u32::from_be_bytes(next) as usize;
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
        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };


        let memory_bytes = memory.write_bytes();
       
        let len_bytes = len.to_be_bytes();

        for x in 0..len_bytes.len() {
            memory_bytes[(self.address + (x as u32) + 4) as usize] = len_bytes[x as usize];
        }
    }

    pub fn empty(self) -> Self {

        let memory = match self.memory {
            Some(x) => x,
            None => unreachable!()
        };


        let memory_bytes = memory.write_bytes();

        for x in 0..6 {
            memory_bytes[(self.address + x as u32) as usize] = 0;
        }

        NP_Map {
            address: self.address,
            head: 0,
            memory: Some(memory),
            value: self.value,
            _val: T::default(),
            len: 0
        }
    }

    fn set_head(&mut self, addr: u32) {

        self.head = addr;

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };


        let memory_bytes = memory.write_bytes();
       
        let head_bytes = addr.to_be_bytes();

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

        let memory = match &self.memory {
            Some(x) => Rc::clone(x),
            None => unreachable!()
        };

        while has_next {

            let key_bytes_addr = *memory.get_4_bytes(next_addr + 8).unwrap_or(&[0; 4]);

            let key_addr = u32::from_be_bytes(key_bytes_addr) as usize;

            let key_bytes_length = *memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

            let bytes_size = u32::from_be_bytes(key_bytes_length) as usize;

            let key_bytes: &[u8] = &memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

            let key_vec = key_bytes.to_vec();

            // found our value!
            if key_vec == *key {
                return Ok(true);
            }
            
            // not found yet, get next address
            let next: [u8; 4] = *memory.get_4_bytes(next_addr + 4).unwrap_or(&[0; 4]);

            let next_ptr = u32::from_be_bytes(next) as usize;
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


pub struct NP_Map_Iterator<T> {
    current_index: u16,
    address: u32, // pointer location
    head: u32,
    memory: Rc<NP_Memory>,
    length: u16,
    current_address: u32,
    value: Rc<NP_Schema>,
    map: NP_Map<T>
}

impl<T: NP_Value + Default> NP_Map_Iterator<T> {

    pub fn new(address: u32, head: u32, length: u16, memory: Rc<NP_Memory>, value: Rc<NP_Schema>) -> Self {
        NP_Map_Iterator {
            current_index: 0,
            address,
            head,
            memory: Rc::clone(&memory),
            current_address: head,
            value: Rc::clone(&value),
            length,
            map: NP_Map::new(address, head, length, memory, value)
        }
    }

    pub fn into_map(self) -> NP_Map<T> {
        self.map
    }
}

impl<T: NP_Value + Default> Iterator for NP_Map_Iterator<T> {
    type Item = NP_Map_Item<T>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.current_address == 0 {
            return None;
        }
        
        let value_address = u32::from_be_bytes(*self.memory.get_4_bytes(self.current_address as usize).unwrap_or(&[0; 4]));

        let key_bytes_addr = *self.memory.get_4_bytes((self.current_address + 8) as usize).unwrap_or(&[0; 4]);

        let key_addr = u32::from_be_bytes(key_bytes_addr) as usize;

        let key_bytes_length = *self.memory.get_4_bytes(key_addr).unwrap_or(&[0; 4]);

        let bytes_size = u32::from_be_bytes(key_bytes_length) as usize;

        let key_bytes: &[u8] = &self.memory.read_bytes()[(key_addr+4)..(key_addr+4+bytes_size)];

        let key_vec = key_bytes.to_vec();

        let next_bytes: [u8; 4] = *self.memory.get_4_bytes((self.current_address + 4) as usize).unwrap_or(&[0; 4]);

        let this_address = self.current_address;
        // point to next value
        self.current_address = u32::from_be_bytes(next_bytes);
        
        self.current_index += 1;
        return Some(NP_Map_Item {
            index: self.current_index - 1,
            has_value: value_address != 0,
            value: Rc::clone(&self.value),
            // length: self.length,
            key: key_vec,
            address: this_address,
            map: NP_Map::new(self.address, self.head, self.length, Rc::clone(&self.memory), Rc::clone(&self.value)),
            memory: Rc::clone(&self.memory)
        });
    }
}

pub struct NP_Map_Item<T> { 
    pub index: u16,
    pub key: Vec<u8>,
    pub has_value: bool,
    pub value: Rc<NP_Schema>,
    address: u32,
    // length: u16,
    map: NP_Map<T>,
    memory: Rc<NP_Memory>
}

impl<T: NP_Value + Default> NP_Map_Item<T> {
    
    pub fn select(&mut self) -> Result<NP_Ptr<T>, NP_Error> {
        Ok(NP_Ptr::new_map_item_ptr(self.address, Rc::clone(&self.value), Rc::clone(&self.memory)))
    }
    // TODO: Build a select statement that grabs the current index in place instead of seeking to it.
    pub fn delete(&mut self) -> Result<bool, NP_Error> {
        self.map.delete(&self.key)
    }
}

