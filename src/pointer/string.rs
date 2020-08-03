use alloc::vec::Vec;
use crate::pointer::NP_ValueInto;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value, utils::from_utf8_lossy, json_flex::JFObject};
use super::{NP_PtrKinds};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;


impl NP_Value for String {

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str || "utf-8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }

    fn schema_state(_type_string: &str, json_schema: &JFObject) -> core::result::Result<i64, NP_Error> {
        match json_schema["size"].into_i64() {
            Some(x) => {
                if *x > 0 && *x < (u32::MAX as i64) {
                    return Ok(*x);
                }
                return Ok(-1);
            },
            None => {
                return Ok(-1);
            }
        }
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        let mut addr = kind.get_value() as usize;

        let write_bytes = memory.write_bytes();

        if schema.type_state != -1 { // fixed size bytes
            let mut set_kind = kind.clone();

            if addr == 0 { // malloc new bytes

                let mut empty_bytes: Vec<u8> = Vec::with_capacity(schema.type_state as usize);
                for _x in 0..(schema.type_state as usize) {
                    empty_bytes.push(0);
                }
                
                addr = memory.malloc(empty_bytes)? as usize;

                // set location address
                set_kind = memory.set_value_address(address, addr as u32, kind);
            }

            for x in 0..(schema.type_state as usize) {
                if x < bytes.len() { // assign values of bytes
                    write_bytes[(addr + x)] = bytes[x];
                } else { // rest is zeros
                    write_bytes[(addr + x)] = 0;
                }
            }

            return Ok(set_kind)
        }

        // flexible size

        let prev_size: usize = if addr != 0 {
            let mut size_bytes: [u8; 4] = [0; 4];
            size_bytes.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
            u32::from_be_bytes(size_bytes) as usize
        } else {
            0 as usize
        };

        if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
    
            let size_bytes = (str_size as u32).to_be_bytes();

            // set string size
            for x in 0..size_bytes.len() {
                write_bytes[(addr + x) as usize] = size_bytes[x as usize];
            }

            // set bytes
            for x in 0..bytes.len() {
                write_bytes[(addr + x + 4) as usize] = bytes[x as usize];
            }

            return Ok(*kind);
        } else { // not enough space or space has not been allocted yet
            

            // first 4 bytes are string length
            addr = memory.malloc((str_size as u32).to_be_bytes().to_vec())? as usize;

            // then string content
            memory.malloc(bytes.to_vec())?;

            return Ok(memory.set_value_address(address, addr as u32, kind));
        }
            
    }

}


impl<'a> NP_ValueInto<'a> for String {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;

        if schema.type_state != -1 { // fixed size
            
            let size = schema.type_state as usize;
            
            // get bytes
            let bytes = &memory.read_bytes()[(addr)..(addr+size)];

            return Ok(Some(Box::new(from_utf8_lossy(bytes))))

        } else { // dynamic size
            // get size of bytes
            let mut size: [u8; 4] = [0; 4];
            size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
            let bytes_size = u32::from_be_bytes(size) as usize;

            // get bytes
            let bytes = &memory.read_bytes()[(addr+4)..(addr+4+bytes_size)];

            return Ok(Some(Box::new(from_utf8_lossy(bytes))))
        }
        
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        JFObject::String(*y)
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let value = kind.get_value();

        // empty value
        if value == 0 {
            return Ok(0)
        }
        
        // get size of bytes
        let addr = value as usize;
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer;

        if schema.type_state != -1 { // fixed size
            return Ok(schema.type_state as u32);
        } else { // flexible size
            size.copy_from_slice(&memory.read_bytes()[addr..(addr+4)]);
            let bytes_size = u32::from_be_bytes(size) as u32;
            
            // return total size of this string
            return Ok(bytes_size + 4);
        }
    }
}

/*
impl NP_Value for &str {

    fn new<T: NP_Value + Default>() -> Self {
        ""
    }

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Can't use '.get()' with type (&str). Cast to (String) instead!"))
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        if str_size >= core::u32::MAX as u64 { 
            Err(NP_Error::new("String too large!"))
        } else {

            let mut addr = kind.get_value() as usize;

            {
                let mut memory = buffer.try_borrow_mut()?;

                let prev_size: usize = if addr != 0 {
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                    u32::from_be_bytes(size_bytes) as usize
                } else {
                    0 as usize
                };

                if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
            
                    let size_bytes = (str_size as u32).to_be_bytes();
                    // set string size
                    for x in 0..size_bytes.len() {
                        memory.bytes[(addr + x) as usize] = size_bytes[x as usize];
                    }

                    // set bytes
                    for x in 0..bytes.len() {
                        memory.bytes[(addr + x + 4) as usize] = bytes[x as usize];
                    }

                    return Ok(*kind);
                } else { // not enough space or space has not been allocted yet
                    

                    // first 4 bytes are string length
                    addr = memory.malloc((str_size as u32).to_be_bytes().to_vec())? as usize;

                    // then string content
                    memory.malloc(bytes.to_vec())?;

                    return Ok(memory.set_value_address(address, addr as u32, kind)?);
                }
            }    
        }
    }
}

impl<'a> NP_ValueInto<'a> for &str {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Can't use '.into()' with type (&str). Cast to (String) instead!"))
    }
}*/