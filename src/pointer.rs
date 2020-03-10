extern crate rand;

use crate::memory::NoProtoMemory;
use crate::NoProtoError;
use crate::{schema::NoProtoSchemaKinds, schema::NoProtoSchema, collection::{map::NoProtoMap, list::NoProtoList, table::NoProtoTable, tuple::NoProtoTuple}};
use std::rc::Rc;
use std::cell::RefCell;
use rand::Rng;
use std::fmt;
use std::time::{SystemTime};

fn to_hex(num: u64, length: i32) -> String {
    let mut result: String = "".to_owned();

    let hex_values = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];

    let mut i = length - 1;
    while i >= 0 {
        let raise = (16i32).pow(i as u32) as f64;
        let index = (num as f64 / raise).floor() as i32;
        result.push_str(hex_values[(index % 16i32) as usize]);
        i -= 1 ;
    }

    result
}

/// Represents a Big Integer Decimal
/// 
/// Allows floating point values to be stored without rounding errors, useful for storing financial data.
/// 
/// NoProto does not implement arithamtic between Big Integer Deciamls, it's recommended you use a crate like `rust_decimal` to perform calculations.  
/// 
/// Do NOT use the conversion to floating point to perform calculations, it'll kind of make the use of this data type moot.
pub struct NoProtoDec {
    num: i64,
    scale: u8
}

impl NoProtoDec {
    pub fn to_float(&self) -> f64 {
        let bottom = 10i32.pow(self.scale as u32)  as f64;

        let m = self.num as f64;

        m / bottom
    }

    pub fn new(num: i64, scale: u8) -> Self {
        NoProtoDec { num, scale }
    }

    pub fn export(&self) -> (i64, u8) {
        (self.num, self.scale)
    }
}

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
}

/// Represents a Geographic Coordinate (lat / lon)
/// 
/// When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
#[derive(Debug)]
pub struct NoProtoGeo {
    pub lat: f64,
    pub lon: f64
}

/// Represents a Time ID type which has a 64 bit timestamp and 64 random bits.
/// 
/// Useful for storing time stamp data that can't have collisions.
pub struct NoProtoTimeID {
    pub id: [u8; 8],
    pub time: u64
}

impl NoProtoTimeID {

    pub fn generate(id_bytes: Option<[u8; 8]>) -> NoProtoTimeID {
        let mut rng = rand::thread_rng();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        match id_bytes {
            Some(x) => {
                NoProtoTimeID {
                    time: now,
                    id: x
                }
            },
            None => {
                let mut id = [0; 8];

                for x in 0..id.len() {
                    id[x] = rng.gen_range(0, 255);
                }
        
                NoProtoTimeID {
                    time: now,
                    id: id
                }
            }
        }
    }

    pub fn to_string(&self, time_padding: Option<u8>) -> String {
        let mut result: String = "".to_owned();

        // u64 can hold up to 20 digits or 584,942,417,355 years of seconds since unix epoch
        // 14 digits gets us 3,170,979 years of seconds after Unix epoch.  
        let mut padding = time_padding.unwrap_or(14);

        if padding < 10 {
            padding = 10;
        }

        if padding > 20 {
            padding = 20;
        }

        // time first
        let formatted_string = match padding {
            10 => { format!("{:0>10}", self.time) },
            11 => { format!("{:0>11}", self.time) },
            12 => { format!("{:0>12}", self.time) },
            13 => { format!("{:0>13}", self.time) }
            14 => { format!("{:0>14}", self.time) },
            15 => { format!("{:0>15}", self.time) },
            16 => { format!("{:0>16}", self.time) },
            17 => { format!("{:0>17}", self.time) },
            18 => { format!("{:0>18}", self.time) },
            19 => { format!("{:0>19}", self.time) },
            20 => { format!("{:0>20}", self.time) },
            _ => { "".to_owned() }
        };

        result.push_str(formatted_string.as_str());
        result.push_str("-");

        // then id
        for x in 0..self.id.len() {
            let value = self.id[x] as u64;
            if x == 4 {
                result.push_str("-");
            }
            result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl fmt::Debug for NoProtoTimeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(Some(20)))
    }
}
/// Represents a V4 UUID, good for globally unique identifiers
/// 
/// `uuid` types are always represented with this struct.
pub struct NoProtoUUID {
    pub value: [u8; 16]
}

impl NoProtoUUID {

    pub fn generate() -> NoProtoUUID {

        let mut rng = rand::thread_rng();

        let mut uuid = NoProtoUUID {
            value: [0; 16]
        };

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + rng.gen_range(0, 15);
            } else {
                uuid.value[x] = rng.gen_range(0, 255);
            }
        }

        uuid
    }

    pub fn to_string(&self) -> String {

        let mut result: String = "".to_owned();

        for x in 0..self.value.len() {
            if x == 4 || x == 6 || x == 8 || x == 10 {
                result.push_str("-");
            }
            let value = self.value[x] as u64;
            result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl fmt::Debug for NoProtoUUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

#[doc(hidden)]
pub enum NoProtoPointerKinds {
    // scalar / collection
    Standard  { value: u32 }, // 4 bytes [4]

    // collection items
    MapItem   { value: u32, next: u32, key: u32 }, // 12 bytes [4, 4, 4]
    TableItem { value: u32, next: u32, i: u8    }, // 9  bytes [4, 4, 1]
    ListItem  { value: u32, next: u32, i: u16   }  // 10 bytes [4, 4, 2]
}

/// The base data type, all information is stored/retrieved against pointers
/// 
/// Each pointer represents at least a 32 bit unsigned integer that is either zero for no value or points to an offset in the buffer.  All pointer addresses are zero based against the beginning of the buffer.
pub struct NoProtoPointer<'a> {
    address: u32, // pointer location
    kind: NoProtoPointerKinds,
    memory: Rc<RefCell<NoProtoMemory>>,
    schema: &'a NoProtoSchema
}

impl<'a> NoProtoPointer<'a> {

    #[doc(hidden)]
    pub fn new_example_ptr(schema: &'a NoProtoSchema) -> Self {

        NoProtoPointer {
            address: 0,
            kind: NoProtoPointerKinds::Standard { value: 0 },
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: vec![0, 0, 0, 0] })),
            schema: schema
        }
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

    pub fn has_value(self) -> bool {
        if self.address == 0 { return false; } else { return true; }
    }

    pub fn clear(&mut self) -> std::result::Result<(), NoProtoError> {
        self.set_value_address(0)?;
        Ok(())
    }

    fn get_value_address(&self) -> u32 {
        match self.kind {
            NoProtoPointerKinds::Standard  { value } =>                      { value },
            NoProtoPointerKinds::MapItem   { value, key: _,  next: _ } =>    { value },
            NoProtoPointerKinds::TableItem { value, i: _,    next: _ } =>    { value },
            NoProtoPointerKinds::ListItem  { value, i:_ ,    next: _ } =>    { value }
        }
    }

    fn set_value_address(&mut self, val: u32) -> std::result::Result<(), NoProtoError> {

        let mut memory = self.memory.try_borrow_mut()?;

        let addr_bytes = val.to_le_bytes();
    
        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }

        match self.kind {
            NoProtoPointerKinds::Standard { value: _ } => {
                self.kind = NoProtoPointerKinds::Standard { value: val}
            },
            NoProtoPointerKinds::MapItem { value: _, key,  next  } => {
                self.kind = NoProtoPointerKinds::MapItem { value: val, key: key, next: next }
            },
            NoProtoPointerKinds::TableItem { value: _, i, next  } => {
                self.kind = NoProtoPointerKinds::TableItem { value: val, i: i, next: next }
            },
            NoProtoPointerKinds::ListItem { value: _, i, next  } => {
                self.kind = NoProtoPointerKinds::ListItem { value: val, i: i, next: next }
            }
        };

        Ok(())
    }

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
 

    pub fn to_string(&self) -> std::result::Result<Option<String>, NoProtoError> {

        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Utf8String => {

                let addr = self.get_value_address() as usize;

                // empty value
                if addr == 0 {
                    return Ok(None);
                }
                
                // get size of string
                let mut size: [u8; 4] = [0; 4];
                let memory = self.memory.try_borrow()?;
                size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                let str_size = u32::from_le_bytes(size) as usize;

                // get string bytes
                let array_bytes = &memory.bytes[(addr+4)..(addr+4+str_size)];

                // convert to string
                let string = String::from_utf8(array_bytes.to_vec())?;

                Ok(Some(string))
            },
            _ => {
                Err(type_error(TypeReq::Read, "string", &model))
            }
        }
    }

    pub fn set_string(&mut self, value: &str) -> std::result::Result<(), NoProtoError> {

        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Utf8String => {
                let bytes = value.as_bytes();
                let str_size = bytes.len() as u64;

                if str_size >= std::u32::MAX as u64 { 
                    Err(NoProtoError::new("String too large!"))
                } else {

                    let mut addr = self.get_value_address() as usize;
                    let mut set_addr = false;

                    {
                        let mut memory = self.memory.try_borrow_mut()?;

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

                            set_addr = true;

                            // then string content
                            memory.malloc(bytes.to_vec())?;
                        }
                    }

                    if set_addr { self.set_value_address(addr as u32)?; };
            
                    Ok(())
                }

            }
            _ => {
                Err(type_error(TypeReq::Write, "string", &model))
            }
        }
    }

    pub fn to_bytes(&self) -> std::result::Result<Vec<u8>, NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Bytes => {
                let value = self.get_value_address();

                // empty value
                if value == 0 {
                    return Err(NoProtoError::new("NULL"));
                }
                
                // get size of bytes
                let addr = value as usize;
                let mut size: [u8; 4] = [0; 4];
                let memory = self.memory.try_borrow()?;
                size.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                let bytes_size = u32::from_le_bytes(size) as usize;

                // get string bytes
                let bytes = &memory.bytes[(addr+4)..(addr+4+bytes_size)];

                Ok(bytes.to_vec())
            },
            _ => {
                Err(type_error(TypeReq::Read, "bytes", &model))
            }
        }
    }

    pub fn set_bytes(&mut self, bytes: Vec<u8>) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        match *model.kind {
            NoProtoSchemaKinds::Bytes => {

                let size = bytes.len() as u64;

                if size >= std::u32::MAX as u64 { 
                    Err(NoProtoError::new("Bytes too large!"))
                } else {

                    let mut addr = self.get_value_address() as usize;
                    let mut set_addr = false;

                    {
                        let mut memory = self.memory.try_borrow_mut()?;

                        let prev_size: usize = if addr != 0 {
                            let mut size_bytes: [u8; 4] = [0; 4];
                            size_bytes.copy_from_slice(&memory.bytes[addr..(addr+4)]);
                            u32::from_le_bytes(size_bytes) as usize
                        } else {
                            0 as usize
                        };
    
                        if prev_size >= size as usize { // previous bytes is larger than this one, use existing memory
                    
                            let size_bytes = size.to_le_bytes();
                            // set string size
                            for x in 0..size_bytes.len() {
                                memory.bytes[(addr + x) as usize] = size_bytes[x as usize];
                            }
    
                            // set bytes
                            for x in 0..bytes.len() {
                                memory.bytes[(addr + x + 4) as usize] = bytes[x as usize];
                            }
    
                        } else { // not enough space or space has not been allocted yet
                            

                            // first 4 bytes are length
                            addr = memory.malloc((size as u32).to_le_bytes().to_vec())? as usize;

                            set_addr = true;

                            // then bytes content
                            memory.malloc(bytes.to_vec())?;
                        }
                    }

                    if set_addr { self.set_value_address(addr as u32)?; } ;
            
                    Ok(())
                }
            },
            _ => {
                Err(type_error(TypeReq::Write, "bytes", &model))
            }
        }
    }

    #[doc(hidden)]
    pub fn get_1_byte(&self) -> std::result::Result<Option<[u8; 1]>, NoProtoError> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return Ok(None);
        }

        let memory = self.memory.try_borrow()?;

        Ok(Some([memory.bytes[value]]))
    }

    #[doc(hidden)]
    pub fn get_2_bytes(&self) -> std::result::Result<Option<[u8; 2]>, NoProtoError> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return Ok(None);
        }

        let mut bytes: [u8; 2] = [0; 2];
        let memory = self.memory.try_borrow()?;
        bytes.copy_from_slice(&memory.bytes[value..(value + 2)]);

        Ok(Some(bytes))
    }

    #[doc(hidden)]
    pub fn get_4_bytes(&self) -> std::result::Result<Option<[u8; 4]>, NoProtoError> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return Ok(None);
        }

        let mut bytes: [u8; 4] = [0; 4];
        let memory = self.memory.try_borrow()?;
        bytes.copy_from_slice(&memory.bytes[value..(value + 4)]);

        Ok(Some(bytes))
    }

    #[doc(hidden)]
    pub fn get_8_bytes(&self) -> std::result::Result<Option<[u8; 8]>, NoProtoError> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return Ok(None);
        }

        let mut bytes: [u8; 8] = [0; 8];
        let memory = self.memory.try_borrow()?;
        bytes.copy_from_slice(&memory.bytes[value..(value + 8)]);

        Ok(Some(bytes))
    }

    #[doc(hidden)]
    pub fn get_16_bytes(&self) -> std::result::Result<Option<[u8; 16]>, NoProtoError> {
        let value = self.get_value_address() as usize;

        // empty value
        if value == 0 {
            return Ok(None);
        }

        let mut bytes: [u8; 16] = [0; 16];
        let memory = self.memory.try_borrow()?;
        bytes.copy_from_slice(&memory.bytes[value..(value + 16)]);

        Ok(Some(bytes))
    }

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
    
    /// Takes an integer and reduces it to be within the range allowed for the pointer's intenger type.
    /// 
    /// If the integer is already within the allowed range, this does nothing.
    /// 
    /// If the pointer is not an integer type, this returns zero.
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
    /// assert_eq!(uint8_ptr.integer_truncate(2938), 255);
    /// assert_eq!(uint8_ptr.integer_truncate(-20329383), 0);
    /// assert_eq!(uint8_ptr.integer_truncate(155), 155);
    /// # Ok::<(), NoProtoError>(())
    /// ```
    pub fn integer_truncate(&self, value: i128) -> i128 {
        let ranges = self.get_integer_range();

        if ranges.0 > value {
            return ranges.0;
        } else if ranges.1 < value {
            return ranges.1;
        }

        value
    }

    /// Allows you to set an arbitrary integer value against the pointer.  This will work as long as the pointer is one of the integer (intX) or unsigned integer (uintX) types.
    /// 
    /// If the pointer is not an integer (intX) or unsigned integer (intX) type, this will fail with a type error.
    /// If the value provided is outside of the range of numbers that can safely be held by the pointer's type, this will fail.
    /// Can be used with `integer_truncate` to garuantee value being passed in is safe to store.
    /// 
    /// # Example: 
    /// Assuming `uint8_ptr` is a `NoProtoPointer` of type `uint8`.
    /// ```
    /// # use json::*;
    /// # use no_proto::error::NoProtoError;
    /// # use no_proto::schema::NoProtoSchema;
    /// # use no_proto::pointer::NoProtoPointer;
    /// # let schema = NoProtoSchema::init().from_json(object!{"type" => "uint8"})?;
    /// # let mut uint8_ptr = NoProtoPointer::new_example_ptr(&schema);
    /// uint8_ptr.set_generic_integer(120);
    /// 
    /// assert_eq!(uint8_ptr.to_uint8()?, Some(120));
    /// # Ok::<(), NoProtoError>(())
    /// ```
    pub fn set_generic_integer(&mut self, value: i128) -> std::result::Result<(), NoProtoError> {
        let model = self.schema;

        let ranges = self.get_integer_range();

        if ranges.0 > value {
            return Err(NoProtoError::new(format!("Integer value {} is too low for type ({}), minimum allowed value is {}!", value, model.get_type_str(), ranges.0).as_str()));
        } else if ranges.1 < value {
            return Err(NoProtoError::new(format!("Integer value {} is too high for type ({}), maximum allowed value is {}!", value, model.get_type_str(), ranges.1).as_str()));
        }

        match *model.kind {
            NoProtoSchemaKinds::Int8 => { self.set_int8(value as i8) },
            NoProtoSchemaKinds::Int16 => { self.set_int16(value as i16) },
            NoProtoSchemaKinds::Int32 => { self.set_int32(value as i32) },
            NoProtoSchemaKinds::Int64 => { self.set_int64(value as i64) },
            NoProtoSchemaKinds::Uint8 => { self.set_uint8(value as u8) },
            NoProtoSchemaKinds::Uint16 => { self.set_uint16(value as u16) },
            NoProtoSchemaKinds::Uint32 => { self.set_uint32(value as u32) },
            NoProtoSchemaKinds::Uint64 => { self.set_uint64(value as u64) }
            _ => {
                Err(type_error(TypeReq::Write, "int8, int16, int32, int64, uint8, uint16, uint32, or uint64", &model))
            }
        }
    }

    /// Allows you to get an arbitrary integer value from the pointer.  This will work as long as the pointer is one of the integer (intX) or unsigned integer (uintX) types.
    /// 
    /// If the pointer is not an integer (intX) or unsigned integer (intX) type, this will fail with a type error.
    /// 
    /// This is identical to calling the specific integer methods but a little slower.
    /// 
    /// # Example: 
    /// Assuming `my_ptr` is a `NoProtoPointer` of type `uint8`.
    /// ```
    /// # use json::*;
    /// # use no_proto::schema::NoProtoSchema;
    /// # use no_proto::pointer::NoProtoPointer;
    /// # use no_proto::error::NoProtoError;
    /// # let schema = NoProtoSchema::init().from_json(object!{"type" => "uint8"})?;
    /// # let mut my_ptr = NoProtoPointer::new_example_ptr(&schema);
    /// my_ptr.set_uint8(120);
    /// 
    /// assert_eq!(my_ptr.to_generic_integer()?, Some(120));
    /// assert_eq!(my_ptr.to_uint8()?, Some(120));
    /// # Ok::<(), NoProtoError>(())
    /// ```
    pub fn to_generic_integer(&self) -> std::result::Result<Option<i128>, NoProtoError> {
        let model = self.schema;

        if self.get_value_address() == 0 {
            return Ok(None);
        };

        match *model.kind {
            NoProtoSchemaKinds::Int8 => { Ok(Some(self.to_int8()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Int16 => { Ok(Some(self.to_int16()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Int32 => { Ok(Some(self.to_int32()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Int64 => { Ok(Some(self.to_int64()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Uint8 => { Ok(Some(self.to_uint8()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Uint16 => { Ok(Some(self.to_uint16()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Uint32 => { Ok(Some(self.to_uint32()?.unwrap_or(0) as i128)) },
            NoProtoSchemaKinds::Uint64 => { Ok(Some(self.to_uint64()?.unwrap_or(0) as i128)) },
            _ => {
                Err(type_error(TypeReq::Write, "int8, int16, int32, int64, uint8, uint16, uint32, or uint64", &model))
            }
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