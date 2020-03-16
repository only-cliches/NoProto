use crate::schema::NoProtoSchema;
use crate::error::NoProtoError;
use crate::memory::NoProtoMemory;
use std::{cell::RefCell, rc::Rc};
use crate::{schema::NoProtoTypeKeys, pointer::NoProtoValue};
use super::NoProtoPointerKinds;

impl<'a> NoProtoValue<'a> for String {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        String::default()
    }

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::UTF8String as i64, "string".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }
        
        // get size of string
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer.try_borrow()?;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let str_size = u32::from_le_bytes(size) as usize;

        // get string bytes
        let array_bytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

        // convert to string
        let newString = String::from_utf8(array_bytes.to_vec())?;

        Ok(Some(Box::new(newString)))
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        if str_size >= std::u32::MAX as u64 { 
            Err(NoProtoError::new("String too large!"))
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

/*
impl NoProtoValue<'a> for &str {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        ""
    }

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::UTF8String as i64, "string".to_owned()) }

    fn buffer_get(&self, address: u32, kind: &NoProtoPointerKinds, schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }
        
        // get size of string
        let mut size: [u8; 4] = [0; 4];
        let memory = buffer.try_borrow()?;
        size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
        let str_size = u32::from_le_bytes(size) as usize;

        // get string bytes
        let array_bytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

        // convert to string
        let newString = String::from_utf8(array_bytes.to_vec())?;

        self = &'static newString.as_str();

        Ok(Some(Box::new(newString.as_str())))
    }

    fn buffer_set(&mut self, address: u32, kind: &NoProtoPointerKinds, schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let bytes = value.as_bytes();
        let str_size = bytes.len() as u64;

        if str_size >= std::u32::MAX as u64 { 
            Err(NoProtoError::new("String too large!"))
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

                } else { // not enough space or space has not been allocted yet
                    

                    // first 4 bytes are string length
                    addr = memory.malloc((str_size as u32).to_le_bytes().to_vec())? as usize;

                    // then string content
                    memory.malloc(bytes.to_vec())?;
                }
            }

            Ok(kind.set_value_address(address, addr as u32, buffer)?)
        }
    }
}*/