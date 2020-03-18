use crate::pointer::NP_ValueInto;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::{schema::NP_TypeKeys, pointer::NP_Value};
use super::NP_PtrKinds;

impl NP_Value for String {

    fn new<T: NP_Value + Default>() -> Self {
        String::default()
    }

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }
        
        // get size of string
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let str_size = u32::from_le_bytes(size) as usize;

        // get string bytes
        let array_bytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

        // convert to string
        let new_string = String::from_utf8(array_bytes.to_vec())?;

        Ok(Some(Box::new(new_string)))
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        if str_size >= std::u32::MAX as u64 { 
            Err(NP_Error::new("String too large!"))
        } else {

            let mut addr = kind.get_value() as usize;

            buffer.borrow_mut(|memory| {
                let prev_size: usize = if addr != 0 {
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                    u32::from_le_bytes(size_bytes) as usize
                } else {
                    0 as usize
                };

                if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
            
                    let size_bytes = (str_size as u32).to_le_bytes();
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
                    addr = memory.malloc((str_size as u32).to_le_bytes().to_vec())? as usize;

                    // then string content
                    memory.malloc(bytes.to_vec())?;

                    return Ok(memory.set_value_address(address, addr as u32, kind));
                }
            })
        }
    }
}


impl<'a> NP_ValueInto<'a> for String {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }
        
        // get size of string
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let str_size = u32::from_le_bytes(size) as usize;

        // get string bytes
        let array_bytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

        // convert to string
        Ok(Some(Box::new(String::from_utf8(array_bytes.to_vec())?)))
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

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Can't use '.get()' with type (&str). Cast to (String) instead!"))
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: &NP_Memory, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        if str_size >= std::u32::MAX as u64 { 
            Err(NP_Error::new("String too large!"))
        } else {

            let mut addr = kind.get_value() as usize;

            {
                let mut memory = buffer.try_borrow_mut()?;

                let prev_size: usize = if addr != 0 {
                    let mut size_bytes: [u8; 4] = [0; 4];
                    size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                    u32::from_le_bytes(size_bytes) as usize
                } else {
                    0 as usize
                };

                if prev_size >= str_size as usize { // previous string is larger than this one, use existing memory
            
                    let size_bytes = (str_size as u32).to_le_bytes();
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
                    addr = memory.malloc((str_size as u32).to_le_bytes().to_vec())? as usize;

                    // then string content
                    memory.malloc(bytes.to_vec())?;

                    return Ok(memory.set_value_address(address, addr as u32, kind)?);
                }
            }    
        }
    }
}

impl<'a> NP_ValueInto<'a> for &str {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        Err(NP_Error::new("Can't use '.into()' with type (&str). Cast to (String) instead!"))
    }
}*/