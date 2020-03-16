use crate::schema::NoProtoSchema;
use crate::error::NoProtoError;
use crate::memory::NoProtoMemory;
use std::{cell::RefCell, rc::Rc};
use crate::{schema::NoProtoTypeKeys, pointer::NoProtoValue};
use super::NoProtoPointerKinds;



impl<'a> NoProtoValue<'a> for i8 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "int8" == type_str || "i8" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Int8 as i64, "int8".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Int8 as i64, "int8".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(i8::from_le_bytes([x])))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }
                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;
                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
    }
}

impl<'a> NoProtoValue<'a> for i16 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "int16" == type_str || "i16" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Int16 as i64, "int16".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Int16 as i64, "int16".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_2_bytes(addr) {
            Some(x) => {
                Some(Box::new(i16::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }   

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}

impl<'a> NoProtoValue<'a> for i32 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "int32" == type_str || "i32" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Int32 as i64, "int32".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Int32 as i64, "int32".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(i32::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}

impl<'a> NoProtoValue<'a> for i64 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "int64" == type_str || "i64" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Int64 as i64, "int64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Int64 as i64, "int64".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(i64::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}

impl<'a> NoProtoValue<'a> for u8 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "uint8" == type_str || "u8" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Uint8 as i64, "uint8".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Uint8 as i64, "uint8".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(u8::from_le_bytes([x])))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }
                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }

        
    }
}

impl<'a> NoProtoValue<'a> for u16 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "uint16" == type_str || "u16" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Uint16 as i64, "uint16".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Uint16 as i64, "uint16".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_2_bytes(addr) {
            Some(x) => {
                Some(Box::new(u16::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }

        }
    }
}

impl<'a> NoProtoValue<'a> for u32 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "uint32" == type_str || "u32" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Uint32 as i64, "uint32".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Uint32 as i64, "uint32".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(u32::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}


impl<'a> NoProtoValue<'a> for u64 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0
    }

    fn is_type( type_str: &str) -> bool {
        "uint64" == type_str || "u64" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Uint64 as i64, "uint64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Uint64 as i64, "uint64".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(u64::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}

impl<'a> NoProtoValue<'a> for f32 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0.0
    }

    fn is_type( type_str: &str) -> bool {
        "float" == type_str || "f32" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Float as i64, "float".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Float as i64, "float".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_4_bytes(addr) {
            Some(x) => {
                Some(Box::new(f32::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
    }
}

impl<'a> NoProtoValue<'a> for f64 {

    fn new<T: NoProtoValue<'a> + Default>() -> Self {
        0.0
    }

    fn is_type( type_str: &str) -> bool {
        "double" == type_str || "f64" == type_str
    }

    fn type_idx() -> (i64, String) { (NoProtoTypeKeys::Double as i64, "double".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NoProtoTypeKeys::Double as i64, "double".to_owned()) }

    fn buffer_get(_address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(f64::from_le_bytes(x)))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NoProtoPointerKinds, _schema: &NoProtoSchema, buffer: Rc<RefCell<NoProtoMemory>>, value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }
                return Ok(*kind);
            } else { // new value

                let bytes = value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;
                return Ok(memory.set_value_address(address, addr as u32, kind)?);
            }
        }
        
    }
}