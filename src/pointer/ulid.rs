//! Represents a ULID type which has a 6 byte timestamp and 10 bytes of randomness
//! 
//! Useful for storing time stamp data that doesn't have collisions.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::ulid::NP_ULID;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "ulid"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.deep_set("", NP_ULID::generate(1604965249484, 50))?;
//! 
//! assert_eq!("1EPQP4CEC3KANC3XYNG9YKAQ", new_buffer.deep_get::<NP_ULID>("")?.unwrap().to_string());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::utils::to_base32;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::pointer::NP_PtrKinds;
use crate::{pointer::NP_Value, error::NP_Error, utils::{Rand}};
use core::{fmt::{Debug, Formatter}};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::NP_Lite_Ptr;



/// Holds ULIDs which are good for time series keys.
/// 
/// Check out documentation [here](../ulid/index.html).
/// 
#[derive(Eq, PartialEq)]
pub struct NP_ULID {
    /// The unix timestamp in milliseconds for this ULID
    pub time: u64,
    /// The random bytes for this ULID
    pub id: u128
}

impl NP_ULID {

    /// Creates a new ULID from the timestamp and provided seed.
    /// 
    /// The random seed is used to generate the ID, the same seed will always lead to the same random bytes so try to use something actually random for the seed.
    /// 
    /// The time should be passed in as the unix epoch in milliseconds.
    pub fn generate(now_ms: u64, random_seed: u32) -> NP_ULID {
        let mut rng = Rand::new(random_seed);

        let mut id: [u8; 16] = [0; 16];

        for x in 6..id.len() {
            id[x] = rng.gen_range(0, 255) as u8;
        }

        NP_ULID {
            time: now_ms,
            id: u128::from_be_bytes(id)
        }
    }

    /// Generates a ULID with the given time and a provided random number generator.
    /// This is the preferrable way to generate a ULID, if you can provide a better RNG function than the psudorandom one built into this library, you should.
    /// 
    pub fn generate_with_rand<F>(now_ms: u64, random_fn: F) -> NP_ULID where F: Fn(u8, u8) -> u8 {

        let mut id: [u8; 16] = [0; 16];

        for x in 6..id.len() {
            id[x] = random_fn(0, 255);
        }

        NP_ULID {
            time: now_ms,
            id: u128::from_be_bytes(id)
        }
    }

    /// Generates a stringified version of this ULID with base32.
    /// 
    pub fn to_string(&self) -> String {
        let mut result: String = "".to_owned();

        result.push_str(to_base32(self.time as u128, 10).as_str());
        result.push_str(to_base32(self.id, 16).as_str());

        result
    }
}


impl Default for NP_ULID {
    fn default() -> Self { 
        NP_ULID { id: 0, time: 0 }
     }
}

impl Debug for NP_ULID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl<'value> NP_Value<'value> for NP_ULID {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Ulid as u8, "ulid".to_owned(), NP_TypeKeys::Ulid) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Ulid as u8, "ulid".to_owned(), NP_TypeKeys::Ulid) }

    fn schema_to_json(_schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let time_bytes: [u8; 8] = value.time.to_be_bytes();
        let id_bytes: [u8; 16] = value.id.to_be_bytes();

        if addr != 0 { // existing value, replace

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..16 {
                if x < 6 {
                    write_bytes[(addr + x as u32) as usize] = time_bytes[x as usize + 2];
                } else {
                    write_bytes[(addr + x as u32) as usize] = id_bytes[x as usize];
                }
            }

            return Ok(ptr.kind);

        } else { // new value

            let mut bytes: [u8; 16] = [0; 16];

            for x in 0..bytes.len() {
                if x < 6 {
                    bytes[(addr + x as u32) as usize] = time_bytes[x as usize + 2];
                } else {
                    bytes[(addr + x as u32) as usize] = id_bytes[x as usize];
                }
            }

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

        let mut time_bytes: [u8; 8] = [0; 8];
        let mut id_bytes: [u8; 16] = [0; 16];

        let read_bytes: [u8; 16] = *ptr.memory.get_16_bytes(addr).unwrap_or(&[0; 16]);

        for x in 0..read_bytes.len() {
            if x < 6 {
                time_bytes[x + 2] = read_bytes[x];
            } else {
                id_bytes[x] = read_bytes[x];
            }
        }

        Ok(Some(Box::new(NP_ULID {
            time: u64::from_be_bytes(time_bytes),
            id: u128::from_be_bytes(id_bytes)
        })))
         
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(y.to_string())
                    },
                    None => {
                        NP_JSON::Null
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
            Ok(16)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        if "ulid" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Ulid as u8);
            return Ok(Some((schema_data, NP_Parsed_Schema::Ulid { 
                i: NP_TypeKeys::Ulid,
                sortable: true
            })))
        }
        
        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }

    fn from_bytes_to_schema(_address: usize, _bytes: &Vec<u8>) -> NP_Parsed_Schema {
        NP_Parsed_Schema::Ulid {
            i: NP_TypeKeys::Ulid,
            sortable: true
        }
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"ulid\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"ulid\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.deep_set("", NP_ULID::generate(2039203, 212))?;
    assert_eq!(buffer.deep_get::<NP_ULID>("")?, Some(Box::new(NP_ULID::generate(2039203, 212))));
    buffer.deep_clear("")?;
    assert_eq!(buffer.deep_get::<NP_ULID>("")?, None);

    buffer = buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4u32);

    Ok(())
}