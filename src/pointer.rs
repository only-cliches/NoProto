use crate::buffer::NoProtoMemory;
use std::cell::RefMut;
use core::cell::Ref;
use std::rc::Rc;
use crate::buffer::NoProtoBuffer;
use crate::collection::table::NoProtoTable;
use std::cell::RefCell;
use std::result;
use json::*;
use std::ops::{ Index, IndexMut, Deref };


pub enum NoProtoDataTypes {
    none,
    table {
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
    utf8_string { size: u32, value: String },
    bytes { size: u32, value: Vec<u8> },
    int8 { value: i8 },
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
    time_id { id: String, time: u64 }, // 24 + 8 bytes
    date { value: u64 } // 8 bytes
}

pub enum NoProtoPointerKinds {
    // scalar / collection
    standard {value: u32},

    // collection items
    map_item {value: u32, key: u32, next: u32},
    table_item {value: u32, i: u32, next: u32},
    list_item {value: u32, i: u32, next: u32, prev: u32}
}

pub struct NoProtoPointer {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
    value: Option<NoProtoDataTypes>,
    type_string: String
}

impl NoProtoPointer {

    pub fn new_standard(address: u32, model: Rc<RefCell<JsonValue>>, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];
        let mut this_type: &str;

        {
            let b_bytes = memory.borrow().bytes;
            head.copy_from_slice(&b_bytes[addr..(addr+4)]);

            let b_model = model.borrow();
            this_type = b_model["type"].as_str().unwrap_or("");
        }


        NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::standard { value: u32::from_le_bytes(head) },
            memory: memory,
            model: model,
            value: None,
            type_string: this_type.to_owned()
        }
    }

    pub fn clear(&mut self) {

        let mut memory = self.memory.borrow_mut();

        for x in 0..4 {
            memory.bytes[(self.address + x) as usize] = 0;
        }

        self.value = None;

        match self.kind {
            NoProtoPointerKinds::standard { value } => {
                value = 0;
            },
            NoProtoPointerKinds::map_item { value, key,  next } => {
                value = 0;
            },
            NoProtoPointerKinds::table_item { value, i, next } => {
                value = 0;
            },
            NoProtoPointerKinds::list_item { value, i, next, prev } => {
                value = 0;
            }
        }

    }

    pub fn to_string(&mut self) -> Option<String> {

        let type_str: &str = self.type_string.as_str(); 

        match type_str {
            "string" => {

                match self.value { // check cache
                    Some(x) => {
                        match x {
                            NoProtoDataTypes::utf8_string {size, value} => {
                                Some(value)
                            },
                            _=> {
                                None
                            }
                        }
                    },
                    None => { // no cache, get value
                        match self.kind {
                            NoProtoPointerKinds::standard {value} => {
        
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
                                        Some(x)
                                    },
                                    Err(_e) => {
                                        // Err("Error parsing string!")
                                        None
                                    }
                                }
                            },
                            _ => {
                                // NoProtoResult::Err("Wrong pointer type!")
                                None
                            }
                        }
                    }
                }


            }
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
                    let addr = memory.malloc(str_size.to_le_bytes().to_vec());
                    // then string content
                    memory.malloc(bytes.to_vec());
                    
                    // set pointer value to new address
                    self.kind = NoProtoPointerKinds::standard { value: addr };
                    let addr_bytes = addr.to_le_bytes();

                    for x in 0..4 {
                        memory.bytes[(self.address + x) as usize] = addr_bytes[x as usize];
                    }

                    // set cache
                    self.value = Some(NoProtoDataTypes::utf8_string { size: str_size, value: value.to_string()});
            
                    Ok(true)
                }

            }
            _ => {
                Err("Not a string type!")
            }
        }
    }
}

pub struct NoProtoValue {
    value: NoProtoDataTypes,
    type_string: String,
}

impl NoProtoValue {
 
/*
    pub fn new(address: u32, model: Rc<RefCell<JsonValue>>, memory: Rc<RefCell<NoProtoMemory>>) -> Self {
        
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];

        let b_bytes = bytes.borrow();
        
        head.copy_from_slice(&b_bytes[addr..(addr+4)]);

        let b_model = model.borrow();

        let this_type: &str = b_model["type"].as_str().unwrap_or("");
 
        NoProtoValue {
            address: address,
            type_string: this_type.to_owned(),
            cached_value: NoProtoDataTypes::none,
            value_is_cached: false,
            model: model,
            memory: memory
        }
    }

    pub fn to_string(&self) -> std::result::Result<String, &'static str> {
        let type_str: &str = self.type_string.as_str();

        match type_str {
            "string" => {
                
                // get size of string
                let addr = self.address as usize;
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
                        Ok(x)
                    },
                    Err(_e) => {
                        Err("Error parsing string!")
                    }
                }
            }
            _ => {
                Err("Not a string type!")
            }
        }
    }

    
    pub fn to_table(&self) -> std::result::Result<NoProtoTable, &'static str> {
        let type_str: &str = self.type_string.as_str();

        match type_str {
            "table" => {
                Ok(NoProtoTable::new(Rc::new(RefCell::new(self))))
            },
            _ => {
                Err("Not a table type!")
            }
        }
    }
*/

    /*
    fn str_type_to_enum(str_type: &str) -> NoProtoDataTypes {
        match str_type {
            // "list" => NoProtoDataTypes::list {tail: 0, size: 0},
            // "table" => NoProtoDataTypes::table,
            // "map" => NoProtoDataTypes::map,
            "string" => NoProtoDataTypes::utf8_string { size: 0, value: "".to_owned() },
            "bytes" => NoProtoDataTypes::bytes { size: 0, value: vec![] },
            "int8" => NoProtoDataTypes::int8 { value: 0 },
            "int32" => NoProtoDataTypes::int32 { value: 0 },
            "int64" => NoProtoDataTypes::int64 { value: 0 }, 
            "uint8" => NoProtoDataTypes::uint8 { value: 0 },
            "uint16" => NoProtoDataTypes::uint16 { value: 0 },
            "uint32" => NoProtoDataTypes::uint32 { value: 0 },
            "uint64" => NoProtoDataTypes::uint64 { value: 0 },
            "float" => NoProtoDataTypes::float { value: 0.0 }, 
            "double" => NoProtoDataTypes::double { value: 0.0 }, 
            "option" => NoProtoDataTypes::option { value: 0 }, 
            "bool" => NoProtoDataTypes::boolean { value: false },
            "boolean" => NoProtoDataTypes::boolean { value: false },
            "geo_16" => NoProtoDataTypes::geo_64 { lat: 0.0, lon: 0.0 },
            "geo_8" => NoProtoDataTypes::geo_32 { lat: 0, lon: 0 }, 
            "geo_4" => NoProtoDataTypes::geo_16 { lat: 0, lon: 0 },
            "uuid" => NoProtoDataTypes::uuid { value: "".to_owned() }, 
            "time_id" => NoProtoDataTypes::time_id { id: "".to_owned(), time: 0 }, 
            "date" => NoProtoDataTypes::date { value: 0 }, 
            _ => NoProtoDataTypes::none
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
                  
                    // first 4 bytes are string length
                    let addr = self.malloc(str_size.to_le_bytes().to_vec())?;
                    // then string content
                    self.malloc(bytes.to_vec())?;
                    
                    // set new address value to the string we just saved
                    self.value = addr;
                    let addr_bytes = addr.to_le_bytes();

                    let mut buffer_bytes = self.bytes.borrow_mut();

                    for x in 0..4 {
                        buffer_bytes[(self.address + x) as usize] = addr_bytes[x as usize];
                    }
            
                    Ok(true)
                }

            }
            _ => {
                Err("Not a string type!")
            }
        }
    }*/


/*
    pub fn into_list(&self) -> std::result::Result<NoProtoList, &'static str> {

    }

    pub fn into_map(&self) -> std::result::Result<NoProtoMap, &'static str> {

    }
*/
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