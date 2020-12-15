//! NoProto supports a large number of native number types.
//! 
//! Signed Integers: <br/>
//! [`i8`](https://doc.rust-lang.org/std/primitive.i8.html), [`i16`](https://doc.rust-lang.org/std/primitive.i16.html), [`i32`](https://doc.rust-lang.org/std/primitive.i32.html), [`i64`](https://doc.rust-lang.org/std/primitive.i64.html) <br/>
//! <br/>
//! Unsigned Integers: <br/>
//! [`u8`](https://doc.rust-lang.org/std/primitive.u8.html), [`u16`](https://doc.rust-lang.org/std/primitive.u16.html), [`u32`](https://doc.rust-lang.org/std/primitive.u32.html), [`u64`](https://doc.rust-lang.org/std/primitive.u64.html) <br/>
//! <br/>
//! Floating Point: <br/>
//! [`f32`](https://doc.rust-lang.org/std/primitive.f32.html), [`f64`](https://doc.rust-lang.org/std/primitive.f64.html)
//! <br/>
//! 
//! The details of using each number type is identical to the pattern below.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "u32"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], 20380u32)?;
//! 
//! assert_eq!(20380u32, new_buffer.get::<u32>(&[])?.unwrap());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 


use alloc::prelude::v1::Box;
use crate::schema::NP_Parsed_Schema;
use alloc::vec::Vec;
use crate::utils::to_unsigned;
use crate::utils::to_signed;
use crate::error::NP_Error;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON, json_flex::JSMAP};

use alloc::string::ToString;
use alloc::{borrow::ToOwned};
use super::{NP_Cursor};
use crate::NP_Memory;

/// The type of number being used
#[derive(Debug)]
#[doc(hidden)]
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

        impl<'value> NP_Value<'value> for $t {

            fn type_idx() -> (&'value str, NP_TypeKeys) { ($str1, $tkey) }

            fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ($str1, $tkey) }

            fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
                let mut schema_json = JSMAP::new();
                schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));
            
                if let Some(default) = <$t>::np_get_default(&schema[address]) {
                    let default_val = default;
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

            fn schema_default<'default>(schema: &'default NP_Parsed_Schema) -> Option<Self> {
                <$t>::np_get_default(&schema)
            }
    
            fn set_value<'set>(cursor: NP_Cursor, memory: &'set NP_Memory, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {

                let c_value = cursor.get_value(memory);

                let mut value_address = c_value.get_addr_value() as usize;

                if value_address != 0 { // existing value, replace
                    let mut bytes = value.to_be_bytes();

                    match $numType {
                        NP_NumType::signed => {
                            bytes[0] = to_unsigned(bytes[0]);
                        },
                        _ => {}
                    };
        
                    let write_bytes = memory.write_bytes();
        
                    // overwrite existing values in buffer
                    for x in 0..bytes.len() {
                        write_bytes[value_address + x] = bytes[x];
                    }
                    return Ok(cursor);
                } else { // new value
        
                    let mut bytes = value.to_be_bytes();

                    match $numType {
                        NP_NumType::signed => {
                            bytes[0] = to_unsigned(bytes[0]);
                        },
                        _ => {}
                    };
        
                    value_address = memory.malloc_borrow(&bytes)?;
                    c_value.set_addr_value(value_address as u16);

                    return Ok(cursor);
                }
                
            }
        
            fn into_value(cursor: &NP_Cursor, memory: &'value NP_Memory) -> Result<Option<Self>, NP_Error> where Self: Sized {

                let c_value = cursor.get_value(memory);

                let value_addr = c_value.get_addr_value() as usize;
        
                // empty value
                if value_addr == 0 {
                    return Ok(None);
                }
        
                let read_memory = memory.read_bytes();
                let mut be_bytes = <$t>::default().to_be_bytes();
                for x in 0..be_bytes.len() {
                    be_bytes[x] = read_memory[value_addr + x];
                }

                match $numType {
                    NP_NumType::signed => {
                        be_bytes[0] = to_signed(be_bytes[0]);
                    },
                    _ => {}
                };

                Ok(Some(<$t>::from_be_bytes(be_bytes)))
            }

            fn to_json(cursor: &NP_Cursor, memory: &'value NP_Memory) -> NP_JSON {

                match Self::into_value(cursor, memory) {
                    Ok(x) => {
                        match x {
                            Some(y) => {
                                match $numType {
                                    NP_NumType::floating => NP_JSON::Float(y as f64),
                                    _ => NP_JSON::Integer(y as i64)
                                }
                            },
                            None => {
                                let schema = &memory.schema[cursor.schema_addr];
                                match <$t>::schema_default(&schema) {
                                    Some(v) => {
                                        match $numType {
                                            NP_NumType::floating => { NP_JSON::Float(v as f64) },
                                            _ => { NP_JSON::Integer(v as i64) }
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

            fn get_size(cursor: &NP_Cursor, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {

                let c_value = cursor.get_value(memory);

                if c_value.get_addr_value() == 0 {
                    Ok(0) 
                } else {
                    Ok(core::mem::size_of::<Self>())
                }
            }

            fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        
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
                };

                let use_schema = match $tkey {
                    NP_TypeKeys::Int8 => {
                        NP_Parsed_Schema::Int8 { sortable: true, i: $tkey, default: i8::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Int16 => {
                        NP_Parsed_Schema::Int16 { sortable: true, i: $tkey, default: i16::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Int32 => {
                        NP_Parsed_Schema::Int32 { sortable: true, i: $tkey, default: i32::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Int64 => {
                        NP_Parsed_Schema::Int64 { sortable: true, i: $tkey, default: i64::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Uint8 => {
                        NP_Parsed_Schema::Uint8 { sortable: true, i: $tkey, default: u8::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Uint16 => {
                        NP_Parsed_Schema::Uint16 { sortable: true, i: $tkey, default: u16::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Uint32 => {
                        NP_Parsed_Schema::Uint32 { sortable: true, i: $tkey, default: u32::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Uint64 => {
                        NP_Parsed_Schema::Uint64 { sortable: true, i: $tkey, default: u64::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Float => {
                        NP_Parsed_Schema::Float { sortable: false, i: $tkey, default: f32::np_get_default_from_json(&json_schema["default"])}
                    },
                    NP_TypeKeys::Double => {
                        NP_Parsed_Schema::Double { sortable: false, i: $tkey, default: f64::np_get_default_from_json(&json_schema["default"])}
                    },
                    _ => { unreachable!() }
                };

                schema.push(use_schema);

                return Ok((true, schema_data, schema));
            
            }

            fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
                schema.push(match $tkey {
                    NP_TypeKeys::Int8 => {
                        NP_Parsed_Schema::Int8 { sortable: true, i: $tkey, default: i8::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Int16 => {
                        NP_Parsed_Schema::Int16 { sortable: true, i: $tkey, default: i16::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Int32 => {
                        NP_Parsed_Schema::Int32 { sortable: true, i: $tkey, default: i32::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Int64 => {
                        NP_Parsed_Schema::Int64 { sortable: true, i: $tkey, default: i64::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Uint8 => {
                        NP_Parsed_Schema::Uint8 { sortable: true, i: $tkey, default: u8::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Uint16 => {
                        NP_Parsed_Schema::Uint16 { sortable: true, i: $tkey, default: u16::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Uint32 => {
                        NP_Parsed_Schema::Uint32 { sortable: true, i: $tkey, default: u32::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Uint64 => {
                        NP_Parsed_Schema::Uint64 { sortable: true, i: $tkey, default: u64::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Float => {
                        NP_Parsed_Schema::Float { sortable: false, i: $tkey, default: f32::np_get_default_from_bytes(address, bytes)}
                    },
                    NP_TypeKeys::Double => {
                        NP_Parsed_Schema::Double { sortable: false, i: $tkey, default: f64::np_get_default_from_bytes(address, bytes)}
                    },
                    _ => { unreachable!() }
                });
                (schema[schema.len() - 1].is_sortable(), schema)
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


impl super::NP_Scalar for i8 {}
impl super::NP_Scalar for i16 {}
impl super::NP_Scalar for i32 {}
impl super::NP_Scalar for i64 {}
impl super::NP_Scalar for u8 {}
impl super::NP_Scalar for u16 {}
impl super::NP_Scalar for u32 {}
impl super::NP_Scalar for u64 {}
impl super::NP_Scalar for f32 {}
impl super::NP_Scalar for f64 {}

trait NP_BigEndian {
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> where Self: Sized;
    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> where Self: Sized;
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> where Self: Sized;
}


impl NP_BigEndian for i8 {

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Int8 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 1] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 2)]);
            Some(i8::from_be_bytes(slice))
        }
    }
}

#[test]
fn i8_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"int8\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"int8\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn i8_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i8\",\"default\":56}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<i8>(&[])?.unwrap(), 56i8);

    Ok(())
}

#[test]
fn i8_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i8\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 56i8)?;
    assert_eq!(buffer.get::<i8>(&[])?.unwrap(), 56i8);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<i8>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for i16 {

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Int16 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 2] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 3)]);
            Some(i16::from_be_bytes(slice))
        }
    }
}

#[test]
fn i16_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"int16\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"int16\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn i16_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i16\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<i16>(&[])?.unwrap(), 293i16);

    Ok(())
}

#[test]
fn i16_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i16\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293i16)?;
    assert_eq!(buffer.get::<i16>(&[])?.unwrap(), 293i16);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<i16>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for i32 {
           

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Int32 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 4] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 5)]);
            Some(i32::from_be_bytes(slice))
        }
    }
}

#[test]
fn i32_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"int32\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"int32\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn i32_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i32\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<i32>(&[])?.unwrap(), 293i32);

    Ok(())
}

#[test]
fn i32_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i32\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293i32)?;
    assert_eq!(buffer.get::<i32>(&[])?.unwrap(), 293i32);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<i32>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for i64 {

           

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Int64 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 8] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 9)]);
            Some(i64::from_be_bytes(slice))
        }
    }
}

#[test]
fn i64_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"int64\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"int64\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn i64_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i64\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<i64>(&[])?.unwrap(), 293i64);

    Ok(())
}

#[test]
fn i64_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"i64\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293i64)?;
    assert_eq!(buffer.get::<i64>(&[])?.unwrap(), 293i64);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<i64>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for u8 {

           

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Uint8 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 1] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 2)]);
            Some(u8::from_be_bytes(slice))
        }
    }
}


#[test]
fn u8_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uint8\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"uint8\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn u8_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u8\",\"default\":198}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<u8>(&[])?.unwrap(), 198u8);

    Ok(())
}

#[test]
fn u8_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u8\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 198u8)?;
    assert_eq!(buffer.get::<u8>(&[])?.unwrap(), 198u8);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<u8>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for u16 {



    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Uint16 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 2] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 3)]);
            Some(u16::from_be_bytes(slice))
        }
    }
}

#[test]
fn u16_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uint16\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"uint16\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn u16_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u16\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<u16>(&[])?.unwrap(), 293u16);

    Ok(())
}

#[test]
fn u16_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u16\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293u16)?;
    assert_eq!(buffer.get::<u16>(&[])?.unwrap(), 293u16);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<u16>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for u32 {

        

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Uint32 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 4] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 5)]);
            Some(u32::from_be_bytes(slice))
        }
    }
}

#[test]
fn u32_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uint32\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"uint32\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn u32_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u32\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<u32>(&[])?.unwrap(), 293u32);

    Ok(())
}

#[test]
fn u32_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u32\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293u32)?;
    assert_eq!(buffer.get::<u32>(&[])?.unwrap(), 293u32);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<u32>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for u64 {

           
    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Uint64 { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 8] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 9)]);
            Some(u64::from_be_bytes(slice))
        }
    }
}

#[test]
fn u64_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uint64\",\"default\":20}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"uint64\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn u64_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u64\",\"default\":293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<u64>(&[])?.unwrap(), 293u64);

    Ok(())
}

#[test]
fn u64_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"u64\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 293u64)?;
    assert_eq!(buffer.get::<u64>(&[])?.unwrap(), 293u64);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<u64>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}

impl NP_BigEndian for f32 {


    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Float { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 4] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 5)]);
            Some(f32::from_be_bytes(slice))
        }
    }
}

#[test]
fn float_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"float\",\"default\":20.183000564575195}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"float\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn float_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"float\",\"default\":2983.2938}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<f32>(&[])?.unwrap(), 2983.2938f32);

    Ok(())
}

#[test]
fn float_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"float\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 2983.2938f32)?;
    assert_eq!(buffer.get::<f32>(&[])?.unwrap(), 2983.2938f32);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<f32>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}


impl NP_BigEndian for f64 {

    fn np_get_default<'default>(ptr: &'default NP_Parsed_Schema) -> Option<Self> {
        match ptr {
            NP_Parsed_Schema::Double { sortable: _, i: _, default } => { *default },
            _ => None
        }
    }
    fn np_get_default_from_json(json: &NP_JSON) -> Option<Self> {
        match json {
            NP_JSON::Float(x) => {
                Some(*x as Self)
            },
            NP_JSON::Integer(x) => {
                Some(*x as Self)
            },
            _ => {
                None
            }
        }
    }
    fn np_get_default_from_bytes<'default>(address: usize, bytes: &'default Vec<u8>) -> Option<Self> {
        if bytes[address + 1] == 0 {
            None
        } else {
            let mut slice: [u8; 8] = Default::default();
            slice.copy_from_slice(&bytes[(address + 1)..(address + 9)]);
            Some(f64::from_be_bytes(slice))
        }
    }
}

#[test]
fn double_schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"double\",\"default\":20.183000564575195}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"double\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn double_default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"double\",\"default\":2983.2938}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<f64>(&[])?.unwrap(), 2983.2938f64);

    Ok(())
}

#[test]
fn double_set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"double\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], 2983.2938f64)?;
    assert_eq!(buffer.get::<f64>(&[])?.unwrap(), 2983.2938f64);
    buffer.del(&[])?;
    assert_eq!(buffer.get::<f64>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}