use alloc::vec::Vec;
use crate::schema::NP_Schema;
use crate::error::NP_Error;
use crate::memory::{NP_Size};
use crate::{schema::NP_TypeKeys, pointer::NP_Value, utils::from_utf8_lossy, json_flex::NP_JSON};
use super::{NP_PtrKinds, NP_Lite_Ptr, bytes::NP_Bytes};

use alloc::string::String;
use alloc::boxed::Box;
use alloc::{rc::Rc, borrow::ToOwned};


impl NP_Value for String {

    fn is_type( type_str: &str) -> bool {
        "string" == type_str || "str" == type_str || "utf8" == type_str || "utf-8" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::UTF8String as i64, "string".to_owned()) }

    fn schema_state(type_string: &str, json_schema: &NP_JSON) -> core::result::Result<i64, NP_Error> {
        NP_Bytes::schema_state(type_string, json_schema)
    }

    fn set_value(pointer: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        let bytes = value.as_bytes().to_vec();
        NP_Bytes::set_value(pointer, Box::new(&NP_Bytes::new(bytes)))
    }

    fn into_value(pointer: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        match NP_Bytes::into_value(pointer)? {
            Some(x) => {
                let bytes = &*x.bytes.to_vec();
                Ok(Some(Box::new(from_utf8_lossy(bytes))))
            },
            None => Ok(None)
        }
    }

    fn schema_default(schema: Rc<NP_Schema>) -> Option<Box<Self>> {
        match &schema.default {
            Some(x) => {
                match x {
                    NP_JSON::String(value) => {
                        Some(Box::new(value.clone()))
                    },
                    _ => {
                        None
                    }
                }
            },
            None => {
                None
            }
        }
    }

    fn to_json(pointer: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(pointer.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(*y)
                    },
                    None => {
                        match &pointer.schema.default {
                            Some(x) => x.clone(),
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(pointer: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        NP_Bytes::get_size(pointer)
    }
}