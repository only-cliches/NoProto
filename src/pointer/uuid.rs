//! Represents a V4 UUID, good for globally unique identifiers
//! 
//! `uuid` types are always represented with this struct.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::uuid::NP_UUID;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "uuid"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.deep_set("", NP_UUID::generate(50))?;
//! 
//! assert_eq!("48E6AAB0-7DF5-409F-4D57-4D969FA065EE", new_buffer.deep_get::<NP_UUID>("")?.unwrap().to_string());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::pointer::NP_PtrKinds;
use crate::{pointer::NP_Value, error::NP_Error, utils::{Rand}};
use core::{fmt::{Debug, Formatter, Write}};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use super::NP_Lite_Ptr;


/// Holds UUID which is good for random keys.
/// 
/// Check out documentation [here](../uuid/index.html).
/// 
#[derive(Eq, PartialEq)]
pub struct NP_UUID {
    /// The random bytes for this UUID
    pub value: [u8; 16]
}

impl NP_UUID {

    /// Generate a new UUID with a given random seed.  You should attempt to provide a seed with as much randomness as possible.
    /// 
    pub fn generate(random_seed: u32) -> NP_UUID {


        let mut uuid = NP_UUID {
            value: [0; 16]
        };

        let mut rng = Rand::new(random_seed);

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + rng.gen_range(0, 15) as u8;
            } else {
                uuid.value[x] = rng.gen_range(0, 255) as u8;
            }
        }

        uuid
    }

    /// Generates a UUID with a provided random number generator.
    /// This is the preferrable way to generate a ULID, if you can provide a better RNG function than the psudorandom one built into this library, you should.
    /// 
    pub fn generate_with_rand<F>(random_fn: F) -> NP_UUID where F: Fn(u8, u8) -> u8 {
        let mut uuid = NP_UUID {
            value: [0; 16]
        };

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + random_fn(0, 15) as u8;
            } else {
                uuid.value[x] = random_fn(0, 255) as u8;
            }
        }

        uuid
    }

    /// Generates a stringified version of the UUID.
    /// 
    pub fn to_string(&self) -> String {

        let mut result = String::with_capacity(32);

        for x in 0..self.value.len() {
            if x == 4 || x == 6 || x == 8 || x == 10 {
                result.push_str("-");
            }
            let byte = self.value[x] as u8;
            write!(result, "{:02X}", byte).unwrap();
            // result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl Debug for NP_UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for NP_UUID {
    fn default() -> Self { 
        NP_UUID { value: [0; 16] }
     }
}

impl<'value> NP_Value<'value> for NP_UUID {

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Uuid as u8, "uuid".to_owned(), NP_TypeKeys::Uuid) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Uuid as u8, "uuid".to_owned(), NP_TypeKeys::Uuid) }

    fn schema_to_json(_schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = value.value;
            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);

        } else { // new value

            let bytes = value.value;
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

        let memory = ptr.memory;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                // copy since we're handing owned value outside the library
                let mut bytes: [u8; 16] = [0; 16];
                bytes.copy_from_slice(x);
                Some(Box::new(NP_UUID { value: bytes}))
            },
            None => None
        })
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

        if "uuid" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Uuid as u8);
            return Ok(Some((schema_data, NP_Parsed_Schema::Uuid { 
                i: NP_TypeKeys::Uuid,
                sortable: true
            })))
        }
        
        Ok(None)
    }

    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }

    fn from_bytes_to_schema(_address: usize, _bytes: &Vec<u8>) -> NP_Parsed_Schema {
        NP_Parsed_Schema::Uuid {
            i: NP_TypeKeys::Uuid,
            sortable: true
        }
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uuid\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}



#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"uuid\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.deep_set("", NP_UUID::generate(212))?;
    assert_eq!(buffer.deep_get::<NP_UUID>("")?, Some(Box::new(NP_UUID::generate(212))));
    buffer.deep_clear("")?;
    assert_eq!(buffer.deep_get::<NP_UUID>("")?, None);

    buffer = buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4u32);

    Ok(())
}