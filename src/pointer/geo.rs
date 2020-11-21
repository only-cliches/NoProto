//! Represents a Geographic Coordinate (lat / lon)
//! 
//! When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::geo::NP_Geo;
//! use no_proto::here;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "geo4"
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None, None);
//! new_buffer.set(here(), NP_Geo::new(4, 45.509616, -122.714625))?;
//! 
//! assert_eq!("{\"lat\":45.5,\"lng\":-122.71}", new_buffer.get::<NP_Geo>(here())?.unwrap().into_json().stringify());
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
use core::{fmt::{Debug}, hint::unreachable_unchecked};
use core::convert::TryInto;

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{string::ToString};

use super::NP_Ptr;


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

impl NP_Geo_Bytes {
    /// Get the actual geographic coordinate for these bytes
    pub fn into_geo(self) -> Option<NP_Geo> {
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

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 16})
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

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 8})
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

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 4})
            },
            _ => {
                None
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
    fn schema_default(_schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        None
    }
    fn type_idx() -> (u8, String, NP_TypeKeys) { NP_Geo::type_idx() }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { NP_Geo::type_idx() }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> { NP_Geo::schema_to_json(&schema_ptr)}

    fn set_value(_ptr: &mut NP_Ptr<'value>, _value: Box<&Self>) -> Result<(), NP_Error> {
        Err(NP_Error::new("Can't set value with NP_Geo_Bytes, use NP_Geo instead!"))
    }
    fn to_json(ptr: &'value NP_Ptr<'value>) -> NP_JSON {
        NP_Geo::to_json(ptr)
    }
    fn get_size(ptr:  &'value NP_Ptr<'value>) -> Result<usize, NP_Error> {
        NP_Geo::get_size(ptr)
    }
    fn into_value<'into>(ptr: &'into NP_Ptr<'into>) -> Result<Option<Box<Self>>, NP_Error> {

        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let size = match &**ptr.schema {
            NP_Parsed_Schema::Geo { i: _, size, default: _, sortable: _ } => {
                size
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        Ok(Some(Box::new(match size {
            16 => {
                let bytes_lat: [u8; 8] = *ptr.memory.get_8_bytes(addr).unwrap_or(&[0; 8]);
                let bytes_lon: [u8; 8] = *ptr.memory.get_8_bytes(addr + 8).unwrap_or(&[0; 8]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 16 }
            },
            8 => {
                let bytes_lat: [u8; 4] = *ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
                let bytes_lon: [u8; 4] = *ptr.memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 8 }
            },
            4 => {
                let bytes_lat: [u8; 2] = *ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2]);
                let bytes_lon: [u8; 2] = *ptr.memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 4 }
            },
            _ => {
                unreachable!();
            }
        })))
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {
        NP_Geo::from_json_to_schema(json_schema)
    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema {
        NP_Geo::from_bytes_to_schema(address, bytes)
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

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Box<Self>> {
        match schema {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default, size: _} => {
                if let Some(d) = default {
                    Some(Box::new(*d.clone()))
                } else {
                    None
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn type_idx() -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Geo as u8, "geo".to_owned(), NP_TypeKeys::Geo) }
    fn self_type_idx(&self) -> (u8, String, NP_TypeKeys) { (NP_TypeKeys::Geo as u8, "geo".to_owned(), NP_TypeKeys::Geo) }

    fn schema_to_json(schema_ptr: &NP_Parsed_Schema)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();

        match schema_ptr {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default, size} => {
                let mut type_str = Self::type_idx().1;
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
            _ => { unsafe { unreachable_unchecked() } }
        }


    }

    fn set_value(ptr: &mut NP_Ptr<'value>, value: Box<&Self>) -> Result<(), NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let size = match &**ptr.schema {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default: _, size} => {
                size
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        let value_bytes_size = *size as usize;

        if value_bytes_size == 0 {
            unreachable!();
        }

        let write_bytes = ptr.memory.write_bytes();

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

        if addr != 0 { // existing value, replace

            // overwrite existing values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[addr + x] = value_bytes[x];
                }
            }

            return Ok(());

        } else { // new value

            addr = match size {
                16 => { ptr.memory.malloc([0; 16].to_vec())? },
                8 => { ptr.memory.malloc([0; 8].to_vec())? },
                4 => { ptr.memory.malloc([0; 4].to_vec())? },
                _ => { 0 }
            };

            // set values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[addr + x] = value_bytes[x];
                }
            }

            ptr.kind = ptr.memory.set_value_address(ptr.address, addr, &ptr.kind);

            return Ok(());
        }
        
    }

    fn into_value<'into>(ptr: &'into NP_Ptr<'into>) -> Result<Option<Box<Self>>, NP_Error> {

        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let size = match &**ptr.schema {
            NP_Parsed_Schema::Geo { i: _, sortable: _, default: _, size} => {
                size
            },
            _ => { unsafe { unreachable_unchecked() } }
        };

        Ok(Some(Box::new(match size {
            16 => {
         
                let mut bytes_lat: [u8; 8] = *ptr.memory.get_8_bytes(addr).unwrap_or(&[0; 8]);
                let mut bytes_lon: [u8; 8] = *ptr.memory.get_8_bytes(addr + 8).unwrap_or(&[0; 8]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i64::from_be_bytes(bytes_lat) as f64;
                let lon = i64::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(16);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 16}
            },
            8 => {
                let mut bytes_lat: [u8; 4] = *ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
                let mut bytes_lon: [u8; 4] = *ptr.memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(8);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 8}
            },
            4 => {
                let mut bytes_lat: [u8; 2] = *ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2]);
                let mut bytes_lon: [u8; 2] = *ptr.memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2]);

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
        })))
    }

    fn to_json(ptr: &'value NP_Ptr<'value>) -> NP_JSON {
        let this_value = Self::into_value(ptr);

        match this_value {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JSMAP::new();

                        object.insert("lat".to_owned(), NP_JSON::Float(y.lat));
                        object.insert("lng".to_owned(), NP_JSON::Float(y.lng));
                        
                        NP_JSON::Dictionary(object)
                    },
                    None => {
                        match &**ptr.schema {
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
                            _ => { unsafe { unreachable_unchecked() } }
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: &'value NP_Ptr<'value>) -> Result<usize, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            let size = match &**ptr.schema {
                NP_Parsed_Schema::Geo { i: _, sortable: _, default: _, size} => {
                    size
                },
                _ => { unsafe { unreachable_unchecked() } }
            };
            Ok(*size as usize)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<(Vec<u8>, NP_Parsed_Schema)>, NP_Error> {

        let type_str = NP_Schema::_get_type(json_schema)?;

        match type_str.as_str() {
            "geo4" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(4);
                let default = match geo_default_value(4, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo().unwrap();
                        Some(Box::new(NP_Geo::new(4, g.lat, g.lng)))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                Ok(Some((schema_data, NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 4,
                    default: default,
                    sortable: false
                })))
            },
            "geo8" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(8);
                let default = match geo_default_value(8, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo().unwrap();
                        Some(Box::new(NP_Geo::new(8, g.lat, g.lng)))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                Ok(Some((schema_data, NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 8,
                    default: default,
                    sortable: false
                })))
            },
            "geo16" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(16);
                let default = match geo_default_value(16, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat.clone());
                        schema_data.extend(x.lng.clone());
                        let g = x.into_geo().unwrap();
                        Some(Box::new(NP_Geo::new(16, g.lat, g.lng)))
                    },
                    None => {
                        schema_data.push(0);
                        None
                    }
                };
                Ok(Some((schema_data, NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: 16,
                    default: default,
                    sortable: false
                })))
            },
            _ => {
                Ok(None)
            }
        }
    }

    fn from_bytes_to_schema(address: usize, bytes: &Vec<u8>) -> NP_Parsed_Schema { 
        let size = bytes[address + 1];

        if bytes[address + 2] == 0 {
            return NP_Parsed_Schema::Geo {
                i: NP_TypeKeys::Geo,
                sortable: false,
                size: size,
                default: None
            }
        }

        match size {
            4 => {
                let lat = &bytes[(address + 3)..(address + 5)];
                let lng = &bytes[(address + 6)..(address + 8)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(Box::new(default_value.into_geo().unwrap()))
                }
            },
            8 => {
                let lat = &bytes[(address + 3)..(address + 7)];
                let lng = &bytes[(address + 7)..(address + 11)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(Box::new(default_value.into_geo().unwrap()))
                }
            },
            16 => {
                let lat = &bytes[(address + 3)..(address + 11)];
                let lng = &bytes[(address + 12)..(address + 20)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Parsed_Schema::Geo {
                    i: NP_TypeKeys::Geo,
                    size: size,
                    sortable: false,
                    default: Some(Box::new(default_value.into_geo().unwrap()))
                }
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
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!((*buffer.get::<NP_Geo>(crate::here())?.unwrap()).get_bytes().unwrap(), NP_Geo::new(4, 20.23, -12.21).get_bytes().unwrap());

    let schema = "{\"type\":\"geo8\",\"default\":{\"lat\":20.2334234,\"lng\":-12.2146363}}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!((*buffer.get::<NP_Geo>(crate::here())?.unwrap()).get_bytes().unwrap(), NP_Geo::new(8, 20.2334234, -12.2146363).get_bytes().unwrap());

    let schema = "{\"type\":\"geo16\",\"default\":{\"lat\":20.233423434,\"lng\":-12.214636323}}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    assert_eq!((*buffer.get::<NP_Geo>(crate::here())?.unwrap()).get_bytes().unwrap(), NP_Geo::new(16, 20.233423434, -12.214636323).get_bytes().unwrap());

    Ok(())
}

#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"geo4\"}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None, None);
    buffer.set(crate::here(), NP_Geo::new(4, 20.23, -12.21))?;
    assert_eq!((*buffer.get::<NP_Geo>(crate::here())?.unwrap()).get_bytes().unwrap(), NP_Geo::new(4, 20.23, -12.21).get_bytes().unwrap());
    buffer.del(crate::here())?;
    assert!({
        match buffer.get::<NP_Geo>(crate::here())? {
            Some(_x) => false,
            None => true
        }
    });

    buffer.compact(None, None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 4usize);

    Ok(())
}