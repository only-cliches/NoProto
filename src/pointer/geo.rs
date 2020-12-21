//! Represents a Geographic Coordinate (lat / lon)
//! 
//! When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::geo::NP_Geo;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "geo4"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Geo::new(4, 45.509616, -122.714625))?;
//! 
//! assert_eq!("{\"lat\":45.5,\"lng\":-122.71}", new_buffer.get::<NP_Geo>(&[])?.unwrap().into_json().stringify());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 

use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::utils::to_signed;
use crate::utils::to_unsigned;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug}};
use core::convert::TryInto;

use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{string::ToString};
use super::{NP_Cursor};
use crate::NP_Memory;

/// Allows you to efficiently retrieve just the bytes of the geographic coordinate
#[derive(Debug, Eq, PartialEq)]
pub struct NP_Geo_Bytes {
    /// Size of this coordinate: 4, 8 or 16
    pub size: u8,
    /// latitude bytes
    pub lat: Vec<u8>,
    /// longitude bytes
    pub lng: Vec<u8>
}

impl super::NP_Scalar for NP_Geo_Bytes{}

impl NP_Geo_Bytes {
    /// Get the actual geographic coordinate for these bytes
    pub fn into_geo(self) -> NP_Geo {
        match self.size {
            16 => {
         
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 8]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 8]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i64::from_be_bytes(bytes_lat) as f64;
                let lon = i64::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(16);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 16}
            },
            8 => {
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 4]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 4]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(8);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 8}
            },
            4 => {
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 2]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 2]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i16::from_be_bytes(bytes_lat) as f64;
                let lon = i16::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(4);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 4}
            },
            _ => {
                NP_Geo { lat: 0f64, lng: 0f64, size: 4}
            }
        }
    }
}

impl Default for NP_Geo_Bytes {
    fn default() -> Self { 
        NP_Geo_Bytes { lat: Vec::new(), lng: Vec::new(), size: 0 }
     }
}

impl<'value> NP_Value<'value> for NP_Geo_Bytes {
    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Self> {
        None
    }
    fn type_idx() -> (&'value str, NP_TypeKeys) { NP_Geo::type_idx() }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { NP_Geo::type_idx() }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> { NP_Geo::schema_to_json(schema, address)}

    fn set_value<'set, M: NP_Memory>(_cursor: NP_Cursor, _memory: &'set M, _value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {
        Err(NP_Error::new("Can't set value with NP_Geo_Bytes, use NP_Geo instead!"))
    }
    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {
        NP_Geo::to_json(cursor, memory)
    }
    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        if c_value.get_addr_value() == 0 {
            return Ok(0) 
        } else {
            let size = match memory.get_schema(cursor.schema_addr) {
                NP_Parsed_Schema::Geo { size, ..} => {
                    *size
                },
                _ => 0
            };
            Ok(size as usize)
        }
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c_value = cursor.get_value(memory);

        let value_addr = c_value.get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        let size = match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Geo { size, .. } => {
                *size
            },
            _ => 0
        };

        Ok(Some(match size {
            16 => {
                let bytes_lat: [u8; 8] = *memory.get_8_bytes(value_addr).unwrap_or(&[0; 8]);
                let bytes_lon: [u8; 8] = *memory.get_8_bytes(value_addr + 8).unwrap_or(&[0; 8]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 16 }
            },
            8 => {
                let bytes_lat: [u8; 4] = *memory.get_4_bytes(value_addr).unwrap_or(&[0; 4]);
                let bytes_lon: [u8; 4] = *memory.get_4_bytes(value_addr + 4).unwrap_or(&[0; 4]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 8 }
            },
            4 => {
                let bytes_lat: [u8; 2] = *memory.get_2_bytes(value_addr).unwrap_or(&[0; 2]);
                let bytes_lon: [u8; 2] = *memory.get_2_bytes(value_addr + 2).unwrap_or(&[0; 2]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 4 }
            },
            _ => {
                unreachable!();
            }
        }))
    }

    fn from_json_to_schema(schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {
        NP_Geo::from_json_to_schema(schema, json_schema)
    }

    fn from_bytes_to_schema(schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        NP_Geo::from_bytes_to_schema(schema, address, bytes)
    }
}



/// Holds geographic coordinates
/// 
/// Check out documentation [here](../geo/index.html).
/// 
#[derive(Debug, Clone)]
pub struct NP_Geo {
    /// The size of this geographic coordinate.  4, 8 or 16
    pub size: u8,
    /// The latitude of this coordinate
    pub lat: f64,
    /// The longitude of this coordinate
    pub lng: f64
}

impl super::NP_Scalar for NP_Geo {}

impl NP_Geo {

    /// Create a new NP_Geo value, make sure the size matches the schema
    pub fn new(size: u8, lat: f64, lng: f64) -> Self {
        NP_Geo { size, lat, lng}
    }

    /// Get the deviser value depending on the resolution of the type in the schema
    pub fn get_deviser(size: i64) -> f64 {
        match size {
            16 => 1000000000f64,
            8 =>  10000000f64,
            4 =>  100f64,
            _ => 0.0
        }
     }

     /// Export this Geo point to JSON
     /// 
     pub fn into_json(&self) -> NP_JSON {
        let mut result_json = JSMAP::new();
        result_json.insert("lat".to_owned(), NP_JSON::Float(self.lat));
        result_json.insert("lng".to_owned(), NP_JSON::Float(self.lng));
        NP_JSON::Dictionary(result_json)
     }

     /// Get the bytes that represent this geographic coordinate
     pub fn get_bytes(&self) -> Option<NP_Geo_Bytes> {
        if self.size == 0 {
            return None
        }

        let dev = NP_Geo::get_deviser(self.size as i64);

        match self.size {
            16 => {

                let mut lat_bytes = ((self.lat * dev) as i64).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i64).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            8 => {

                let mut lat_bytes = ((self.lat * dev) as i32).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i32).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            4 => {

                let mut lat_bytes = ((self.lat * dev) as i16).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i16).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            _ => {
                None
            }
        }
     }
}

impl Default for NP_Geo {
    fn default() -> Self { 
        NP_Geo { lat: 0.0, lng: 0.0, size: 0 }
     }
}

fn geo_default_value(size: u8, json: &NP_JSON) -> Result<Option<NP_Geo_Bytes>, NP_Error> {
    match &json["default"] {
        NP_JSON::Dictionary(x) => {
            let mut lat = 0f64;
            match x.get("lat") {
                Some(x) => {
                    match x {
                        NP_JSON::Integer(y) => {
                            lat = *y as f64;
                        },
                        NP_JSON::Float(y) => {
                            lat = *y as f64;
                        },
                        _ => {}
                    }
                },
                None => {
                    return Err(NP_Error::new("Default values for NP_Geo should have lat key!"))
                }
            };
            let mut lng = 0f64;
            match x.get("lng") {
                Some(x) => {
                    match x {
                        NP_JSON::Integer(y) => {
                            lng = *y as f64;
                        },
                        NP_JSON::Float(y) => {
                            lng = *y as f64;
                        },
                        _ => {}
                    }
                },
                None => {
                    return Err(NP_Error::new("Default values for NP_Geo should have lng key!"))
                }
            };

            match NP_Geo::new(size, lat, lng).get_bytes() {
                Some(b) => return Ok(Some(b)),
                None => return Ok(None)
            }
        },
        _ => return Ok(None)
    }
}

impl<'value> NP_Value<'value> for NP_Geo {

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Self> {
        match schema {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default, size: _} => {
                if let Some(d) = default {
                    Some(d.clone())
                } else {
                    None
                }
            },
            _ => None
        }
    }

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("geo", NP_TypeKeys::Geo) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("geo", NP_TypeKeys::Geo) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();

        match &schema[address] {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default, size} => {
                let mut type_str = Self::type_idx().0.to_string();
                type_str.push_str(size.to_string().as_str());
                schema_json.insert("type".to_owned(), NP_JSON::String(type_str));
            
                if let Some(d) = default {
                    let mut default_map = JSMAP::new();
                    default_map.insert("lat".to_owned(), NP_JSON::Float(d.lat));
                    default_map.insert("lng".to_owned(), NP_JSON::Float(d.lng));
                    schema_json.insert("default".to_owned(), NP_JSON::Dictionary(default_map));
                }
        
                Ok(NP_JSON::Dictionary(schema_json))
            },
            _ => Err(NP_Error::new("unreachable"))
        }


    }

    fn set_value<'set, M: NP_Memory>(cursor: NP_Cursor, memory: &'set M, value: Self) -> Result<NP_Cursor, NP_Error> where Self: 'set + Sized {

        let c_value = cursor.get_value(memory);

        let size = match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Geo { size, .. } => {
                *size
            },
            _ => 0
        };

        let value_bytes_size = size as usize;

        if value_bytes_size == 0 {
            unreachable!();
        }

        let write_bytes: &mut [u8];

        let half_value_bytes = value_bytes_size / 2;

        // convert input values into bytes
        let value_bytes = match size {
            16 => {
                let dev = NP_Geo::get_deviser(16);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i64).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i64).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

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
                let dev = NP_Geo::get_deviser(8);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i32).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i32).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

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
                let dev = NP_Geo::get_deviser(4);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i16).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i16).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

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

        let mut value_address = c_value.get_addr_value() as usize;

        if value_address != 0 { // existing value, replace

            write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[value_address + x] = value_bytes[x];
                }
            }


        } else { // new value

            value_address = match size {
                16 => { memory.malloc_borrow(&[0u8; 16])? },
                8 => { memory.malloc_borrow(&[0u8; 8])? },
                4 => { memory.malloc_borrow(&[0u8; 4])? },
                _ => { 0 }
            };

            write_bytes = memory.write_bytes();

            // set values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[value_address + x] = value_bytes[x];
                }
            }

            c_value.set_addr_value(value_address as u16);

        }

        Ok(cursor)
    }

    fn into_value<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> Result<Option<Self>, NP_Error> where Self: Sized {

        let c_value = cursor.get_value(memory);

        let value_addr = c_value.get_addr_value() as  usize;

        // empty value
        if value_addr == 0 {
            return Ok(None);
        }
    
        let size = match memory.get_schema(cursor.schema_addr) {
            NP_Parsed_Schema::Geo { size, .. } => {
                *size
            },
            _ => 0
        };

        Ok(Some(match size {
            16 => {
         
                let mut bytes_lat: [u8; 8] = *memory.get_8_bytes(value_addr).unwrap_or(&[0; 8]);
                let mut bytes_lon: [u8; 8] = *memory.get_8_bytes(value_addr + 8).unwrap_or(&[0; 8]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i64::from_be_bytes(bytes_lat) as f64;
                let lon = i64::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(16);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 16}
            },
            8 => {
                let mut bytes_lat: [u8; 4] = *memory.get_4_bytes(value_addr).unwrap_or(&[0; 4]);
                let mut bytes_lon: [u8; 4] = *memory.get_4_bytes(value_addr + 4).unwrap_or(&[0; 4]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(8);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 8}
            },
            4 => {
                let mut bytes_lat: [u8; 2] = *memory.get_2_bytes(value_addr).unwrap_or(&[0; 2]);
                let mut bytes_lon: [u8; 2] = *memory.get_2_bytes(value_addr + 2).unwrap_or(&[0; 2]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i16::from_be_bytes(bytes_lat) as f64;
                let lon = i16::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(4);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 4}
            },
            _ => {
                unreachable!();
            }
        }))
    }

    fn to_json<M: NP_Memory>(cursor: &NP_Cursor, memory: &'value M) -> NP_JSON {

        match Self::into_value(cursor, memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JSMAP::new();

                        object.insert("lat".to_owned(), NP_JSON::Float(y.lat));
                        object.insert("lng".to_owned(), NP_JSON::Float(y.lng));
                        
                        NP_JSON::Dictionary(object)
                    },
                    None => {

                        match &memory.get_schema(cursor.schema_addr) {
                            NP_Parsed_Schema::Geo { i: _, sortable: _, default, size: _} => {
                                if let Some(d) = default {
                                    let mut object = JSMAP::new();

                                    object.insert("lat".to_owned(), NP_JSON::Float(d.lat));
                                    object.insert("lng".to_owned(), NP_JSON::Float(d.lng));
                                    
                                    NP_JSON::Dictionary(object)
                                } else {
                                    NP_JSON::Null
                                }
                            },
                            _ => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size<M: NP_Memory>(cursor: &NP_Cursor, memory: &M) -> Result<usize, NP_Error> {

        let c_value = cursor.get_value(memory);

        let value_addr = c_value.get_addr_value();

        if value_addr == 0 {
            return Ok(0) 
        } else {
            let size = match memory.get_schema(cursor.schema_addr) {
                NP_Parsed_Schema::Geo { i: _, sortable: _, default: _, size} => {
                    *size
                },
                _ => 0
            };
            Ok(size as usize)
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let type_str = NP_Schema::_get_type(&json_schema)?;

        match type_str.as_str() {
            "geo4" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(4);
                let default = match geo_default_value(4, &json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo();
                        Some(NP_Geo::new(4, g.lat, g.lng))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 4,
                    default: default,
                    sortable: false
                });
                Ok((false, schema_data, schema))
            },
            "geo8" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(8);
                let default = match geo_default_value(8, &json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo();
                        Some(NP_Geo::new(8, g.lat, g.lng))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 8,
                    default: default,
                    sortable: false
                });
                Ok((false, schema_data, schema))
            },
            "geo16" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(16);
                let default = match geo_default_value(16, &json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo();
                        Some(NP_Geo::new(16, g.lat, g.lng))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 16,
                    default: default,
                    sortable: false
                });
                Ok((false, schema_data, schema))
            },
            _ => {
                Ok((false, Vec::new(), Vec::new()))
            }
        }
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &[u8]) -> (bool, Vec<NP_Parsed_Schema>) {
        let size = bytes[address + 1];

        // no default
        if bytes[address + 2] == 0 {
            schema.push(NP_Parsed_Schema::Geo {
                i: NP_TypeKeys::Geo,
                sortable: false,
                size: size,
                default: None
            });
            return (false, schema) 
        }

        // has default
        match size {
            4 => {
                let lat = &bytes[(address + 3)..(address + 5)];
                let lng = &bytes[(address + 6)..(address + 8)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(default_value.into_geo())
                });
                (false, schema)
            },
            8 => {
                let lat = &bytes[(address + 3)..(address + 7)];
                let lng = &bytes[(address + 7)..(address + 11)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(default_value.into_geo())
                });
                (false, schema)
            },
            16 => {
                let lat = &bytes[(address + 3)..(address + 11)];
                let lng = &bytes[(address + 12)..(address + 20)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                schema.push(NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(default_value.into_geo())
                });
                (false, schema)
            },
            _ => {
                unreachable!();
            }
        }
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"geo4\",\"default\":{\"lat\":20.23,\"lng\":-12.21}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"geo4\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"geo8\",\"default\":{\"lat\":20.2334234,\"lng\":-12.2146363}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"geo8\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"geo16\",\"default\":{\"lat\":20.233423434,\"lng\":-12.214636323}}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"geo16\"}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}


#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"geo4\",\"default\":{\"lat\":20.23,\"lng\":-12.21}}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!((buffer.get::<NP_Geo>(&[])?.unwrap()).get_bytes().unwrap(), NP_Geo::new(4, 20.23, -12.21).get_bytes().unwrap());

    let schema = "{\"type\":\"geo8\",\"default\":{\"lat\":20.2334234,\"lng\":-12.2146363}}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!((buffer.get::<NP_Geo>(&[])?.unwrap()).get_bytes().unwrap(), NP_Geo::new(8, 20.2334234, -12.2146363).get_bytes().unwrap());

    let schema = "{\"type\":\"geo16\",\"default\":{\"lat\":20.233423434,\"lng\":-12.214636323}}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!((buffer.get::<NP_Geo>(&[])?.unwrap()).get_bytes().unwrap(), NP_Geo::new(16, 20.233423434, -12.214636323).get_bytes().unwrap());

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"geo4\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], NP_Geo::new(4, 20.23, -12.21))?;
    assert_eq!((buffer.get::<NP_Geo>(&[])?.unwrap()).get_bytes().unwrap(), NP_Geo::new(4, 20.23, -12.21).get_bytes().unwrap());
    buffer.del(&[])?;
    assert!({
        match buffer.get::<NP_Geo>(&[])? {
            Some(_x) => false,
            None => true
        }
    });

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 3usize);

    Ok(())
}