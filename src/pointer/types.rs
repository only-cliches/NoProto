use std::{fmt, time::SystemTime};
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