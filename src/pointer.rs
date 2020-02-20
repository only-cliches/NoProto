use std::cell::RefMut;
use core::cell::Ref;
use std::rc::Rc;
use crate::buffer::NoProtoBuffer;
use crate::collection::table::NoProtoTable;
use std::cell::RefCell;
use std::result;
use json::*;
use std::ops::{ Index, IndexMut, Deref };

pub enum NoProtoScalar {
    none,
    /*table,
    map,
    list {
        tail: u32,
        size: u16
    },
    map_item {
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


pub struct NoProtoValue<'a> {
    cached_value: NoProtoScalar,
    type_string: String,
    value_is_cached: bool,
    model: &'a Rc<RefCell<JsonValue>>,
    bytes: &'a Rc<RefCell<Vec<u8>>>,
}

impl<'a> NoProtoValue<'a> {


    pub fn new(address: u32, model: &'a Rc<RefCell<JsonValue>>, bytes: &'a Rc<RefCell<Vec<u8>>>) -> Self {
        /*
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];

        let b_bytes = bytes.borrow();
        
        head.copy_from_slice(&b_bytes[addr..(addr+4)]);*/

        let b_model = model.borrow();

        let this_type: &str = b_model["type"].as_str().unwrap_or("");
 
        NoProtoValue {
            type_string: this_type.to_owned(),
            cached_value: NoProtoScalar::none,
            value_is_cached: false,
            model: model,
            bytes: bytes
        }
    }


    fn str_type_to_enum(str_type: &str) -> NoProtoScalar {
        match str_type {
            // "list" => NoProtoScalar::list {tail: 0, size: 0},
            // "table" => NoProtoScalar::table,
            // "map" => NoProtoScalar::map,
            "string" => NoProtoScalar::utf8_string { size: 0, value: "".to_owned() },
            "bytes" => NoProtoScalar::bytes { size: 0, value: vec![] },
            "int8" => NoProtoScalar::int8 { value: 0 },
            "int32" => NoProtoScalar::int32 { value: 0 },
            "int64" => NoProtoScalar::int64 { value: 0 }, 
            "uint8" => NoProtoScalar::uint8 { value: 0 },
            "uint16" => NoProtoScalar::uint16 { value: 0 },
            "uint32" => NoProtoScalar::uint32 { value: 0 },
            "uint64" => NoProtoScalar::uint64 { value: 0 },
            "float" => NoProtoScalar::float { value: 0.0 }, 
            "double" => NoProtoScalar::double { value: 0.0 }, 
            "option" => NoProtoScalar::option { value: 0 }, 
            "bool" => NoProtoScalar::boolean { value: false },
            "boolean" => NoProtoScalar::boolean { value: false },
            "geo_16" => NoProtoScalar::geo_64 { lat: 0.0, lon: 0.0 },
            "geo_8" => NoProtoScalar::geo_32 { lat: 0, lon: 0 }, 
            "geo_4" => NoProtoScalar::geo_16 { lat: 0, lon: 0 },
            "uuid" => NoProtoScalar::uuid { value: "".to_owned() }, 
            "time_id" => NoProtoScalar::time_id { id: "".to_owned(), time: 0 }, 
            "date" => NoProtoScalar::date { value: 0 }, 
            _ => NoProtoScalar::none
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
                    /*
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
                    }*/
            
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
                /*
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
                }*/
                Ok("".to_owned())
            }
            _ => {
                Err("Not a string type!")
            }
        }
    }

    
    pub fn into_table(&self) -> std::result::Result<NoProtoTable, &'static str> {
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