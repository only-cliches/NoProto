extern crate rand;

// use crate::collection::map::NoProtoMap;
// use crate::collection::list::NoProtoList;
use crate::NoProtoSchema;
use crate::collection::table::NoProtoTable;
use crate::NoProtoSchemaKinds;
use crate::NoProtoMemory;
use std::cell::RefMut;
use core::cell::Ref;
use std::rc::Rc;
use std::cell::RefCell;
use std::result;
use json::*;
use std::{slice, ops::{ Index, IndexMut, Deref }};
use rand::Rng;
use std::fmt;
use std::time::{Duration, SystemTime};


fn to_hex(num: u64, length: i32) -> String {
    let mut result: String = "".to_owned();

    let hex_values = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];

    let mut i = length - 1;
    while i >= 0 {
        let raise = (16i32).pow(i as u32) as f64;
        let index = (num as f64 / raise).floor() as i32;
        result.push_str(hex_values[(index % 16i32) as usize]);
        i -= 1 ;
    }

    result
}

#[derive(Debug)]
pub struct NoProtoGeo {
    pub lat: f64,
    pub lon: f64
}

pub struct NoProtoTimeID {
    pub id: [u8; 8],
    pub time: u64
}

impl NoProtoTimeID {

    pub fn generate(id_bytes: Option<[u8; 8]>) -> NoProtoTimeID {
        let mut rng = rand::thread_rng();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        match id_bytes {
            Some(x) => {
                NoProtoTimeID {
                    time: now,
                    id: x
                }
            },
            None => {
                let mut id = [0; 8];

                for x in 0..id.len() {
                    id[x] = rng.gen_range(0, 255);
                }
        
                NoProtoTimeID {
                    time: now,
                    id: id
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut result: String = "".to_owned();

        // u64 can hold up to 20 digits or 584,942,417,355 years of seconds since unix epoch
        // 14 digits gets us 3,170,979 years of seconds after Unix epoch.  
        result.push_str(format!("{:0>14}", self.time).as_str()); // time first
        result.push_str("-"); // dash

        // id
        for x in 0..self.id.len() {
            let value = self.id[x] as u64;
            if x == 4 {
                result.push_str("-"); // dash
            }
            result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl fmt::Debug for NoProtoTimeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub struct NoProtoUUID {
    pub value: [u8; 16]
}

impl NoProtoUUID {

    pub fn generate() -> NoProtoUUID {

        let mut rng = rand::thread_rng();

        let mut uuid = NoProtoUUID {
            value: [0; 16]
        };

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + rng.gen_range(0, 15);
            } else {
                uuid.value[x] = rng.gen_range(0, 255);
            }
        }

        uuid
    }

    pub fn to_string(&self) -> String {

        let mut result: String = "".to_owned();

        for x in 0..self.value.len() {
            if x == 4 || x == 6 || x == 8 || x == 10 {
                result.push_str("-");
            }
            let value = self.value[x] as u64;
            result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl fmt::Debug for NoProtoUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

pub enum NoProtoPointerKinds {
    // scalar / collection
    Standard {value: u32}, // 4 bytes [4]

    // collection items
    MapItem {value: u32, key: u32, next: u32, prev: u32}, // 16 bytes [4, 4, 4, 4]
    TableItem {value: u32, i: u8, next: u32, prev: u32}, // 13 bytes [4, 1, 4, 4]
    ListItem {value: u32, i: u16, next: u32, prev: u32} // 14 bytes [4, 2, 4, 4]
}

pub struct NoProtoPointer<'a> {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    schema: &'a NoProtoSchema
}

impl<'a> NoProtoPointer<'a> {

    pub fn new_standard(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        {
            let b_bytes = &memory.borrow().bytes;
            value.copy_from_slice(&b_bytes[addr..(addr+4)]);
        }

        NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::Standard { value: u32::from_le_bytes(value) },
            memory: memory,
            schema: schema
        }
    }


    pub fn new_table_item(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        let mut next: [u8; 4] = [0; 4];
        let mut prev: [u8; 4] = [0; 4];
        let mut index: u8 = 0;

        {
            let b_bytes = &memory.borrow().bytes;
            value.copy_from_slice(&b_bytes[addr..(addr + 4)]);
            index = b_bytes[addr + 4];
            next.copy_from_slice(&b_bytes[(addr + 5)..(addr + 9)]);
            prev.copy_from_slice(&b_bytes[(addr + 9)..(addr + 13)]);
        }

        NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::TableItem { 
                value: u32::from_le_bytes(value),
                i: index,
                next: u32::from_le_bytes(next),
                prev: u32::from_le_bytes(prev)
            },
            memory: memory,
            schema: schema
        }
    }

    pub fn clear(&mut self) {
        self.set_value_address(0);
    }

    fn get_value_address(&self) -> u32 {
        match self.kind {
            NoProtoPointerKinds::Standard  { value } =>                               { value },
            NoProtoPointerKinds::MapItem   { value, key: _,  next: _, prev: _ } =>    { value },
            NoProtoPointerKinds::TableItem { value, i: _,    next: _, prev: _ } =>    { value },
            NoProtoPointerKinds::ListItem  { value, i:_ ,    next: _, prev: _ } =>    { value }
        }
    }

    fn set_value_address(&mut self, val: u32) {

        let mut memory = self.memory.borrow_mut();

        let addr_bytes = val.to_le_bytes();
    
        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }

        match self.kind {
            NoProtoPointerKinds::Standard { value: _ } => {
                self.kind = NoProtoPointerKinds::Standard { value: val}
            },
            NoProtoPointerKinds::MapItem { value: _, key,  next, prev } => {
                self.kind = NoProtoPointerKinds::MapItem { value: val, key: key, next: next, prev: prev}
            },
            NoProtoPointerKinds::TableItem { value: _, i, next, prev } => {
                self.kind = NoProtoPointerKinds::TableItem { value: val, i: i, next: next, prev: prev}
            },
            NoProtoPointerKinds::ListItem { value: _, i, next, prev } => {
                self.kind = NoProtoPointerKinds::ListItem { value: val, i: i, next: next, prev: prev}
            }
        }
    }

    pub fn as_table(&mut self) -> Option<NoProtoTable> {

        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Table { columns } => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                let mut head: [u8; 4] = [0; 4];

                // no table here, make one
                if addr == 0 {
                    let mut memory = self.memory.borrow_mut();

                    addr = memory.malloc([0 as u8; 4].to_vec()).unwrap_or(0); // stores HEAD for table
                    set_addr = true;

                    // out of memory
                    if addr == 0 {
                        return None;
                    }
                }

                if set_addr { // new head, empty value
                    self.set_value_address(addr);
                } else { // existing head, read value
                    let b_bytes = &self.memory.borrow().bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                }

                Some(NoProtoTable {
                    head: u32::from_le_bytes(head),
                    address: addr,
                    memory: Rc::clone(&self.memory),
                    columns: &columns
                })
            },
            _ => {
                None
            }
        }
    }

/*
    pub fn as_list(&self) -> Option<NoProtoList> {

    }

    pub fn as_map(&self) -> Option<NoProtoMap> {

    }

    pub fn as_tuple(&self) -> Option<NoProtoTuple> {

    }
 */
    pub fn to_string(&self) -> Option<String> {

        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Utf8String => {

                let mut addr = self.get_value_address() as usize;
                let mut set_addr = false;

                // empty value
                if addr == 0 {
                    return None;
                }
                
                // get size of string
                let mut size: [u8; 4] = [0; 4];
                let memory = self.memory.borrow();
                size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                let str_size = u32::from_le_bytes(size) as usize;

                // get string bytes
                let arrayBytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

                // convert to string
                let string = String::from_utf8(arrayBytes.to_vec());

                match string {
                    Ok(x) => {
                        // return string
                        Some(x)
                    },
                    Err(_e) => {
                        // Err("Error parsing string!")
                        None
                    }
                }
                
            },
            _ => {
                // NoProtoResult::Err("Not a string type in data model!")
                None
            }
        }
    }

    pub fn set_string(&mut self, value: &str) -> std::result::Result<bool, &'static str> {

        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Utf8String => {
                let bytes = value.as_bytes();
                let str_size = bytes.len() as u64;

                if str_size >= std::u32::MAX as u64 { 
                    Err("String too large!")
                } else {

                    let mut addr = self.get_value_address() as usize;
                    let mut set_addr = false;

                    {
                        let mut memory = self.memory.borrow_mut();

                        let prev_size: usize = if addr != 0 {
                            let mut size_bytes: [u8; 4] = [0; 4];
                            size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                            u32::from_le_bytes(size_bytes) as usize
                        } else {
                            0 as usize
                        };
    
                        if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
                    
                            let size_bytes = str_size.to_le_bytes();
                            // set string size
                            for x in 0..size_bytes.len() {
                                memory.bytes[(self.address + x as u32) as usize] = size_bytes[x as usize];
                            }
    
                            // set bytes
                            for x in 0..bytes.len() {
                                memory.bytes[(self.address + x as u32 + 4) as usize] = bytes[x as usize];
                            }
    
                        } else { // not enough space or space has not been allocted yet
                            

                            // first 4 bytes are string length
                            addr = memory.malloc((str_size as u32).to_le_bytes().to_vec()).unwrap_or(0) as usize;

                            set_addr = true;

                            // then string content
                            let addr2 = memory.malloc(bytes.to_vec()).unwrap_or(0);

                            if addr == 0 || addr2 == 0 {
                                return Err("Not enough memory!");
                            }
                        }
                    }

                    if set_addr { self.set_value_address(addr as u32) };
            
                    Ok(true)
                }

            }
            _ => {
                Err("Not a string type!")
            }
        }
    }

    pub fn to_bytes(&self) -> Option<Vec<u8>> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Bytes => {
                let value = self.get_value_address();

                // empty value
                if value == 0 {
                    return None;
                }
                
                // get size of bytes
                let addr = value as usize;
                let mut size: [u8; 4] = [0; 4];
                let memory = self.memory.borrow();
                size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                let bytes_size = u32::from_le_bytes(size) as usize;

                // get string bytes
                let bytes = &memory.bytes[(addr+4)..(addr+4+bytes_size)];

                Some(bytes.to_vec())
            },
            _ => {
                None
            }
        }
    }

    pub fn set_bytes(&mut self, bytes: Vec<u8>) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Bytes => {

                let size = bytes.len() as u64;

                if size >= std::u32::MAX as u64 { 
                    Err("Bytes too large!")
                } else {

                    let mut addr = self.get_value_address() as usize;
                    let mut set_addr = false;

                    {
                        let mut memory = self.memory.borrow_mut();

                        let prev_size: usize = if addr != 0 {
                            let mut size_bytes: [u8; 4] = [0; 4];
                            size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                            u32::from_le_bytes(size_bytes) as usize
                        } else {
                            0 as usize
                        };
    
                        if prev_size >= size as usize { // previous bytes is larger than this one, use existing memory
                    
                            let size_bytes = size.to_le_bytes();
                            // set string size
                            for x in 0..size_bytes.len() {
                                memory.bytes[(self.address + x as u32) as usize] = size_bytes[x as usize];
                            }
    
                            // set bytes
                            for x in 0..bytes.len() {
                                memory.bytes[(self.address + x as u32 + 4) as usize] = bytes[x as usize];
                            }
    
                        } else { // not enough space or space has not been allocted yet
                            

                            // first 4 bytes are bytes length
                            addr = memory.malloc((size as u32).to_le_bytes().to_vec()).unwrap_or(0) as usize;

                            set_addr = true;

                            // then string content
                            let addr2 = memory.malloc(bytes.to_vec()).unwrap_or(0);

                            if addr == 0 || addr2 == 0 {
                                return Err("Not enough memory!");
                            }
                        }
                    }

                    if set_addr { self.set_value_address(addr as u32) } ;
            
                    Ok(true)
                }
            },
            _ => {
                Err("Not a bytes type!")
            }
        }
    }

    pub fn get_1_byte(&self) -> Option<[u8; 1]> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return None;
        }

        let mut bytes: [u8; 1] = [0; 1];
        let memory = self.memory.borrow();
        bytes.copy_from_slice(&memory.bytes[value..(value + 1)]);

        Some(bytes)
    }

    pub fn get_2_bytes(&self) -> Option<[u8; 2]> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return None;
        }

        let mut bytes: [u8; 2] = [0; 2];
        let memory = self.memory.borrow();
        bytes.copy_from_slice(&memory.bytes[value..(value + 2)]);

        Some(bytes)
    }

    pub fn get_4_bytes(&self) -> Option<[u8; 4]> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return None;
        }

        let mut bytes: [u8; 4] = [0; 4];
        let memory = self.memory.borrow();
        bytes.copy_from_slice(&memory.bytes[value..(value + 4)]);

        Some(bytes)
    }

    pub fn get_8_bytes(&self) -> Option<[u8; 8]> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return None;
        }

        let mut bytes: [u8; 8] = [0; 8];
        let memory = self.memory.borrow();
        bytes.copy_from_slice(&memory.bytes[value..(value + 8)]);

        Some(bytes)
    }

    pub fn get_16_bytes(&self) -> Option<[u8; 16]> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return None;
        }

        let mut bytes: [u8; 16] = [0; 16];
        let memory = self.memory.borrow();
        bytes.copy_from_slice(&memory.bytes[value..(value + 16)]);

        Some(bytes)
    }
/*
    pub fn to_dec8(&self) -> Option<(i8, u8)> {
        let model = self.model;

        match *model.kind {
            NoProtoSchemaKinds::Utf8String => {
                match self.get_2_bytes() {
                    Some(x) => {
                        Some((i8::from_le_bytes([x[0]]), u8::from_le_bytes([x[1]])))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_dec8(&mut self, dec8: i8, raise: u8) -> std::result::Result<bool, &'static str> {
        match self.model.kind.as_str() {
            "dec8" => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = dec8.to_le_bytes();
    
                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
    
                        let raise_bytes = raise.to_le_bytes();
    
                        for x in 0..raise_bytes.len() {
                            memory.bytes[(addr + x as u32 + bytes.len() as u32) as usize] = raise_bytes[x as usize];
                        }
    
                    } else { // new value
       
                        let bytes = dec8.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0) as u32;
                        set_addr = true;
    
                        let raise_bytes = raise.to_le_bytes();
    
                        let new_addr2 = memory.malloc(raise_bytes.to_vec()).unwrap_or(0);
    
                        if addr == 0 || new_addr2 == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };


                Ok(true)
            },
            _ => {
                Err("Not a dec8 type!")
            }
        }
    }
*/
    pub fn to_int8(&self) -> Option<i8> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int8 => {
                match self.get_1_byte() {
                    Some(x) => {
                        Some(i8::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_int8(&mut self, int8: i8) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int8 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = int8.to_le_bytes();
    
                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
    
                    } else { // new value
       
                        let bytes = int8.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;
    
                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a int8 type!")
            }
        }
    }

    pub fn to_int16(&self) -> Option<i16> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int16 => {
                match self.get_2_bytes() {
                    Some(x) => {
                        Some(i16::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_int16(&mut self, int16: i16) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int16 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = int16.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value

                        let bytes = int16.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a int16 type!")
            }
        }
    }

    pub fn to_int32(&self) -> Option<i32> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int32 => {
                match self.get_4_bytes() {
                    Some(x) => {
                        Some(i32::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_int32(&mut self, int32: i32) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int32 => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = int32.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = int32.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }

                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a int32 type!")
            }
        }
    }

    pub fn to_int64(&self) -> Option<i64> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int64 => {
                match self.get_8_bytes() {
                    Some(x) => {
                        Some(i64::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_int64(&mut self, int64: i64) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int64 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = int64.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = int64.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }

                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a int64 type!")
            }
        }
    }

    pub fn to_uint8(&self) -> Option<u8> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint8 => {
                match self.get_1_byte() {
                    Some(x) => {
                        Some(u8::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_uint8(&mut self, uint8: u8) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint8 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = uint8.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint8.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a uint8 type!")
            }
        }
    }

    pub fn to_uint16(&self) -> Option<u16> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint16 => {
                match self.get_2_bytes() {
                    Some(x) => {
                        Some(u16::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_uint16(&mut self, uint16: u16) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint16 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = uint16.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint16.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a uint16 type!")
            }
        }
    }

    pub fn to_uint32(&self) -> Option<u32> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint32 => {
                match self.get_4_bytes() {
                    Some(x) => {
                        Some(u32::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_uint32(&mut self, uint32: u32) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint32 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = uint32.to_le_bytes();
    
                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
    
                    } else { // new value
       
                        let bytes = uint32.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;
    
                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a uint32 type!")
            }
        }
    }

    pub fn to_uint64(&self) -> Option<u64> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint64 => {
                match self.get_8_bytes() {
                    Some(x) => {
                        Some(u64::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_uint64(&mut self, uint64: u64) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint64 => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();
                    
                    if addr != 0 { // existing value, replace
                        let bytes = uint64.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint64.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a uint64 type!")
            }
        }
    }

    pub fn to_float(&self) -> Option<f32> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Float => {
                match self.get_4_bytes() {
                    Some(x) => {
                        Some(f32::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_float(&mut self, float: f32) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Float => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = float.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = float.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }   

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a float type!")
            }
        }
    }

    pub fn to_double(&self) -> Option<f64> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Double => {
                match self.get_8_bytes() {
                    Some(x) => {
                        Some(f64::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_double(&mut self, double: f64) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Double => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = double.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = double.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a double type!")
            }
        }
    }

    pub fn to_option(&self) -> Option<String> {

        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Enum { choices } => {
                match self.get_1_byte() {
                    Some(x) => {

                        let value_num = u8::from_le_bytes(x) as usize;

                        if value_num > choices.len() {
                            return None;
                        }

                        Some(choices[value_num].clone())
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_option(&mut self, option: String) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Enum { choices } => {

                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;

                    for opt in choices {
                        if option == opt.to_string() {
                            value_num = ct as i32;
                        }
                        ct += 1;
                    };

                    if value_num == -1 {
                        return Err("Option not found, cannot set uknown option!");
                    }
                }

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    let bytes = (value_num as u8).to_le_bytes();

                    if addr != 0 { // existing value, replace

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
                
            },
            _ => {
                Err("Not a option type!")
            }
        }
    }

    pub fn to_boolean(&self) -> Option<bool> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Boolean => {
                match self.get_1_byte() {
                    Some(x) => {
                        Some(if x[0] == 1 { true } else { false })
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_boolean(&mut self, boolean: bool) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Boolean => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = if boolean == true {
                            [1] as [u8; 1]
                        } else {
                            [0] as [u8; 1]
                        };

                        // overwrite existing values in buffer
                        memory.bytes[addr as usize] = bytes[0];

                    } else { // new value
    
                        let bytes = if boolean == true {
                            [1] as [u8; 1]
                        } else {
                            [0] as [u8; 1]
                        };

                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a boolean type!")
            }
        }
    }

    pub fn to_geo(&self) -> Option<NoProtoGeo> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Geo16 => {
                match self.get_16_bytes() {
                    Some(x) => {
                        let mut bytes_lat: [u8; 8] = [0; 8];
                        let mut bytes_lon: [u8; 8] = [0; 8];
    
                        for i in 0..x.len() {
                            if i < 8 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 8] = x[i as usize];
                            }
                        }

                        Some(NoProtoGeo { lat: f64::from_le_bytes(bytes_lat), lon: f64::from_le_bytes(bytes_lon)})
                    },
                    None => None
                }
            },
            NoProtoSchemaKinds::Geo8 => {
                match self.get_8_bytes() {
                    Some(x) => {
                        let mut bytes_lat: [u8; 4] = [0; 4];
                        let mut bytes_lon: [u8; 4] = [0; 4];
    
                        for i in 0..x.len() {
                            if i < 4 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 4] = x[i as usize];
                            }
                        }

                        let lat = i32::from_le_bytes(bytes_lat) as f64;
                        let lon = i32::from_le_bytes(bytes_lon) as f64;

                        let dev = 10000000f64;

                        Some(NoProtoGeo { lat: lat / dev, lon: lon / dev})
                    },
                    None => None
                }
            },
            NoProtoSchemaKinds::Geo4 => {
                match self.get_4_bytes() {
                    Some(x) => {
                        let mut bytes_lat: [u8; 2] = [0; 2];
                        let mut bytes_lon: [u8; 2] = [0; 2];
    
                        for i in 0..x.len() {
                            if i < 2 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 2] = x[i as usize];
                            }
                        }

                        let lat = i16::from_le_bytes(bytes_lat) as f64;
                        let lon = i16::from_le_bytes(bytes_lon) as f64;

                        let dev = 100f64;

                        Some(NoProtoGeo { lat: lat / dev, lon: lon / dev})
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_geo(&mut self, geo: NoProtoGeo) -> std::result::Result<bool, &'static str> {

        let mut addr = self.get_value_address();
        let mut set_addr = false;

        {

            let mut memory = self.memory.borrow_mut();

            let model = self.schema;

            let value_bytes_size = match *model.kind {
                NoProtoSchemaKinds::Geo16 => { 16 },
                NoProtoSchemaKinds::Geo8 => { 8 },
                NoProtoSchemaKinds::Geo4 => { 4 },
                _ => { 0 }
            };

            if value_bytes_size == 0 {
                return Err("Not a geo type!");
            }

            let half_value_bytes = value_bytes_size / 2;

            // convert input values into bytes
            let value_bytes = match *model.kind {
                NoProtoSchemaKinds::Geo16 => {
                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = geo.lat.to_le_bytes();
                    let lon_bytes = geo.lon.to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                NoProtoSchemaKinds::Geo8 => {
                    let dev = 10000000f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((geo.lat * dev) as i32).to_le_bytes();
                    let lon_bytes = ((geo.lon * dev) as i32).to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                NoProtoSchemaKinds::Geo4 => {
                    let dev = 100f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((geo.lat * dev) as i16).to_le_bytes();
                    let lon_bytes = ((geo.lon * dev) as i16).to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                _ => {
                    [0; 16]
                }
            };

            if addr != 0 { // existing value, replace

                // overwrite existing values in buffer
                for x in 0..value_bytes.len() {
                    if x < value_bytes_size {
                        memory.bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                    }
                }

            } else { // new value

                addr = match *model.kind {
                    NoProtoSchemaKinds::Geo16 => { memory.malloc([0; 16].to_vec()).unwrap_or(0) },
                    NoProtoSchemaKinds::Geo8 => { memory.malloc([0; 8].to_vec()).unwrap_or(0) },
                    NoProtoSchemaKinds::Geo4 => { memory.malloc([0; 4].to_vec()).unwrap_or(0) },
                    _ => { 0 }
                };
                set_addr = true;

                if addr == 0 {
                    return Err("Not enough memory!");
                }

                // set values in buffer
                for x in 0..value_bytes.len() {
                    if x < value_bytes_size {
                        memory.bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                    }
                }
            }
        }

        if set_addr { self.set_value_address(addr) };

        Ok(true)
    }

    pub fn to_uuid(&self) -> Option<NoProtoUUID> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uuid => {
                match self.get_16_bytes() {
                    Some(x) => {
                        Some(NoProtoUUID { value: x})
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_uuid(&mut self, uuid: NoProtoUUID) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uuid => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = uuid.value;

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uuid.value;
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a uuid type!")
            }
        }
    }

    pub fn to_time_id(&self) -> Option<NoProtoTimeID> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Tid => {
                match self.get_16_bytes() {
                    Some(x) => {
                        let mut id_bytes: [u8; 8] = [0; 8];
                        id_bytes.copy_from_slice(&x[0..8]);

                        let mut time_bytes: [u8; 8] = [0; 8];
                        time_bytes.copy_from_slice(&x[8..16]);

                        Some(NoProtoTimeID {
                            id: id_bytes,
                            time: u64::from_le_bytes(time_bytes)
                        })
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_time_id(&mut self, time_id: NoProtoTimeID) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Tid => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace

                        let time_bytes = time_id.time.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..16 {
                            if x < 8 {
                                memory.bytes[(addr + x as u32) as usize] = time_id.id[x as usize];
                            } else {
                                memory.bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                            }
                        }

                    } else { // new value
    
                        let mut bytes: [u8; 16] = [0; 16];
                        let time_bytes = time_id.time.to_le_bytes();

                        for x in 0..bytes.len() {
                            if x < 8 {
                                bytes[(addr + x as u32) as usize] = time_id.id[x as usize];
                            } else {
                                bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                            }
                        }

                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a time_id type!")
            }
        }
    }

    pub fn to_date(&self) -> Option<u64> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Date => {
                match self.get_8_bytes() {
                    Some(x) => {
                        Some(u64::from_le_bytes(x))
                    },
                    None => None
                }
            },
            _ => {
                None
            }
        }
    }

    pub fn set_date(&mut self, date: u64) -> std::result::Result<bool, &'static str> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Date => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.borrow_mut();

                    if addr != 0 { // existing value, replace
                        let bytes = date.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = date.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec()).unwrap_or(0);
                        set_addr = true;

                        if addr == 0 {
                            return Err("Not enough memory!");
                        }
                    }                    
                }

                if set_addr { self.set_value_address(addr) };

                Ok(true)
            },
            _ => {
                Err("Not a date type!")
            }
        }
    }

}

/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
pub enum NoProtoDataType {
    none,
    /*table {
        head: u32
    },
    map {
        head: u32
    },
    list {
        head: u32,
        tail: u32,
        size: u16
    },
    tuple {
        head: u32
    },*/
    utf8_string { size: u32, value: String },
    bytes { size: u32, value: Vec<u8> },
    int8 { value: i8 },
    int16 { value: i16 },
    int32 { value: i32 },
    int64 { value: i64 }, 
    uint8 { value: u8 },
    uint16 { value: u16 },
    uint32 { value: u32 },
    uint64 { value: u64 },
    float { value: f32 }, // -3.4E+38 to +3.4E+38
    double { value: f64 }, // -1.7E+308 to +1.7E+308
    option { value: u8 }, // enum
    dec32 { value: i32, expo: i8},
    dec64 { value: i64, exp: i8},
    boolean { value: bool },
    geo_16 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_8 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_4 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: String }, // 16 bytes 21,267,647,932,558,653,966,460,912,964,485,513,216 possibilities (255^15 * 16) or over 2 quadrillion times more possibilites than stars in the universe
    time_id { id: String, time: u64 }, // 8 + 8 bytes
    date { value: u64 } // 8 bytes  
}*/

// Pointer -> String
/*impl From<&NoProtoPointer> for Option<String> {
    fn from(ptr: &NoProtoPointer) -> Option<String> {
        ptr.to_string()
    }
}*/

/*
// cast i64 => Pointer
impl From<i64> for NoProtoValue {
    fn from(num: i64) -> Self {
        NoProtoValue {
            loaded: false,
            address: 0,
            value: NoProtoValue::int64 { value: num },
            // model: None
        }
    }
}

// cast Pointer => Option<i64>
impl From<&NoProtoValue> for Option<i64> {
    fn from(ptr: &NoProtoValue) -> Option<i64> {
        match ptr.value {
            NoProtoValue::int64 { value } => {
                Some(value)
            }
            _ => None
        }
    }
}*/