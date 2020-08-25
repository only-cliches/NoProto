use crate::utils::to_unsigned;
use crate::utils::to_signed;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};

macro_rules! noproto_number {
    ($t:ty, $str1: tt, $str2: tt, $tkey: expr, $signedInt: expr, $fp: expr) => {
        impl NP_Value for $t {
            fn is_type( type_str: &str) -> bool {
                $str1 == type_str || $str2 == type_str
            }
            fn type_idx() -> (i64, String) { ($tkey as i64, $str1.to_owned()) }
            fn self_type_idx(&self) -> (i64, String) { ($tkey as i64, $str1.to_owned()) }
            fn schema_default(schema: Rc<NP_Schema>) -> Option<Box<Self>> {
                match &schema.default {
                    Some(x) => {
                        match x {
                            NP_JSON::Integer(value) => {
                                Some(Box::new(*value as Self))
                            },
                            NP_JSON::Float(value) => {
                                Some(Box::new(*value as Self))
                            },
                            _ => {
                                None
                            }
                        }
                    },
                    None => {
                        None
                    }
                }
            }
            fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

                let mut addr = ptr.kind.get_value_addr();
        
                if addr != 0 { // existing value, replace
                    let mut bytes = value.to_be_bytes();

                    if $signedInt {
                        bytes[0] = to_unsigned(bytes[0]);
                    }
        
                    let write_bytes = ptr.memory.write_bytes();
        
                    // overwrite existing values in buffer
                    for x in 0..bytes.len() {
                        write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
                    }
                    return Ok(ptr.kind);
                } else { // new value
        
                    let mut bytes = value.to_be_bytes();

                    if $signedInt {
                        bytes[0] = to_unsigned(bytes[0]);
                    }
        
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

                if $signedInt {
                    be_bytes[0] = to_signed(be_bytes[0]);
                }
        
                Ok(Some(Box::new(<$t>::from_be_bytes(be_bytes))))
            }

            fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
                let this_string = Self::into_value(ptr.clone());
        
                match this_string {
                    Ok(x) => {
                        match x {
                            Some(y) => {
                                if $fp {
                                    NP_JSON::Float(*y as f64)
                                } else {
                                    NP_JSON::Integer(*y as i64)
                                }
                            },
                            None => {
                                match &ptr.schema.default {
                                    Some(x) => x.clone(),
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
        }
    }
}

// signed integers
noproto_number!(i8, "int8", "i8", NP_TypeKeys::Int8, true, false);
noproto_number!(i16, "int16", "i16", NP_TypeKeys::Int16, true, false);
noproto_number!(i32, "int32", "i32", NP_TypeKeys::Int32, true, false);
noproto_number!(i64, "int64", "i64", NP_TypeKeys::Int64, true, false);

// unsigned integers
noproto_number!(u8, "uint8", "u8", NP_TypeKeys::Uint8, false, false);
noproto_number!(u16, "uint16", "u16", NP_TypeKeys::Uint16, false, false);
noproto_number!(u32, "uint32", "u32", NP_TypeKeys::Uint32, false, false);
noproto_number!(u64, "uint64", "u64", NP_TypeKeys::Uint64, false, false);

// floating point
noproto_number!(f32, "float", "f32", NP_TypeKeys::Float, false, true);
noproto_number!(f64, "double", "f64", NP_TypeKeys::Double, false, true);