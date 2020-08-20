use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};

impl NP_Value for i8 {

    fn is_type( type_str: &str) -> bool {
        "int8" == type_str || "i8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Int8 as i64, "int8".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Int8 as i64, "int8".to_owned()) }

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

        let offset = core::i8::MAX as i16;

        if addr != 0 { // existing value, replace
            let bytes = (((**value as i16) + offset) as u8).to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = (((**value as i16) + offset) as u8).to_be_bytes();
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

        let offset = core::i8::MAX as i16;

        Ok(match ptr.memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(((u8::from_be_bytes([x]) as i16) - offset) as i8))
            },
            None => None
        })
    }


    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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

impl NP_Value for i16 {

    fn is_type( type_str: &str) -> bool {
        "int16" == type_str || "i16" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Int16 as i64, "int16".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Int16 as i64, "int16".to_owned()) }

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

        let offset = core::i16::MAX as i32;

        if addr != 0 { // existing value, replace
            let bytes = (((**value as i32) + offset) as u16).to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = (((**value as i32) + offset) as u16).to_be_bytes();
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

        let offset = core::i16::MAX as i32;

        Ok(match ptr.memory.get_2_bytes(addr) {
            Some(x) => {
                Some(Box::new(((u16::from_be_bytes(*x) as i32) - offset) as i16))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for i32 {

    fn is_type( type_str: &str) -> bool {
        "int32" == type_str || "i32" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Int32 as i64, "int32".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Int32 as i64, "int32".to_owned()) }

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

        let offset = core::i32::MAX as i64;

        if addr != 0 { // existing value, replace
            let bytes = (((**value as i64) + offset) as u32).to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = (((**value as i64) + offset) as u32).to_be_bytes();
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

        let offset = core::i32::MAX as i64;

        Ok(match ptr.memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(((u32::from_be_bytes(*x) as i64) - offset) as i32))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for i64 {

    fn is_type( type_str: &str) -> bool {
        "int64" == type_str || "i64" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Int64 as i64, "int64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Int64 as i64, "int64".to_owned()) }

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

        let offset = core::i64::MAX as i128;

        if addr != 0 { // existing value, replace
            let bytes = (((**value as i128) + offset) as u64).to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = (((**value as i128) + offset) as u64).to_be_bytes();
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

        let offset = core::i64::MAX as i128;

        Ok(match ptr.memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(((u64::from_be_bytes(*x) as i128) - offset) as i64))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for u8 {

    fn is_type( type_str: &str) -> bool {
        "uint8" == type_str || "u8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uint8 as i64, "uint8".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uint8 as i64, "uint8".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(u8::from_be_bytes([x])))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for u16 {

    fn is_type( type_str: &str) -> bool {
        "uint16" == type_str || "u16" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uint16 as i64, "uint16".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uint16 as i64, "uint16".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_2_bytes(addr) {
            Some(x) => {
                Some(Box::new(u16::from_be_bytes(*x)))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for u32 {
    fn is_type( type_str: &str) -> bool {
        "uint32" == type_str || "u32" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uint32 as i64, "uint32".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uint32 as i64, "uint32".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(u32::from_be_bytes(*x)))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for u64 {

    fn is_type( type_str: &str) -> bool {
        "uint64" == type_str || "u64" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uint64 as i64, "uint64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uint64 as i64, "uint64".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(u64::from_be_bytes(*x)))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(*y as i64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for f32 {

    fn is_type( type_str: &str) -> bool {
        "float" == type_str || "f32" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Float as i64, "float".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Float as i64, "float".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(f32::from_be_bytes(*x)))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Float(*y as f64)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}

impl NP_Value for f64 {

    fn is_type( type_str: &str) -> bool {
        "double" == type_str || "f64" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Double as i64, "double".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Double as i64, "double".to_owned()) }

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
            let bytes = value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let bytes = value.to_be_bytes();
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

        Ok(match ptr.memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(f64::from_be_bytes(*x)))
            },
            None => None
        })
    }
    
    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Float(*y)
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
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<Self>() as u32)
        }
    }

}