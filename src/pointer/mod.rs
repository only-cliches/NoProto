//! All values in NoProto buffers are accessed and modified through NoProtoPointers
//! 
//! NoProto Pointers are the primary abstraction to read, update or delete values in a buffer.
//! Pointers should *never* be created directly, instead the various methods provided by the library to access
//! the internals of the buffer should be used.
//! 
//! Once you have a pointer you can read it's contents if it's a scalar value (`.to_string()`, `.to_int8()`, etc) or convert it to a collection type (`.as_table()`, `.as_map()`, etc).
//! When you attempt to read, update, or convert a pointer the schema is checked for that pointer location.  If the schema conflicts with what you're attempting to do, for example
//! if you call `to_string()` but the schema for that location is of `int8` type, the operation will fail.  As a result, you should be careful to make sure your reads and updates to the 
//! buffer line up with the schema you provided.

pub mod misc;
pub mod string;
pub mod bytes;
pub mod any;
pub mod numbers;

use crate::collection::table::NoProtoTable;
use crate::memory::NoProtoMemory;
use crate::NoProtoError;
use crate::{schema::{NoProtoSchemaKinds, NoProtoSchema}};
use std::rc::Rc;
use std::cell::RefCell;
use json::JsonValue;



/*
#[doc(hidden)]
pub enum TypeReq {
    Read, Write, Collection
}

fn type_error(req: TypeReq, kind: &str, schema: &NoProtoPointerKinds) -> NoProtoError {
    match req {
        TypeReq::Collection => {
            return NoProtoError::new(format!("TypeError: Attempted to get collection of type ({}) from pointer of type ({})!", kind, schema.kind).as_str());
        },
        TypeReq::Read => {
            return NoProtoError::new(format!("TypeError: Attempted to read value of type ({}) from pointer of type ({})!", kind, schema.kind).as_str());
        },
        TypeReq::Write => {
            return NoProtoError::new(format!("TypeError: Attempted to write value of type ({}) to pointer of type ({})!", kind, schema.kind).as_str());
        }
    }
}
*/

#[doc(hidden)]
#[derive(Debug, Clone, Copy)]
pub enum NoProtoPointerKinds {
    None,
    // scalar / collection
    Standard  { value: u32 }, // 4 bytes [4]

    // collection items
    MapItem   { value: u32, next: u32, key: u32 },  // 12 bytes [4, 4, 4]
    TableItem { value: u32, next: u32, i: u8    },  // 9  bytes [4, 4, 1]
    ListItem  { value: u32, next: u32, i: u16   },  // 10 bytes [4, 4, 2]
}

impl NoProtoPointerKinds {
    pub fn get_value(&self) -> u32 {
        match self {
            NoProtoPointerKinds::None                                                => { 0 },
            NoProtoPointerKinds::Standard  { value } =>                      { *value },
            NoProtoPointerKinds::MapItem   { value, key: _,  next: _ } =>    { *value },
            NoProtoPointerKinds::TableItem { value, i: _,    next: _ } =>    { *value },
            NoProtoPointerKinds::ListItem  { value, i:_ ,    next: _ } =>    { *value }
        }
    }
}

pub trait NoProtoValue<'a> {
    fn new<T: NoProtoValue<'a> + Default>() -> Self;
    fn is_type(_type_str: &str) -> bool { false }
    fn type_idx() -> (i64, String) { (-1, "null".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (-1, "null".to_owned()) }
    fn schema_state(_type_string: &str, _json_schema: &JsonValue) -> std::result::Result<i64, NoProtoError> { Ok(0) }
    fn buffer_get(_address: u32, _kind: &NoProtoPointerKinds, _schema: &'a NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        Err(NoProtoError::new(format!("This type ({}) doesn't support .get()!", Self::type_idx().1).as_str()))
    }
    fn buffer_set(_address: u32, _kind: &NoProtoPointerKinds, _schema: &'a NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>, _value: Box<&Self>) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {
        Err(NoProtoError::new(format!("This type ({}) doesn't support .set()!", Self::type_idx().1).as_str()))
    }
    fn buffer_into(_address: u32, _kind: NoProtoPointerKinds, _schema: &'a NoProtoSchema, _buffer: Rc<RefCell<NoProtoMemory>>) -> std::result::Result<Option<Box<Self>>, NoProtoError> {
        Err(NoProtoError::new("This type doesn't support .into()!"))
    }
}


/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
pub struct NoProtoPointer<'a, T: NoProtoValue<'a> + Default> {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    pub schema: &'a NoProtoSchema,
    cached: bool,
    value: T
}

impl<'a, T: NoProtoValue<'a> + Default> NoProtoPointer<'a, T> {
/*
    #[doc(hidden)]
    pub fn new_example_ptr(schema: &'a NoProtoSchema, _value: T) -> Self {

        NoProtoPointer {
            address: 0,
            kind: &NoProtoPointerKinds::Standard { value: 0 },
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![0, 0, 0, 0] })),
            schema: schema,
            cached: false,
            value: T::default()
        }
    }

    #[doc(hidden)]
    pub fn new(address: u32, schema: &'a NoProtoSchema, memory: Rc<RefCell<NoProtoMemory>>) -> Self {

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
            memory: memory,
            schema: schema,
            cached: false,
            value: T::default()
        }
    }
*/

    pub fn get(&mut self) -> std::result::Result<Option<&T>, NoProtoError> {

        if self.cached {
            return Ok(Some(&self.value));
        }

        let value = T::buffer_get(self.address, &self.kind, self.schema, Rc::clone(&self.memory))?;
        
        Ok(match value {
            Some (x) => {
                self.value = *x;
                self.cached = true;
                Some(&self.value)
            },
            None => None
        })
    }

    pub fn set(&mut self, value: T) -> std::result::Result<(), NoProtoError> {
        self.kind = T::buffer_set(self.address, &self.kind, self.schema, Rc::clone(&self.memory), Box::new(&value))?;
        self.value = value;
        self.cached = true;
        Ok(())
    }

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
            schema: schema,
            cached: false,
            value: T::default()
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
            schema: schema,
            cached: false,
            value: T::default()
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
            schema: schema,
            cached: false,
            value: T::default()
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
            schema: schema,
            cached: false,
            value: T::default()
        })
    }

    pub fn has_value(&mut self) -> bool {
        if self.address == 0 { return false; } else { return true; }
    }

    pub fn clear(&mut self) -> std::result::Result<(), NoProtoError> {
        // self.kind.set_value_address(self.address, 0, Rc::clone(&self.memory));
        Ok(())
    }

    pub fn convert(self) -> std::result::Result<Option<T>, NoProtoError> {
        let result = T::buffer_into(self.address, self.kind, self.schema, self.memory)?;

        Ok(match result {
            Some(x) => Some(*x),
            None => None
        })
    }

    /*
    pub fn as_table(&mut self) -> std::result::Result<T, NoProtoError> {

        match &*self.schema.kind {
            NoProtoSchemaKinds::Table { columns } => {

                let mut addr = self.kind.get_value();

                let mut head: [u8; 4] = [0; 4];

                // no table here, make one
                if addr == 0 {
                    // no table here, make one
                    let mut memory = self.memory.try_borrow_mut()?;
                    addr = memory.malloc([0 as u8; 4].to_vec())?; // stores HEAD for table
                    memory.set_value_address(self.address, addr, &self.kind)?;
                } else {
                    // existing head, read value
                    let b_bytes = &self.memory.try_borrow()?.bytes;
                    let a = addr as usize;
                    head.copy_from_slice(&b_bytes[a..(a+4)]);
                }

                unsafe { Ok(NoProtoTable::new(addr, u32::from_le_bytes(head), Rc::clone(&self.memory), &columns)) }
            },
            _ => {
                Err(NoProtoError::new(""))
            }
        }
    }*/

/*
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