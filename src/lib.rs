use std::result;
use json::*;
use std::collections::HashMap;
use std::ops::{ Index, IndexMut, Deref };

pub enum NoProtoValue {
    table {
        head: u32,
        tail: u32
    },
    list {
        head: u32,
        tail: u32,
        size: u16
    },
    map {
        head: u32,
        tail: u32
    },
    linked_item {
        prev: u32,
        next: u32,
        index: u16, // used by list and table 
        key_value: Vec<u8> // used by map
    }, 
    utf8_string { value: String },
    bytes { value: Vec<u8> },
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

pub struct NoProtoMapModel {
    keyType: String,
    valueType: String
}

pub struct NoProtoDataModel {
    colKey: String,
    colType: String,
    options: JsonValue,
    table: Option<Box<HashMap<String, NoProtoDataModel>>>, // nested type (table)
    list: Option<Box<NoProtoDataModel>>, // nested type (list)
    map: Option<Box<NoProtoMapModel>> // nested map type
}

pub struct NoProtoPointer {
    loaded: bool,
    address: u32,
    value: NoProtoValue,
    model: Option<NoProtoDataModel>
}

impl NoProtoPointer {

}

// cast i64 => Pointer
impl From<i64> for NoProtoPointer {
    fn from(num: i64) -> Self {
        NoProtoPointer {
            loaded: false,
            address: 0,
            value: NoProtoValue::int64 { value: num },
            model: None
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
}


impl<'a> Index<&'a str> for NoProtoPointer {
    type Output = NoProtoPointer;

    fn index(&self, index: &'a str) -> &NoProtoPointer {
        &NoProtoPointer {
            loaded: false,
            address: 0,
            value: NoProtoValue::int64 { value: 0 },
            model: None
        }
    }
}

fn main() {
    let mut xx: NoProtoPointer = 42.into();

    // let x: Option<i64> = xx.into();
    let y: Option<i64> = (&xx["value"]).into();
    // let g: i64 = y.into().unwrap();
}