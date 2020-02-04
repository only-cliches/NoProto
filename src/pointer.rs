use std::cell::RefMut;
use core::cell::Ref;
use std::rc::Rc;
use crate::buffer::NoProtoBuffer;
use crate::collection::table::NoProtoTable;
use std::cell::RefCell;
use std::result;
use json::*;
use std::ops::{ Index, IndexMut, Deref };

pub enum NoProtoValue {
    none,
    table, // address is head
    map, // address is head
    list { // address is head
        tail: u32,
        size: u16
    },
    /*map_item {
        key_address: u32,
        next_item: u32
    },
    list_item {
        next_item: u32,
        prev_item: u32
    },
    table_column {
        next_item: u32,
        item_index: i16
    },*/
    utf8_string { size: i32, value: String },
    bytes { size: i32, value: Vec<u8> },
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


pub struct NoProtoPointer<'a> {
    address: u32,
    value: u32,
    cached_value: NoProtoValue,
    type_string: String,
    value_is_cached: bool,
    model: &'a Rc<RefCell<JsonValue>>,
    bytes: &'a Rc<RefCell<Vec<u8>>>,
}

impl<'a> NoProtoPointer<'a> {


    pub fn new(address: u32, model: &'a Rc<RefCell<JsonValue>>, bytes: &'a Rc<RefCell<Vec<u8>>>) -> Self {
        
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];

        let b_bytes = bytes.borrow();
        
        head.copy_from_slice(&b_bytes[addr..(addr+4)]);

        let b_model = model.borrow();

        let this_type: &str = b_model["type"].as_str().unwrap_or("");
 
        NoProtoPointer {
            address: address, // the location of this pointer
            value: u32::from_le_bytes(head), // points to value in buffer
            type_string: this_type.to_owned(),
            cached_value: NoProtoValue::none,
            value_is_cached: false,
            model: model,
            bytes: bytes
        }
    }


    fn str_type_to_enum(str_type: &str) -> NoProtoValue {
        match str_type {
            "list" => NoProtoValue::list {tail: 0, size: 0},
            "table" => NoProtoValue::table,
            "map" => NoProtoValue::map,
            "string" => NoProtoValue::utf8_string { size: 0, value: "".to_owned() },
            "bytes" => NoProtoValue::bytes { size: 0, value: vec![] },
            "int8" => NoProtoValue::int8 { value: 0 },
            "int32" => NoProtoValue::int32 { value: 0 },
            "int64" => NoProtoValue::int64 { value: 0 }, 
            "uint8" => NoProtoValue::uint8 { value: 0 },
            "uint16" => NoProtoValue::uint16 { value: 0 },
            "uint32" => NoProtoValue::uint32 { value: 0 },
            "uint64" => NoProtoValue::uint64 { value: 0 },
            "float" => NoProtoValue::float { value: 0.0 }, 
            "double" => NoProtoValue::double { value: 0.0 }, 
            "option" => NoProtoValue::option { value: 0 }, 
            "bool" => NoProtoValue::boolean { value: false },
            "boolean" => NoProtoValue::boolean { value: false },
            "geo_16" => NoProtoValue::geo_64 { lat: 0.0, lon: 0.0 },
            "geo_8" => NoProtoValue::geo_32 { lat: 0, lon: 0 }, 
            "geo_4" => NoProtoValue::geo_16 { lat: 0, lon: 0 },
            "uuid" => NoProtoValue::uuid { value: "".to_owned() }, 
            "time_id" => NoProtoValue::time_id { id: "".to_owned(), time: 0 }, 
            "date" => NoProtoValue::date { value: 0 }, 
            _ => NoProtoValue::none
        }
    }

    fn malloc(&mut self, bytes: Vec<u8>) -> std::result::Result<u32, &'static str> {

        match self.bytes.try_borrow_mut() {
            Ok(mut buffer) => {
                let location: u32 = bytes.len() as u32;
                buffer.extend(bytes);
                Ok(location)
            },
            Err(err) => {
                Err("Failed to mutate buffer bytes!")
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
    }

    pub fn get_string(&self) -> std::result::Result<String, &'static str> {
        let type_str: &str = self.type_string.as_str();

        match type_str {
            "string" => {

                // get size of string
                let addr = self.value as usize;
                let mut size: [u8; 4] = [0; 4];
                let buffer_bytes = self.bytes.borrow();
                size.copy_from_slice(&buffer_bytes[addr..(addr+4)]);
                let str_size = u32::from_le_bytes(size) as usize;

                // get string bytes
                let arrayBytes = &buffer_bytes[(addr+4)..(addr+4+str_size)];

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

    /*
    pub fn into_table(&mut self) -> std::result::Result<NoProtoTable, &'static str> {

        Ok(NoProtoTable {

        })
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
impl From<i64> for NoProtoPointer {
    fn from(num: i64) -> Self {
        NoProtoPointer {
            loaded: false,
            address: 0,
            value: NoProtoValue::int64 { value: num },
            // model: None
        }
    }
}

// cast Pointer => Option<i64>
impl From<&NoProtoPointer> for Option<i64> {
    fn from(ptr: &NoProtoPointer) -> Option<i64> {
        match ptr.value {
            NoProtoValue::int64 { value } => {
                Some(value)
            }
            _ => None
        }
    }
}*/