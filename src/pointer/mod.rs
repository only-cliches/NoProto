//! All values in NoProto buffers are accessed by a pointer
//! 
//! NoProto Pointers are the primary abstraction to read, update or delete values in a buffer.
//! Pointers should *never* be created directly, instead the various methods provided by the library to access
//! the internals of the buffer should be used.
//! 
//! Once you have a pointer you can read it's contents if it's a scalar value (`.to_string()`, `.to_int8()`, etc) or convert it to a collection type (`.as_table()`, `.as_map()`, etc).
//! When you attempt to read, update, or convert a pointer the schema is checked for that pointer location.  If the schema conflicts with what you're attempting to do, for example
//! if you call `to_string()` but the schema for that location is of `int8` type, the operation will fail.  As a result, you should be careful to make sure your reads and updates to the 
//! buffer line up with the schema you provided.

pub mod types;
pub mod string;
pub mod bytes;
pub mod any;

use crate::memory::NoProtoMemory;
use crate::NoProtoError;
use crate::{schema::NoProtoSchemaKinds, schema::NoProtoSchema, collection::{map::NoProtoMap, list::NoProtoList, table::NoProtoTable, tuple::NoProtoTuple}};
use std::rc::Rc;
use std::cell::RefCell;



/*
#[doc(hidden)]
pub enum TypeReq {
    Read, Write, Collection
}

fn type_error(req: TypeReq, kind: &str, schema: &NoProtoSchema) -> NoProtoError {
    match req {
        TypeReq::Collection => {
            return NoProtoError::new(format!("TypeError: Attempted to get collection of type ({}) from pointer of type ({})!", kind, &schema.get_type_str()).as_str());
        },
        TypeReq::Read => {
            return NoProtoError::new(format!("TypeError: Attempted to read value of type ({}) from pointer of type ({})!", kind, &schema.get_type_str()).as_str());
        },
        TypeReq::Write => {
            return NoProtoError::new(format!("TypeError: Attempted to write value of type ({}) to pointer of type ({})!", kind, &schema.get_type_str()).as_str());
        }
    }
}*/


#[doc(hidden)]
pub enum NoProtoPointerKinds {
    None,
    // scalar / collection
    Standard  { value: u32 }, // 4 bytes [4]

    // collection items
    MapItem   { value: u32, next: u32, key: u32 }, // 12 bytes [4, 4, 4]
    TableItem { value: u32, next: u32, i: u8    }, // 9  bytes [4, 4, 1]
    ListItem  { value: u32, next: u32, i: u16   },  // 10 bytes [4, 4, 2]
}

impl NoProtoPointerKinds {
    fn get_value(&self) -> u32 {
        match self {
            NoProtoPointerKinds::Standard  { value } =>                      { *value },
            NoProtoPointerKinds::MapItem   { value, key: _,  next: _ } =>    { *value },
            NoProtoPointerKinds::TableItem { value, i: _,    next: _ } =>    { *value },
            NoProtoPointerKinds::ListItem  { value, i:_ ,    next: _ } =>    { *value }
        }
    }
    fn set_value_address(&self, address: u32, val: u32, buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let mut memory = buffer.try_borrow_mut()?;

        let addr_bytes = val.to_le_bytes();
    
        for x in 0..addr_bytes.len() {
            memory.bytes[(address + x as u32) as usize] = addr_bytes[x as usize];
        }

        Ok(match self {
            NoProtoPointerKinds::Standard { value: _ } => {
                NoProtoPointerKinds::Standard { value: val}
            },
            NoProtoPointerKinds::MapItem { value: _, key,  next  } => {
                NoProtoPointerKinds::MapItem { value: val, key: *key, next: *next }
            },
            NoProtoPointerKinds::TableItem { value: _, i, next  } => {
                NoProtoPointerKinds::TableItem { value: val, i: *i, next: *next }
            },
            NoProtoPointerKinds::ListItem { value: _, i, next  } => {
                NoProtoPointerKinds::ListItem { value: val, i: *i, next: *next }
            }
        })
    }
}


pub trait NoProtoValue {
    fn new<T: NoProtoValue + Default>() -> Self;
    fn is_type(&self, type_str: &str) -> bool { false }
    fn type_idx() -> (i64, &'static str) { (-1, "null") }
    fn self_type_idx(&self) -> (i64, &'static str) { (-1, "null") }
    fn buffer_read<T: NoProtoValue + Default>(&self, address: u32, kind: &NoProtoPointerKinds, schemaData: (i64, &'static str), buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<T>, NoProtoError>;
    fn buffer_write(&mut self, address: u32, kind: &NoProtoPointerKinds, schemaData: (i64, &'static str), buffer: Rc<RefCell<NoProtoMemory>>, value: Self) -> std::result::Result<NoProtoPointerKinds, NoProtoError>;
}


impl NoProtoValue for Vec<u8> {

}

impl NoProtoValue for i8 {

}

impl NoProtoValue for i16 {

}

impl NoProtoValue for i32 {

}

impl NoProtoValue for i64 {

}

/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
pub struct NoProtoPointer<'a, T: NoProtoValue + Default> {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    schema: &'a NoProtoSchema<T>,
    cached: bool,
    value: T,
    valueIdx: (i64, &'static str)
}

impl<'a, T: NoProtoValue + Default> NoProtoPointer<'a, T> {

    #[doc(hidden)]
    pub fn new_example_ptr(schema: &'a NoProtoSchema<T>, value: T) -> Self {

        NoProtoPointer {
            address: 0,
            kind: NoProtoPointerKinds::Standard { value: 0 },
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![0, 0, 0, 0] })),
            schema: schema,
            cached: false,
            valueIdx: T::type_idx(),
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn new(address: u32, schema: &'a NoProtoSchema<T>, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

        let thisKind = match *schema.kind {
            NoProtoSchemaKinds::None => {
                NoProtoPointerKinds::None
            },
            NoProtoSchemaKinds::Scalar => {
                NoProtoPointerKinds::Standard { value: 0 }
            },
            NoProtoSchemaKinds::List { of } => {
                NoProtoPointerKinds::Standard { value: 0 }
            },
            NoProtoSchemaKinds::Table { columns  } => {
                NoProtoPointerKinds::Standard { value: 0 }
            },
            NoProtoSchemaKinds::Map { value } => {
                NoProtoPointerKinds::Standard { value: 0 }
            },
            NoProtoSchemaKinds::Enum { choices } => {
                NoProtoPointerKinds::Standard { value: 0 }
            },
            NoProtoSchemaKinds::Tuple { values } => {
                NoProtoPointerKinds::Standard { value: 0 }
            }
        };

        NoProtoPointer {
            address: address,
            kind: thisKind,
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![0, 0, 0, 0] })),
            schema: schema,
            cached: false,
            value: T::default(),
            valueIdx: T::type_idx()
        }
    }


    pub fn get(&mut self) -> std::result::Result<Option<&T>, NoProtoError> {

        if self.cached {
            return Ok(Some(&self.value));
        }

        let value = self.value.buffer_read::<T>(self.address, &self.kind, self.valueIdx, Rc::clone(&self.memory))?;
        
        Ok(match value {
            Some (x) => {
                self.value = x;
                self.cached = true;
                Some(&self.value)
            },
            None => None
        })
    }

    pub fn set(&mut self, value: T) -> std::result::Result<(), NoProtoError> {
        self.value = value;
        self.kind = self.value.buffer_write(self.address, &self.kind, self.valueIdx, Rc::clone(&self.memory), value)?;
        self.cached = true;
        Ok(())
    }

    pub fn into(self) -> std::result::Result<Option<T>, NoProtoError> {
        if self.cached {
            return Ok(Some(self.value));
        }

        let result_get = self.get()?;

        Ok(match result_get {
            Some(x) => {
                Some(self.value)
            },
            None => None
        })
    }
/*
    #[doc(hidden)]
    pub fn new_standard_ptr(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Self, NoProtoError> {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        {
            let b_bytes = &memory.try_borrow()?.bytes;
            value.copy_from_slice(&b_bytes[addr..(addr+4)]);
        }

        Ok(NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::Standard { value: u32::from_le_bytes(value) },
            memory: memory,
            schema: schema
        })
    }

    #[doc(hidden)]
    pub fn new_table_item_ptr(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Self, NoProtoError> {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        let mut next: [u8; 4] = [0; 4];
        let index: u8;

        {
            let b_bytes = &memory.try_borrow()?.bytes;
            value.copy_from_slice(&b_bytes[addr..(addr + 4)]);
            next.copy_from_slice(&b_bytes[(addr + 4)..(addr + 8)]);
            index = b_bytes[addr + 8];
        }

        Ok(NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::TableItem { 
                value: u32::from_le_bytes(value),
                next: u32::from_le_bytes(next),
                i: index
            },
            memory: memory,
            schema: schema
        })
    }

    #[doc(hidden)]
    pub fn new_map_item_ptr(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Self, NoProtoError> {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        let mut next: [u8; 4] = [0; 4];
        let mut key: [u8; 4] = [0; 4];

        {
            let b_bytes = &memory.try_borrow()?.bytes;
            value.copy_from_slice(&b_bytes[addr..(addr + 4)]);
            next.copy_from_slice(&b_bytes[(addr + 4)..(addr + 8)]);
            key.copy_from_slice(&b_bytes[(addr + 8)..(addr + 12)]);
        }

        Ok(NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::MapItem { 
                value: u32::from_le_bytes(value),
                next: u32::from_le_bytes(next),
                key: u32::from_le_bytes(key)
            },
            memory: memory,
            schema: schema
        })
    }

    #[doc(hidden)]
    pub fn new_list_item_ptr(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Self, NoProtoError> {

        let addr = address as usize;
        let mut value: [u8; 4] = [0; 4];
        let mut next: [u8; 4] = [0; 4];
        let mut index: [u8; 2] = [0; 2];

        {
            let b_bytes = &memory.try_borrow()?.bytes;
            value.copy_from_slice(&b_bytes[addr..(addr + 4)]);
            next.copy_from_slice(&b_bytes[(addr + 4)..(addr + 8)]);
            index.copy_from_slice(&b_bytes[(addr + 8)..(addr + 10)]);
        }

        Ok(NoProtoPointer {
            address: address,
            kind: NoProtoPointerKinds::ListItem { 
                value: u32::from_le_bytes(value),
                next: u32::from_le_bytes(next),
                i: u16::from_le_bytes(index)
            },
            memory: memory,
            schema: schema
        })
    }
*/
    pub fn has_value(self) -> bool {
        if self.address == 0 { return false; } else { return true; }
    }

    pub fn clear(&mut self) -> std::result::Result<(), NoProtoError> {
        self.kind.set_value_address(self.address, 0, Rc::clone(&self.memory));
        Ok(())
    }

/*

    pub fn as_table(&mut self) -> std::result::Result<NoProtoTable, NoProtoError> {

        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Table { columns } => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                let mut head: [u8; 4] = [0; 4];

                // no table here, make one
                if addr == 0 {
                    let mut memory = self.memory.try_borrow_mut()?;
                    addr = memory.malloc([0 as u8; 4].to_vec())?; // stores HEAD for table
                    set_addr = true;
                }

                if set_addr { // new head, empty value
                    self.set_value_address(addr)?;
                } else { // existing head, read value
                    let b_bytes = &self.memory.try_borrow()?.bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                }

                Ok(NoProtoTable::new(addr, u32::from_le_bytes(head), Rc::clone(&self.memory), &columns))
            },
            _ => {
                Err(type_error(TypeReq::Collection, "table", &model))
            }
        }
    }


    pub fn as_list(&mut self) -> std::result::Result<NoProtoList, NoProtoError> {
        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::List { of } => {
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                let mut head: [u8; 4] = [0; 4];
                let mut tail: [u8; 4] = [0; 4];

                // no list here, make one
                if addr == 0 {
                    let mut memory = self.memory.try_borrow_mut()?;

                    addr = memory.malloc([0 as u8; 8].to_vec())?; // stores HEAD & TAIL for list
                    set_addr = true;
                }

                if set_addr { // new head, empty value
                    self.set_value_address(addr)?;
                } else { // existing head, read values
                    let b_bytes = &self.memory.try_borrow()?.bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                    tail.copy_from_slice(&b_bytes[(a+4)..(a+8)]);
                }

                Ok(NoProtoList::new(addr, u32::from_le_bytes(head), u32::from_le_bytes(tail), Rc::clone(&self.memory), &of))
            }
            _ => {
                Err(type_error(TypeReq::Collection, "list", &model))
            }
        }
    }

    pub fn as_tuple(&mut self) -> std::result::Result<NoProtoTuple, NoProtoError> {

        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Tuple { values } => {
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                let mut head: [u8; 4] = [0; 4];

                // no tuple here, make one
                if addr == 0 {
                    let mut memory = self.memory.try_borrow_mut()?;

                    let value_num = values.len();

                    let mut value_bytes: Vec<u8> = Vec::new();

                    // there is one u32 address for each value
                    for _x in 0..(value_num * 4) {
                        value_bytes.push(0);
                    }

                    addr = memory.malloc(value_bytes)?; // stores HEAD for tuple
                    set_addr = true;
                }

                if set_addr { // new head, empty value
                    self.set_value_address(addr)?;
                } else { // existing head, read value
                    let b_bytes = &self.memory.try_borrow()?.bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                }

                Ok(NoProtoTuple::new(addr, u32::from_le_bytes(head), Rc::clone(&self.memory), &values))
            }
            _ => {
                Err(type_error(TypeReq::Collection, "tuple", &model))
            }
        }
    }


    pub fn as_map(&mut self) -> std::result::Result<NoProtoMap, NoProtoError> {
        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Map { value } => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                let mut head: [u8; 4] = [0; 4];

                // no map here, make one
                if addr == 0 {
                    let mut memory = self.memory.try_borrow_mut()?;

                    addr = memory.malloc([0 as u8; 4].to_vec())?; // stores HEAD for map
                    set_addr = true;
                }

                if set_addr { // new head, empty value
                    self.set_value_address(addr)?;
                } else { // existing head, read value
                    let b_bytes = &self.memory.try_borrow()?.bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                }

                Ok(NoProtoMap::new(addr, u32::from_le_bytes(head), Rc::clone(&self.memory), value))
            }
            _ => {
                Err(type_error(TypeReq::Collection, "map", &model))
            }
        }
    }
 
*/


    pub fn to_dec64(&self) -> std::result::Result<Option<NoProtoDec>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Dec64 => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        let mem = self.memory.try_borrow()?;
                        let addr = self.get_value_address();
                        Some(NoProtoDec::new(i64::from_le_bytes(x), u8::from_le_bytes([mem.bytes[(addr + 8) as usize]])))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "dec64", &model))
            }
        }
    }

    pub fn set_dec64(&mut self, dec64: NoProtoDec) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Dec64 => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = dec64.num.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                        let bytes2 = dec64.scale.to_le_bytes();
                        memory.bytes[(addr + 8) as usize] = bytes2[0];

                    } else { // new value

                        let bytes = dec64.num.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                        memory.malloc(dec64.scale.to_le_bytes().to_vec())?;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "dec64", &model))
            }
        }
    }

    /// Allows you to get the allowed range for the given pointer.  This will work as long as the pointer is one of the integer (intX) or unsigned integer (uintX) types.
    /// 
    /// If the pointer is not an integer (intX) or unsigned integer (uintX) type, this returns two zeros (0,0).
    /// 
    /// # Example: 
    /// Assuming `uint8_ptr` is a `NoProtoPointer` of type `uint8`.
    /// ```
    /// # use json::*;
    /// # use no_proto::error::NoProtoError;
    /// # use no_proto::schema::NoProtoSchema;
    /// # use no_proto::pointer::NoProtoPointer;
    /// # let schema = NoProtoSchema::init().from_json(object!{"type" => "uint8"}).unwrap();
    /// # let mut uint8_ptr = NoProtoPointer::new_example_ptr(&schema);
    /// assert_eq!(uint8_ptr.get_integer_range(), (0, 255));
    /// # Ok::<(), NoProtoError>(())
    /// ```
    pub fn get_integer_range(&self) -> (i128, i128) {
        let model = self.schema;
        match *model.kind {
            NoProtoSchemaKinds::Int8 => { ((2i128.pow(7) * -1), 2i128.pow(7)) },
            NoProtoSchemaKinds::Int16 => { ((2i128.pow(15) * -1), 2i128.pow(15)) },
            NoProtoSchemaKinds::Int32 => { ((2i128.pow(31) * -1), 2i128.pow(31)) },
            NoProtoSchemaKinds::Int64 => { ((2i128.pow(63) * -1), 2i128.pow(63)) },
            NoProtoSchemaKinds::Uint8 => { (0, 2i128.pow(8) - 1) },
            NoProtoSchemaKinds::Uint16 => { (0, 2i128.pow(16) - 1) },
            NoProtoSchemaKinds::Uint32 => { (0, 2i128.pow(32) - 1) },
            NoProtoSchemaKinds::Uint64 => { (0, 2i128.pow(64) - 1) }
            _ => { (0, 0)}
        }
    }

    pub fn to_int8(&self) -> std::result::Result<Option<i8>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int8 => {
                Ok(match self.get_1_byte()? {
                    Some(x) => {
                        Some(i8::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "int8", &model))
            }
        }
    }

    pub fn set_int8(&mut self, int8: i8) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int8 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = int8.to_le_bytes();
    
                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
    
                    } else { // new value
       
                        let bytes = int8.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "int8", &model))
            }
        }
    }

    pub fn to_int16(&self) -> std::result::Result<Option<i16>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int16 => {
                Ok(match self.get_2_bytes()? {
                    Some(x) => {
                        Some(i16::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "int16", &model))
            }
        }
        
    }

    pub fn set_int16(&mut self, int16: i16) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int16 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = int16.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value

                        let bytes = int16.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "int16", &model))
            }
        }
    }

    pub fn to_int32(&self) -> std::result::Result<Option<i32>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int32 => {
                Ok(match self.get_4_bytes()? {
                    Some(x) => {
                        Some(i32::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "int32", &model))
            }
        }
    }

    pub fn set_int32(&mut self, int32: i32) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int32 => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = int32.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = int32.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "int32", &model))
            }
        }
    }

    pub fn to_int64(&self) -> std::result::Result<Option<i64>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int64 => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        Some(i64::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "int64", &model))
            }
        }
    }

    pub fn set_int64(&mut self, int64: i64) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Int64 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = int64.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = int64.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "int64", &model))
            }
        }
    }

    pub fn to_uint8(&self) -> std::result::Result<Option<u8>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint8 => {
                Ok(match self.get_1_byte()? {
                    Some(x) => {
                        Some(u8::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "uint8", &model))
            }
        }
    }

    pub fn set_uint8(&mut self, uint8: u8) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint8 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = uint8.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint8.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "uint8", &model))
            }
        }
    }

    pub fn to_uint16(&self) -> std::result::Result<Option<u16>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint16 => {
                Ok(match self.get_2_bytes()? {
                    Some(x) => {
                        Some(u16::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "uint16", &model))
            }
        }
    }

    pub fn set_uint16(&mut self, uint16: u16) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint16 => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = uint16.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint16.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "uint16", &model))
            }
        }
    }

    pub fn to_uint32(&self) -> std::result::Result<Option<u32>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint32 => {
                Ok(match self.get_4_bytes()? {
                    Some(x) => {
                        Some(u32::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "uint32", &model))
            }
        }
    }

    pub fn set_uint32(&mut self, uint32: u32) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint32 => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = uint32.to_le_bytes();
    
                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
    
                    } else { // new value
       
                        let bytes = uint32.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "uint32", &model))
            }
        }
    }

    pub fn to_uint64(&self) -> std::result::Result<Option<u64>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint64 => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        Some(u64::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "uint64", &model))
            }
        }
    }

    pub fn set_uint64(&mut self, uint64: u64) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uint64 => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;
                    
                    if addr != 0 { // existing value, replace
                        let bytes = uint64.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uint64.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "uint64", &model))
            }
        }
    }

    pub fn to_float(&self) -> std::result::Result<Option<f32>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Float => {
                Ok(match self.get_4_bytes()? {
                    Some(x) => {
                        Some(f32::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "float", &model))
            }
        }
    }

    pub fn set_float(&mut self, float: f32) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Float => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = float.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = float.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }   

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "float", &model))
            }
        }
    }

    pub fn to_double(&self) -> std::result::Result<Option<f64>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Double => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        Some(f64::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "double", &model))
            }
        }
    }

    pub fn set_double(&mut self, double: f64) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Double => {
                

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = double.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = double.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "double", &model))
            }
        }
    }

    pub fn to_option(&self) -> std::result::Result<Option<String>, NoProtoError> {

        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Enum { choices } => {

                Ok(match self.get_1_byte()? {
                    Some(x) => {
                        let value_num = u8::from_le_bytes(x) as usize;

                        if value_num > choices.len() {
                            None
                        } else {
                            Some(choices[value_num].clone())
                        }
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "option", &model))
            }
        }
    }

    pub fn set_option(&mut self, option: String) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match &*model.kind {
            NoProtoSchemaKinds::Enum { choices } => {

                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;

                    for opt in choices {
                        if option == opt.to_string() {
                            value_num = ct as i32;
                        }
                        ct += 1;
                    };

                    if value_num == -1 {
                        return Err(NoProtoError::new("Option not found, cannot set uknown option!"));
                    }
                }

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    let bytes = (value_num as u8).to_le_bytes();

                    if addr != 0 { // existing value, replace

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
                
            },
            _ => {
                Err(type_error(TypeReq::Write, "option", &model))
            }
        }
    }

    pub fn to_boolean(&self) -> std::result::Result<Option<bool>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Boolean => {
                Ok(match self.get_1_byte()? {
                    Some(x) => {
                        Some(if x[0] == 1 { true } else { false })
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "bool", &model))
            }
        }
    }

    pub fn set_boolean(&mut self, boolean: bool) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Boolean => {
                
                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = if boolean == true {
                            [1] as [u8; 1]
                        } else {
                            [0] as [u8; 1]
                        };

                        // overwrite existing values in buffer
                        memory.bytes[addr as usize] = bytes[0];

                    } else { // new value
    
                        let bytes = if boolean == true {
                            [1] as [u8; 1]
                        } else {
                            [0] as [u8; 1]
                        };

                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "bool", &model))
            }
        }
    }

    pub fn to_geo(&self) -> std::result::Result<Option<NoProtoGeo>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Geo16 => {
                Ok(match self.get_16_bytes()? {
                    Some(x) => {
                        let mut bytes_lat: [u8; 8] = [0; 8];
                        let mut bytes_lon: [u8; 8] = [0; 8];
        
                        for i in 0..x.len() {
                            if i < 8 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 8] = x[i as usize];
                            }
                        }
        
                        Some(NoProtoGeo { lat: f64::from_le_bytes(bytes_lat), lon: f64::from_le_bytes(bytes_lon)})
                    },
                    None => None
                })              
            },
            NoProtoSchemaKinds::Geo8 => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        let mut bytes_lat: [u8; 4] = [0; 4];
                        let mut bytes_lon: [u8; 4] = [0; 4];
    
                        for i in 0..x.len() {
                            if i < 4 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 4] = x[i as usize];
                            }
                        }
    
                        let lat = i32::from_le_bytes(bytes_lat) as f64;
                        let lon = i32::from_le_bytes(bytes_lon) as f64;
    
                        let dev = 10000000f64;
    
                        Some(NoProtoGeo { lat: lat / dev, lon: lon / dev})
                    },
                    None => None
                })

                 
            },
            NoProtoSchemaKinds::Geo4 => {
                Ok(match self.get_4_bytes()? {
                    Some(x) => {
                        let mut bytes_lat: [u8; 2] = [0; 2];
                        let mut bytes_lon: [u8; 2] = [0; 2];

                        for i in 0..x.len() {
                            if i < 2 {
                                bytes_lat[i as usize] = x[i as usize];
                            } else {
                                bytes_lon[i as usize - 2] = x[i as usize];
                            }
                        }

                        let lat = i16::from_le_bytes(bytes_lat) as f64;
                        let lon = i16::from_le_bytes(bytes_lon) as f64;

                        let dev = 100f64;

                        Some(NoProtoGeo { lat: lat / dev, lon: lon / dev})
                    },
                    None => None
                })             
            },
            _ => {
                Err(type_error(TypeReq::Read, "geo4, geo8 or geo16", &model))
            }
        }
    }

    pub fn set_geo(&mut self, geo: NoProtoGeo) -> std::result::Result<(), NoProtoError> {

        let mut addr = self.get_value_address();
        let mut set_addr = false;

        {

            let mut memory = self.memory.try_borrow_mut()?;

            let model = self.schema;

            let value_bytes_size = match *model.kind {
                NoProtoSchemaKinds::Geo16 => { 16 },
                NoProtoSchemaKinds::Geo8 => { 8 },
                NoProtoSchemaKinds::Geo4 => { 4 },
                _ => { 0 }
            };

            if value_bytes_size == 0 {
                return Err(type_error(TypeReq::Write, "geo4, geo8 or geo16", &model));
            }

            let half_value_bytes = value_bytes_size / 2;

            // convert input values into bytes
            let value_bytes = match *model.kind {
                NoProtoSchemaKinds::Geo16 => {
                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = geo.lat.to_le_bytes();
                    let lon_bytes = geo.lon.to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                NoProtoSchemaKinds::Geo8 => {
                    let dev = 10000000f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((geo.lat * dev) as i32).to_le_bytes();
                    let lon_bytes = ((geo.lon * dev) as i32).to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                NoProtoSchemaKinds::Geo4 => {
                    let dev = 100f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((geo.lat * dev) as i16).to_le_bytes();
                    let lon_bytes = ((geo.lon * dev) as i16).to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                _ => {
                    [0; 16]
                }
            };

            if addr != 0 { // existing value, replace

                // overwrite existing values in buffer
                for x in 0..value_bytes.len() {
                    if x < value_bytes_size {
                        memory.bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                    }
                }

            } else { // new value

                addr = match *model.kind {
                    NoProtoSchemaKinds::Geo16 => { memory.malloc([0; 16].to_vec())? },
                    NoProtoSchemaKinds::Geo8 => { memory.malloc([0; 8].to_vec())? },
                    NoProtoSchemaKinds::Geo4 => { memory.malloc([0; 4].to_vec())? },
                    _ => { 0 }
                };

                set_addr = true;

                // set values in buffer
                for x in 0..value_bytes.len() {
                    if x < value_bytes_size {
                        memory.bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                    }
                }
            }
        }

        if set_addr { self.set_value_address(addr)?; };

        Ok(())
    }

    pub fn to_uuid(&self) -> std::result::Result<Option<NoProtoUUID>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uuid => {
                Ok(match self.get_16_bytes()? {
                    Some(x) => {
                        Some(NoProtoUUID { value: x})
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "uuid", &model))
            }
        }
    }

    pub fn set_uuid(&mut self, uuid: NoProtoUUID) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Uuid => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = uuid.value;

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = uuid.value;
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "uuid", &model))
            }
        }
    }

    pub fn to_time_id(&self) -> std::result::Result<Option<NoProtoTimeID>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Tid => {
                Ok(match self.get_16_bytes()? {
                    Some(x) => {
                        let mut id_bytes: [u8; 8] = [0; 8];
                        id_bytes.copy_from_slice(&x[0..8]);

                        let mut time_bytes: [u8; 8] = [0; 8];
                        time_bytes.copy_from_slice(&x[8..16]);

                        Some(NoProtoTimeID {
                            id: id_bytes,
                            time: u64::from_le_bytes(time_bytes)
                        })
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "tid", &model))
            }
        }
    }

    pub fn set_time_id(&mut self, time_id: NoProtoTimeID) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Tid => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace

                        let time_bytes = time_id.time.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..16 {
                            if x < 8 {
                                memory.bytes[(addr + x as u32) as usize] = time_id.id[x as usize];
                            } else {
                                memory.bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                            }
                        }

                    } else { // new value
    
                        let mut bytes: [u8; 16] = [0; 16];
                        let time_bytes = time_id.time.to_le_bytes();

                        for x in 0..bytes.len() {
                            if x < 8 {
                                bytes[(addr + x as u32) as usize] = time_id.id[x as usize];
                            } else {
                                bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                            }
                        }

                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "tid", &model))
            }
        }
    }

    pub fn to_date(&self) -> std::result::Result<Option<u64>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Date => {
                Ok(match self.get_8_bytes()? {
                    Some(x) => {
                        Some(u64::from_le_bytes(x))
                    },
                    None => None
                })
            },
            _ => {
                Err(type_error(TypeReq::Read, "date", &model))
            }
        }
    }

    pub fn set_date(&mut self, date: u64) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Date => {

                let mut addr = self.get_value_address();
                let mut set_addr = false;

                {
                    let mut memory = self.memory.try_borrow_mut()?;

                    if addr != 0 { // existing value, replace
                        let bytes = date.to_le_bytes();

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }

                    } else { // new value
    
                        let bytes = date.to_le_bytes();
                        addr = memory.malloc(bytes.to_vec())?;
                        set_addr = true;
                    }                    
                }

                if set_addr { self.set_value_address(addr)?; };

                Ok(())
            },
            _ => {
                Err(type_error(TypeReq::Write, "date", &model))
            }
        }
    }
}

// Pointer -> String
impl<'a> From<NoProtoPointer<'a>> for String {
    fn from(ptr: NoProtoPointer) -> String {
        match ptr.to_string() {
            Ok(x) => x.unwrap(),
            Err(e) => panic!(e)
        }
    }
}

/*
// unsigned integer size:        0 to (2^i) -1
//   signed integer size: -2^(i-1) to  2^(i-1) 
pub enum NoProtoDataType {
    none,
    /*table {
        head: u32
    },
    map {
        head: u32
    },
    list {
        head: u32,
        tail: u32,
        size: u16
    },
    tuple {
        head: u32
    },*/
    utf8_string { size: u32, value: String },
    bytes { size: u32, value: Vec<u8> },
    int8 { value: i8 },
    int16 { value: i16 },
    int32 { value: i32 },
    int64 { value: i64 }, 
    uint8 { value: u8 },
    uint16 { value: u16 },
    uint32 { value: u32 },
    uint64 { value: u64 },
    float { value: f32 }, // -3.4E+38 to +3.4E+38
    double { value: f64 }, // -1.7E+308 to +1.7E+308
    option { value: u8 }, // enum
    dec32 { value: i32, expo: i8},
    dec64 { value: i64, exp: i8},
    boolean { value: bool },
    geo_16 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_8 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_4 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: String }, // 16 bytes 21,267,647,932,558,653,966,460,912,964,485,513,216 possibilities (255^15 * 16) or over 2 quadrillion times more possibilites than stars in the universe
    time_id { id: String, time: u64 }, // 8 + 8 bytes
    date { value: u64 } // 8 bytes  
}*/

// Pointer -> String
/*impl From<&NoProtoPointer> for std::result::Result<String> {
    fn from(ptr: &NoProtoPointer) -> std::result::Result<String> {
        ptr.to_string()
    }
}*/

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

// cast Pointer => std::result::Result<i64>
impl From<&NoProtoValue> for std::result::Result<i64> {
    fn from(ptr: &NoProtoValue) -> std::result::Result<i64> {
        match ptr.value {
            NoProtoValue::int64 { value } => {
                Some(value)
            }
            _ => None
        }
    }
}*/