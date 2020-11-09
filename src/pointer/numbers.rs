use crate::schema::NP_Schema_Ptr;
use alloc::vec::Vec;
use crate::utils::to_unsigned;
use crate::utils::to_signed;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON, json_flex::JSMAP};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{borrow::ToOwned};

/// The type of number being used
#[derive(Debug)]
pub enum NP_NumType {
    /// Unsigned integer type (only positive whole numbers)
    unsigned,
    /// Signed integer type (positive or negative whole numbers)
    signed,
    /// Decimal point numbers
    floating
}



macro_rules! noproto_number {
    ($t:ty, $str1: tt, $str2: tt, $tkey: expr, $numType: expr) => {

        impl<'num> NP_Value<'num> for $t {

            fn type_idx() -> (u8, String) { ($tkey as u8, $str1.to_owned()) }

            fn self_type_idx(&self) -> (u8, String) { ($tkey as u8, $str1.to_owned()) }

            fn schema_to_json(schema_ptr: &NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
                let mut schema_json = JSMAP::new();
                schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));
            
                if let Some(default) = Self::schema_default(&schema_ptr) {
                    let default_val = *default;
                    match $numType {
                        NP_NumType::signed => {
                            schema_json.insert("default".to_owned(), NP_JSON::Integer(default_val as i64));
                        },
                        NP_NumType::unsigned => {
                            schema_json.insert("default".to_owned(), NP_JSON::Integer(default_val as i64));
                        },
                        NP_NumType::floating => {
                            schema_json.insert("default".to_owned(), NP_JSON::Float(default_val as f64));
                        }
                    };
                    
                }
        
                Ok(NP_JSON::Dictionary(schema_json))
            }

            fn schema_default(ptr: &NP_Schema_Ptr) -> Option<Box<Self>> {

                let size = core::mem::size_of::<Self>();
                let has_default = ptr.schema.bytes[(ptr.address + 1)];
                if has_default == 0 {
                    return None
                }
                let default_bytes = &ptr.schema.bytes[(ptr.address + 2)..(ptr.address + 2 + size)];
                Some(<$t>::np_from_be_bytes(default_bytes))
            }
    
            fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

                let mut addr = ptr.kind.get_value_addr();
        
                if addr != 0 { // existing value, replace
                    let mut bytes = value.to_be_bytes();

                    match $numType {
                        NP_NumType::signed => {
                            bytes[0] = to_unsigned(bytes[0]);
                        },
                        _ => {}
                    };
        
                    let write_bytes = ptr.memory.write_bytes();
        
                    // overwrite existing values in buffer
                    for x in 0..bytes.len() {
                        write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
                    }
                    return Ok(ptr.kind);
                } else { // new value
        
                    let mut bytes = value.to_be_bytes();

                    match $numType {
                        NP_NumType::signed => {
                            bytes[0] = to_unsigned(bytes[0]);
                        },
                        _ => {}
                    };
        
                    addr = ptr.memory.malloc(bytes.to_vec())?;
                    return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
                }
                
            }
        
            fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
                let addr = ptr.kind.get_value_addr() as usize;
        
                // empty value
                if addr == 0 {
                    return Ok(None);
                }
        
                let read_memory = ptr.memory.read_bytes();
                let mut be_bytes = <$t>::default().to_be_bytes();
                for x in 0..be_bytes.len() {
                    be_bytes[x] = read_memory[addr + x];
                }

                match $numType {
                    NP_NumType::signed => {
                        be_bytes[0] = to_signed(be_bytes[0]);
                    },
                    _ => {}
                };
        
                Ok(Some(Box::new(<$t>::from_be_bytes(be_bytes))))
            }

            fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
                let this_value = Self::into_value(ptr.clone());
        
                match this_value {
                    Ok(x) => {
                        match x {
                            Some(y) => {
                                match $numType {
                                    NP_NumType::floating => NP_JSON::Float(*y as f64),
                                    _ => NP_JSON::Integer(*y as i64)
                                }
                            },
                            None => {
                                let value = <$t>::schema_default(&ptr.schema);

                                match value {
                                    Some(v) => {
                                        match $numType {
                                            NP_NumType::floating => { NP_JSON::Float(*v as f64) },
                                            _ => { NP_JSON::Integer(*v as i64) }
                                        }
                                    },
                                    None => NP_JSON::Null
                                }
                            }
                        }
                    },
                    Err(_e) => {
                        NP_JSON::Null
                    }
                }
            }

            fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
         
                if ptr.kind.get_value_addr() == 0 {
                    Ok(0) 
                } else {
                    Ok(core::mem::size_of::<Self>() as u32)
                }
            }

            fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {
        
                let type_str = NP_Schema::get_type(json_schema)?;
        
                if type_str == $str1 || type_str == $str2 {
        
                    let mut schema_data: Vec<u8> = Vec::new();
                    schema_data.push($tkey as u8);
        
                    match json_schema["default"] {
                        NP_JSON::Float(x) => {
                            schema_data.push(1);
                            schema_data.extend((x as $t).to_be_bytes().to_vec());
                        },
                        NP_JSON::Integer(x) => {
                            schema_data.push(1);
                            schema_data.extend((x as $t).to_be_bytes().to_vec());
                        },
                        _ => {
                            schema_data.push(0);
                        }
                    }
        
                    return Ok(Some(schema_data));
                }
                
                Ok(None)
            }
        }
    }
}

// signed integers
noproto_number!(i8,    "int8",  "i8", NP_TypeKeys::Int8  , NP_NumType::signed);
noproto_number!(i16,  "int16", "i16", NP_TypeKeys::Int16 , NP_NumType::signed);
noproto_number!(i32,  "int32", "i32", NP_TypeKeys::Int32 , NP_NumType::signed);
noproto_number!(i64,  "int64", "i64", NP_TypeKeys::Int64 , NP_NumType::signed);

// unsigned integers
noproto_number!(u8,   "uint8",  "u8", NP_TypeKeys::Uint8 , NP_NumType::unsigned);
noproto_number!(u16, "uint16", "u16", NP_TypeKeys::Uint16, NP_NumType::unsigned);
noproto_number!(u32, "uint32", "u32", NP_TypeKeys::Uint32, NP_NumType::unsigned);
noproto_number!(u64, "uint64", "u64", NP_TypeKeys::Uint64, NP_NumType::unsigned);

// floating point
noproto_number!(f32,  "float", "f32", NP_TypeKeys::Float , NP_NumType::floating);
noproto_number!(f64, "double", "f64", NP_TypeKeys::Double, NP_NumType::floating);

trait NP_BigEndian {
    fn np_from_be_bytes(_bytes: &[u8]) -> Box<Self> { panic!() }
}

impl NP_BigEndian for i8 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 1] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(i8::from_be_bytes(slice))
    }
}

impl NP_BigEndian for i16 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 2] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(i16::from_be_bytes(slice))
    }
}

impl NP_BigEndian for i32 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 4] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(i32::from_be_bytes(slice))
    }
}

impl NP_BigEndian for i64 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 8] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(i64::from_be_bytes(slice))
    }
}

impl NP_BigEndian for u8 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 1] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(u8::from_be_bytes(slice))
    }
}

impl NP_BigEndian for u16 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 2] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(u16::from_be_bytes(slice))
    }
}

impl NP_BigEndian for u32 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 4] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(u32::from_be_bytes(slice))
    }
}

impl NP_BigEndian for u64 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 8] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(u64::from_be_bytes(slice))
    }
}

impl NP_BigEndian for f32 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 4] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(f32::from_be_bytes(slice))
    }
}

impl NP_BigEndian for f64 {
    fn np_from_be_bytes(bytes: &[u8]) -> Box<Self> {
        let mut slice: [u8; 8] = Default::default();
        slice.copy_from_slice(bytes);
        Box::new(f64::from_be_bytes(slice))
    }
}