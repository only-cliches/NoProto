use crate::collection::map::NoProtoMap;
use crate::collection::list::NoProtoList;
use crate::collection::table::NoProtoTable;
use crate::NoProtoMemory;
use std::cell::RefMut;
use core::cell::Ref;
use std::rc::Rc;
use std::cell::RefCell;
use std::result;
use json::*;
use std::{slice, ops::{ Index, IndexMut, Deref }};

pub struct NoProtoGeo {
    lat: f64,
    lon: f64
}

pub struct NoProtoTimeID {
    id: [u8; 16],
    time: u64
}

impl NoProtoTimeID {
    pub fn to_string(&self) -> String {

    }
}

pub struct NoProtoUUID {
    value: [u8; 32]
}

impl NoProtoUUID {
    pub fn to_string(&self) -> String {

    }
}

pub enum NoProtoPointerKinds {
    // scalar / collection
    standard {value: u32},

    // collection items
    map_item {value: u32, key: u32, next: u32},
    table_item {value: u32, i: u16, next: u32},
    list_item {value: u32, i: u16, next: u32}
}

pub struct NoProtoPointer {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
    // value: Option<NoProtoDataTypes>,
    type_string: String
}

impl NoProtoPointer {

    pub fn new_standard(address: u32, model: Rc<RefCell<JsonValue>>, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];
        let type_string;

        {
            let b_bytes = &memory.borrow().bytes;
            head.copy_from_slice(&b_bytes[addr..(addr+4)]);

            let b_model = model.borrow();
            type_string = b_model["type"].as_str().unwrap_or("").to_owned();
        }

        NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::standard { value: u32::from_le_bytes(head) },
            memory: memory,
            model: model,
            // value: None,
            type_string: type_string
        }
    }

    pub fn clear(&mut self) {

        let mut memory = self.memory.borrow_mut();

        for x in 0..4 {
            memory.bytes[(self.address + x) as usize] = 0;
        }

        // self.value = None;

        match self.kind {
            NoProtoPointerKinds::standard { mut value } => {
                value = 0;
            },
            NoProtoPointerKinds::map_item { mut value, key,  next } => {
                value = 0;
            },
            NoProtoPointerKinds::table_item { mut value, i, next } => {
                value = 0;
            },
            NoProtoPointerKinds::list_item { mut value, i, next } => {
                value = 0;
            }
        }
    }

    fn get_value(&self) -> u32 {
        match self.kind {
            NoProtoPointerKinds::standard { value } =>                { value },
            NoProtoPointerKinds::map_item { value, key,  next } =>    { value },
            NoProtoPointerKinds::table_item { value, i, next } =>     { value },
            NoProtoPointerKinds::list_item { value, i, next } =>      { value }
        }
    }

    pub fn as_table(&self) -> Option<NoProtoTable> {

        let type_str: &str = self.type_string.as_str(); 

        match type_str {
            "table" => {
                Some(NoProtoTable::new(self.address, Rc::clone(&self.memory), Rc::clone(&self.model)))
            },
            _ => {
                None
            }
        }
    }

    pub fn as_list(&self) -> Option<NoProtoList> {

    }

    pub fn as_map(&self) -> Option<NoProtoMap> {

    }
 
    pub fn to_string(&mut self) -> Option<String> {

        let type_str: &str = self.type_string.as_str(); 

        match type_str {
            "string" => {

                let value = self.get_value();

                // empty value
                if value == 0 {
                    return None;
                }
                
                // get size of string
                let addr = value as usize;
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

        let type_str: &str = self.type_string.as_str();

        match type_str {
            "string" => {
                let bytes = value.as_bytes();
                let str_size = bytes.len() as u32;

                if str_size >= (2 as u32).pow(32) - 1 { 
                    Err("String too large!")
                } else {

                    let mut memory = self.memory.borrow_mut();
                  
                    // first 4 bytes are string length
                    let addr = memory.malloc(str_size.to_le_bytes().to_vec()).unwrap_or(0);
                    // then string content
                    let addr2 = memory.malloc(bytes.to_vec()).unwrap_or(0);

                    if addr == 0 || addr2 == 0 {
                        return Err("Not enough memory!");
                    }
                    
                    // set pointer value to new address
                    self.kind = NoProtoPointerKinds::standard { value: addr };
                    let addr_bytes = addr.to_le_bytes();

                    for x in 0..4 {
                        memory.bytes[(self.address + x) as usize] = addr_bytes[x as usize];
                    }
            
                    Ok(true)
                }

            }
            _ => {
                Err("Not a string type!")
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
    boolean { value: bool },
    geo_64 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_32 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_16 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: String }, // 32 bytes
    time_id { id: String, time: u64 }, // 16 + 8 bytes
    date { value: u64 } // 8 bytes  
}*/

// Pointer -> String
impl From<&NoProtoPointer> for Option<String> {
    fn from(ptr: &NoProtoPointer) -> Option<String> {
        ptr.to_string()
    }
}



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