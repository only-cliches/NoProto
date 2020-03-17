use crate::pointer::NP_ValueInto;
use json::JsonValue;
use crate::schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys};
use crate::pointer::NP_PtrKinds;
use crate::{memory::NP_Memory, pointer::NP_Value, error::NP_Error};
use std::{fmt, time::SystemTime, cell::RefCell, rc::Rc};
use rand::Rng;

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
/// NP_ does not implement arithamtic between Big Integer Deciamls, it's recommended you use a crate like `rust_decimal` to perform calculations.  
/// 
/// Do NOT use the conversion to floating point to perform calculations, it'll kind of make the use of this data type moot.
pub struct NP_Dec {
    num: i64,
    scale: u8
}

impl NP_Dec {
    pub fn to_float(&self) -> f64 {
        let bottom = 10i32.pow(self.scale as u32)  as f64;

        let m = self.num as f64;

        m / bottom
    }

    pub fn new(num: i64, scale: u8) -> Self {
        NP_Dec { num, scale }
    }

    pub fn export(&self) -> (i64, u8) {
        (self.num, self.scale)
    }
}

impl Default for NP_Dec {
    fn default() -> Self { 
        NP_Dec::new(0,0)
     }
}

impl NP_Value for NP_Dec {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Dec::new(0,0)
    }

    fn is_type( type_str: &str) -> bool {
        "dec64" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Dec64 as i64, "dec64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Dec64 as i64, "dec64".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Dec::new(i64::from_le_bytes(x), u8::from_le_bytes([memory.bytes[(addr + 8) as usize]]))))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.num.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                let bytes2 = value.scale.to_le_bytes();
                memory.bytes[(addr + 8) as usize] = bytes2[0];

                return Ok(*kind);
            } else { // new value

                let bytes = value.num.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;
                memory.malloc(value.scale.to_le_bytes().to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind));
            }
        }
        
    }
}

impl<'a> NP_ValueInto<'a> for NP_Dec {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Dec::new(i64::from_le_bytes(x), u8::from_le_bytes([memory.bytes[(addr + 8) as usize]]))))
            },
            None => None
        })
    }
}


/// Represents a Geographic Coordinate (lat / lon)
/// 
/// When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
#[derive(Debug)]
pub struct NP_Geo {
    pub lat: f64,
    pub lon: f64
}

impl Default for NP_Geo {
    fn default() -> Self { 
        NP_Geo { lat: 0.0, lon: 0.0 }
     }
}

impl NP_Value for NP_Geo {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Geo { lat: 0.0, lon: 0.0 }
    }

    fn is_type( type_str: &str) -> bool {
        "geo4" == type_str || "geo8" == type_str || "geo16" == type_str 
    }

    fn schema_state(type_string: &str, _json_schema: &JsonValue) -> std::result::Result<i64, NP_Error> {
        Ok(match type_string {
            "geo4" => 4,
            "geo8" => 8,
            "geo16" => 16,
            _ => 0
        })
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Geo as i64, "geo".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Geo as i64, "geo".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        match schema.type_state {
            4 => {
                Ok(match memory.get_4_bytes(addr) {
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

                        Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
                    },
                    None => None
                })
            },
            8 => {
                Ok(match memory.get_8_bytes(addr) {
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
    
                        Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
                    },
                    None => None
                })
            },
            16 => {
                Ok(match memory.get_16_bytes(addr) {
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
        
                        Some(Box::new(NP_Geo { lat: f64::from_le_bytes(bytes_lat), lon: f64::from_le_bytes(bytes_lon)}))
                    },
                    None => None
                })   
            }
            _ => {
                unreachable!();
            }
        }
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {

            let mut memory = buffer.try_borrow_mut()?;


            let value_bytes_size = schema.type_state as usize;

            if value_bytes_size == 0 {
                unreachable!();
            }

            let half_value_bytes = value_bytes_size / 2;

            // convert input values into bytes
            let value_bytes = match schema.type_state {
                16 => {
                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = value.lat.to_le_bytes();
                    let lon_bytes = value.lon.to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                8 => {
                    let dev = 10000000f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((value.lat * dev) as i32).to_le_bytes();
                    let lon_bytes = ((value.lon * dev) as i32).to_le_bytes();

                    for x in 0..value_bytes_size {
                        if x < half_value_bytes {
                            v_bytes[x] = lat_bytes[x];
                        } else {
                            v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                        }
                    }
                    v_bytes
                },
                4 => {
                    let dev = 100f64;

                    let mut v_bytes: [u8; 16] = [0; 16];
                    let lat_bytes = ((value.lat * dev) as i16).to_le_bytes();
                    let lon_bytes = ((value.lon * dev) as i16).to_le_bytes();

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

                return Ok(*kind);

            } else { // new value

                addr = match schema.type_state {
                    16 => { memory.malloc([0; 16].to_vec())? },
                    8 => { memory.malloc([0; 8].to_vec())? },
                    4 => { memory.malloc([0; 4].to_vec())? },
                    _ => { 0 }
                };

                // set values in buffer
                for x in 0..value_bytes.len() {
                    if x < value_bytes_size {
                        memory.bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                    }
                }

                return Ok(memory.set_value_address(address, addr as u32, kind));
            }
        }
    }
}

impl<'a> NP_ValueInto<'a> for NP_Geo {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        match schema.type_state {
            4 => {
                Ok(match memory.get_4_bytes(addr) {
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

                        Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
                    },
                    None => None
                })
            },
            8 => {
                Ok(match memory.get_8_bytes(addr) {
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
    
                        Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
                    },
                    None => None
                })
            },
            16 => {
                Ok(match memory.get_16_bytes(addr) {
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
        
                        Some(Box::new(NP_Geo { lat: f64::from_le_bytes(bytes_lat), lon: f64::from_le_bytes(bytes_lon)}))
                    },
                    None => None
                })   
            }
            _ => {
                unreachable!();
            }
        }
    }
}

/// Represents a Time ID type which has a 64 bit timestamp and 64 random bits.
/// 
/// Useful for storing time stamp data that can't have collisions.
pub struct NP_TimeID {
    pub id: [u8; 8],
    pub time: u64
}

impl NP_TimeID {

    pub fn generate(id_bytes: Option<[u8; 8]>) -> NP_TimeID {
        let mut rng = rand::thread_rng();
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        match id_bytes {
            Some(x) => {
                NP_TimeID {
                    time: now,
                    id: x
                }
            },
            None => {
                let mut id = [0; 8];

                for x in 0..id.len() {
                    id[x] = rng.gen_range(0, 255);
                }
        
                NP_TimeID {
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


impl Default for NP_TimeID {
    fn default() -> Self { 
        NP_TimeID { id: [0; 8], time: 0 }
     }
}

impl fmt::Debug for NP_TimeID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string(Some(20)))
    }
}

impl NP_Value for NP_TimeID {

    fn new<T: NP_Value + Default>() -> Self {
        NP_TimeID { id: [0; 8], time: 0 }
    }

    fn is_type( type_str: &str) -> bool {
        "tid" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Tid as i64, "tid".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Tid as i64, "tid".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                let mut id_bytes: [u8; 8] = [0; 8];
                id_bytes.copy_from_slice(&x[0..8]);

                let mut time_bytes: [u8; 8] = [0; 8];
                time_bytes.copy_from_slice(&x[8..16]);

                Some(Box::new(NP_TimeID {
                    id: id_bytes,
                    time: u64::from_le_bytes(time_bytes)
                }))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace

                let time_bytes = value.time.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..16 {
                    if x < 8 {
                        memory.bytes[(addr + x as u32) as usize] = value.id[x as usize];
                    } else {
                        memory.bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                    }
                }

                return Ok(*kind);

            } else { // new value

                let mut bytes: [u8; 16] = [0; 16];
                let time_bytes = value.time.to_le_bytes();

                for x in 0..bytes.len() {
                    if x < 8 {
                        bytes[(addr + x as u32) as usize] = value.id[x as usize];
                    } else {
                        bytes[(addr + x as u32) as usize] = time_bytes[x as usize - 8];
                    }
                }

                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind));
            }                    
        }
    }
}

impl<'a> NP_ValueInto<'a> for NP_TimeID {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                let mut id_bytes: [u8; 8] = [0; 8];
                id_bytes.copy_from_slice(&x[0..8]);

                let mut time_bytes: [u8; 8] = [0; 8];
                time_bytes.copy_from_slice(&x[8..16]);

                Some(Box::new(NP_TimeID {
                    id: id_bytes,
                    time: u64::from_le_bytes(time_bytes)
                }))
            },
            None => None
        })
    }
}

/// Represents a V4 UUID, good for globally unique identifiers
/// 
/// `uuid` types are always represented with this struct.
pub struct NP_UUID {
    pub value: [u8; 16]
}

impl NP_UUID {

    pub fn generate() -> NP_UUID {

        let mut rng = rand::thread_rng();

        let mut uuid = NP_UUID {
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

impl fmt::Debug for NP_UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for NP_UUID {
    fn default() -> Self { 
        NP_UUID { value: [0; 16] }
     }
}

impl NP_Value for NP_UUID {

    fn new<T: NP_Value + Default>() -> Self {
        NP_UUID { value: [0; 16] }
    }

    fn is_type( type_str: &str) -> bool {
        "uuid" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uuid as i64, "uuid".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uuid as i64, "uuid".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_UUID { value: x}))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.value;

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);

            } else { // new value

                let bytes = value.value;
                addr = memory.malloc(bytes.to_vec())?;

                return Ok(memory.set_value_address(address, addr as u32, kind));
            }                    
        }
    }
}

impl<'a> NP_ValueInto<'a> for NP_UUID {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_UUID { value: x}))
            },
            None => None
        })
    }
}

pub struct NP_Option {
    pub value: Option<String>
}

impl NP_Value for NP_Option {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Option { value: None }
    }

    fn is_type( type_str: &str) -> bool {
        "option" == type_str || "enum" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Enum as i64, "option".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Enum as i64, "option".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        match &*schema.kind {
            NP_SchemaKinds::Enum { choices } => {

                Ok(match memory.get_1_byte(addr) {
                    Some(x) => {
                        let value_num = u8::from_le_bytes([x]) as usize;
        
                        if value_num > choices.len() {
                            None
                        } else {
                            Some(Box::new(NP_Option { value: Some(choices[value_num].clone()) }))
                        }
                    },
                    None => None
                })
            },
            _ => {
                Err(NP_Error::new(format!("TypeError: Attempted to read value of type ({}) to pointer of type ({})!", "option", schema.type_data.1).as_str()))
            }
        }
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();
        
        match &*schema.kind {
            NP_SchemaKinds::Enum { choices } => {

                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;

                    for opt in choices {
                        if value.value == Some(opt.to_string()) {
                            value_num = ct as i32;
                        }
                        ct += 1;
                    };

                    if value_num == -1 {
                        return Err(NP_Error::new("Option not found, cannot set uknown option!"));
                    }
                }

                {
                    let mut memory = buffer.try_borrow_mut()?;

                    let bytes = (value_num as u8).to_le_bytes();

                    if addr != 0 { // existing value, replace

                        // overwrite existing values in buffer
                        for x in 0..bytes.len() {
                            memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                        }
                        return Ok(*kind);

                    } else { // new value
    
                        addr = memory.malloc(bytes.to_vec())?;

                        return Ok(memory.set_value_address(address, addr as u32, kind));
                    }                    
                }
                
            },
            _ => {
                Err(NP_Error::new(format!("TypeError: Attempted to read value of type ({}) to pointer of type ({})!", "option", schema.type_data.1).as_str()))
            }
        }
    }
}

impl Default for NP_Option {
    fn default() -> Self { 
        NP_Option { value: None }
     }
}

impl<'a> NP_ValueInto<'a> for NP_Option {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        match &*schema.kind {
            NP_SchemaKinds::Enum { choices } => {

                Ok(match memory.get_1_byte(addr) {
                    Some(x) => {
                        let value_num = u8::from_le_bytes([x]) as usize;
        
                        if value_num > choices.len() {
                            None
                        } else {
                            Some(Box::new(NP_Option { value: Some(choices[value_num].clone()) }))
                        }
                    },
                    None => None
                })
            },
            _ => {
                Err(NP_Error::new(format!("TypeError: Attempted to read value of type ({}) to pointer of type ({})!", "option", schema.type_data.1).as_str()))
            }
        }
    }
}


impl NP_Value for bool {

    fn new<T: NP_Value + Default>() -> Self {
        false
    }

    fn is_type( type_str: &str) -> bool {
        "bool" == type_str || "boolean" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Boolean as i64, "bool".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Boolean as i64, "bool".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(if x == 1 { true } else { false }))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = if **value == true {
                    [1] as [u8; 1]
                } else {
                    [0] as [u8; 1]
                };

                // overwrite existing values in buffer
                memory.bytes[addr as usize] = bytes[0];

                return Ok(*kind);

            } else { // new value

                let bytes = if **value == true {
                    [1] as [u8; 1]
                } else {
                    [0] as [u8; 1]
                };

                addr = memory.malloc(bytes.to_vec())?;
                return Ok(memory.set_value_address(address, addr as u32, kind));
            }
        }
    }
}

impl<'a> NP_ValueInto<'a> for bool {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(if x == 1 { true } else { false }))
            },
            None => None
        })
    }
}

pub struct NP_Date {
    pub value: u64
}

impl NP_Date {
    pub fn now() -> Self {
        NP_Date { value: SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() }
    }
}

impl Default for NP_Date {
    fn default() -> Self { 
        NP_Date { value: 0 }
     }
}

impl fmt::Debug for NP_Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl NP_Value for NP_Date {

    fn new<T: NP_Value + Default>() -> Self {
        NP_Date { value: 0 }
    }

    fn is_type( type_str: &str) -> bool {
        "date" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Date as i64, "date".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Date as i64, "date".to_owned()) }

    fn buffer_get(_address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Date { value: u64::from_le_bytes(x) }))
            },
            None => None
        })
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, buffer: Rc<RefCell<NP_Memory>>, value: Box<&Self>) -> std::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        {
            let mut memory = buffer.try_borrow_mut()?;

            if addr != 0 { // existing value, replace
                let bytes = value.value.to_le_bytes();

                // overwrite existing values in buffer
                for x in 0..bytes.len() {
                    memory.bytes[(addr + x as u32) as usize] = bytes[x as usize];
                }

                return Ok(*kind);

            } else { // new value

                let bytes = value.value.to_le_bytes();
                addr = memory.malloc(bytes.to_vec())?;
                return Ok(memory.set_value_address(address, addr as u32, kind));
            }                    
        }
    }
}

impl<'a> NP_ValueInto<'a> for NP_Date {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: Rc<RefCell<NP_Memory>>) -> std::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer.try_borrow()?;
        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Date { value: u64::from_le_bytes(x) }))
            },
            None => None
        })
    }
}