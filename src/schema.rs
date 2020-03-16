//! Schemas are JSON used to declare & store the shape of buffer objects.
//! 
//! No Proto Schemas are JSON objects that describe how the data in a NoProto buffer is stored.
//! 
//! Every schema object has at least a "type" key that provides the kind of value stored at that part of the schema.  Additional keys are dependent on the type of schema.
//! 
//! Schemas are validated and sanity checked by the [NoProtoFactory](../struct.NoProtoFactory.html) struct upon creation.  You cannot pass an invalid schema into a factory constructor and build/parse buffers with it.
//! 
//! If you're familiar with typescript, schemas can be described by this interface:
//! ```text
//! interface InoProtoSchema {
//!     type: string;
//!     
//!     // used by list types
//!     of?: InoProtoSchema
//!     
//!     // used by map types
//!     value?: InoProtoSchema
//! 
//!     // used by tuple types
//!     values?: InoProtoSchema[]
//! 
//!     // used by table types
//!     columns?: [string, InoProtoSchema][]
//! }
//! ```
//! 
//! Schemas do not have to contain collections, for example a perfectly valid schema for just a string would be:
//! ```text
//! {
//!     "type": "string"
//! }
//! ```
//! 
//! However, nesting is easy to perform.  For example, this  is a list of tables.  Each table has two columns: id and title.  Both columns are a string type.
//! ```text
//! {
//!     "type": "list",
//!     "of": {
//!         "type": "table",
//!         "columns": [
//!             ["id",    {type: "string"}]
//!             ["title", {type: "string"}]
//!         ]
//!     }
//! }
//! ```
//! 
//! A list of strings is just as easy...
//! 
//! ```text
//! {
//!     "type": "list",
//!     "of": { type: "string" }
//! }
//! ```
//! 
//! Each type has trade offs associated with it.  The table and documentation below go into further detail.
//! 
//! Here is a table of supported types. 
//! 
//! | Type                | Rust Type / Struct                                                       | Bytes (Size)   | Limits / Notes                                                           |
//! |---------------------|--------------------------------------------------------------------------|----------------|--------------------------------------------------------------------------|
//! | [`table`](#table)   | [`NoProtoTable`](../collection/table/index.html)                         | 4 bytes - ~4GB | Linked list with indexed keys that map against up to 255 named columns.  |
//! | [`list`](#list)     | [`NoProtoList`](../collection/list/index.html)                           | 8 bytes - ~4GB | Linked list with up to 65,535 items.                                     |
//! | [`map`](#map)       | [`NoProtoMap`](../collection/map/index.html)                             | 4 bytes - ~4GB | Linked list with Vec<u8> keys.                                           |
//! | [`tuple`](#tuple)   | [`NoProtoTuple`](../collection/tuple/index.html)                         | 4 bytes - ~4GB | Static sized collection of values.                                       |
//! | [`string`](#string) | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      | 4 bytes - ~4GB | Utf-8 formatted string.                                                  |
//! | [`bytes`](#bytes)   | [`NoProtoBytes`](https://doc.rust-lang.org/std/vec/struct.Vec.html)      | 4 bytes - ~4GB | Arbitrary bytes.                                                         |
//! | [`int8`](#int8)     | [`i8`](https://doc.rust-lang.org/std/primitive.i8.html)                  | 1 byte         | -127 to 127                                                              |
//! | [`int16`](#int16)   | [`i16`](https://doc.rust-lang.org/std/primitive.i16.html)                | 2 bytes        | -32,768 to 32,768                                                        |
//! | [`int32`](#int32)   | [`i32`](https://doc.rust-lang.org/std/primitive.i32.html)                | 4 bytes        | -2,147,483,648 to 2,147,483,648                                          |
//! | [`int64`](#int64)   | [`i64`](https://doc.rust-lang.org/std/primitive.i64.html)                | 8 bytes        | -9.22e18 to 9.22e18                                                      |
//! | [`uint8`](#uint8)   | [`u8`](https://doc.rust-lang.org/std/primitive.u8.html)                  | 1 byte         | 0 - 255                                                                  |
//! | [`uint16`](#uint16) | [`u16`](https://doc.rust-lang.org/std/primitive.u16.html)                | 2 bytes        | 0 - 65,535                                                               |
//! | [`uint32`](#uint32) | [`u32`](https://doc.rust-lang.org/std/primitive.u32.html)                | 4 bytes        | 0 - 4,294,967,295                                                        |
//! | [`uint64`](#uint64) | [`u64`](https://doc.rust-lang.org/std/primitive.u64.html)                | 8 bytes        | 0 - 1.84e19                                                              |
//! | [`float`](#float)   | [`f32`](https://doc.rust-lang.org/std/primitive.f32.html)                | 4 bytes        | -3.4e38 to 3.4e38                                                        |
//! | [`double`](#double) | [`f64`](https://doc.rust-lang.org/std/primitive.f64.html)                | 8 bytes        | -1.7e308 to 1.7e308                                                      |
//! | [`option`](#option) | [`NoProtoOption`](https://doc.rust-lang.org/std/string/.html)            | 1 byte         | Up to 255 strings in schema.                                             |
//! | [`bool`](#bool)     | [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)              | 1 byte         |                                                                          |
//! | [`dec64`](#dec64)   | [`NoProtoDec`](..pointer/struct.NoProtoDec.html)                         | 9 bytes        | Big Integer Decimal format.                                              |
//! | [`geo4`](#geo4)     | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 4 bytes        | 1.5km resolution (city) geographic coordinate                            |
//! | [`geo8`](#geo8)     | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 8 bytes        | 16mm resolution (marble) geographic coordinate                           |
//! | [`geo16`](#geo16)   | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 16 bytes       | 3.5nm resolution (flea) geographic coordinate                            |
//! | [`tid`](#tid)       | [`NoProtoTimeID`](../pointer/struct.NoProtoTimeID.html)                  | 16 bytes       | 8 byte u64 for time with 8 bytes of random numbers.                      |
//! | [`uuid`](#uuid)     | [`NoProtoUUID`](../pointer/struct.NoProtoUUID.html)                      | 16 bytes       | v4 UUID, 2e37 possible UUID v4s                                          |
//! | [`date`](#date)     | [`NoProtoDate`](https://doc.rust-lang.org/std/primitive.u64.html)        | 8 bytes        | Good to store unix epoch (in seconds) until the year 584,942,417,355     |
//!  
//! # table
//! 
//! # list
//! 
//! # map
//! 
//! # tuple
//! 
//! # string
//! 
//! # bytes
//! 
//! # int8, int16, int32, int64
//! 
//! # uint8, uint16, uint32, uint64
//! 
//! # float, double
//! 
//! # option
//! 
//! # bool
//! 
//! # dec64
//! 
//! # geo4, ge8, geo16
//! 
//! # tid
//! 
//! # uuid
//! 
//! # date
//! 
//!  
use crate::pointer::any::NoProtoAny;
use crate::pointer::misc::NoProtoDate;
use crate::pointer::misc::NoProtoUUID;
use crate::pointer::misc::NoProtoTimeID;
use crate::pointer::misc::NoProtoGeo;
use crate::pointer::misc::NoProtoDec;
use crate::collection::tuple::NoProtoTuple;
use crate::pointer::bytes::NoProtoBytes;
use crate::collection::{list::NoProtoList, table::NoProtoTable, map::NoProtoMap};
use crate::pointer::{misc::NoProtoOption, NoProtoValue};
use json::*;
use crate::error::NoProtoError;


pub enum NoProtoSchemaKinds {
    None,
    Scalar,
    Table { columns: Vec<Option<(u8, String, NoProtoSchema)>> },
    List { of: NoProtoSchema },
    Map { value: NoProtoSchema },
    Enum { choices: Vec<String> },
    Tuple { values: Vec<NoProtoSchema>}
}

/*
#[derive(Debug)]
pub enum NoProtoSchemaKinds {
    None,
    Utf8String,
    Bytes,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Float,
    Double,
    Dec64,
    Boolean,
    Geo4,
    Geo8,
    Geo16,
    Uuid,
    Tid,
    Date,
    Table { columns: Vec<Option<(u8, String, NoProtoSchema)>> },
    List { of: NoProtoSchema },
    Map { value: NoProtoSchema },
    Enum { choices: Vec<String> },
    Tuple { values: Vec<NoProtoSchema>}
}
*/

/*
const VALID_KINDS_COLLECTIONS: [&str; 4] = [
    "table",
    "map",
    "list",
    "tuple",
];


const VALID_KINDS_SCALAR: [&str; 21] = [
    "string",
    "bytes",
    "int8",
    "int16",
    "int32",
    "int64",
    "uint8",
    "uint16",
    "uint32",
    "uint64",
    "float",
    "double",
    "option",
    "dec64",
    "boolean",
    "geo4",
    "geo8",
    "geo16",
    "uuid",
    "tid",
    "date"
];
*/

// These are just used for runtime type comparison, the type information is never stored in the buffer.
// When you cast a pointer to some type, this enum is used as comparing numbers is very efficient.
pub enum NoProtoTypeKeys {
    Any = 0,
    UTF8String = 1,
    Bytes = 2,
    Int8 = 3,
    Int16 = 4,
    Int32 = 5,
    Int64 = 6,
    Uint8 = 7,
    Uint16 = 8,
    Uint32 = 9,
    Uint64 = 10,
    Float = 11,
    Double = 12,
    Dec64 = 13,
    Boolean = 14,
    Geo = 15,
    Uuid = 16,
    Tid = 17,
    Date = 18,
    Enum = 19,
    Table = 20,
    Map = 21, 
    List = 22,
    Tuple = 23
}

pub struct NoProtoSchema {
    pub kind: Box<NoProtoSchemaKinds>,
    pub type_data: (i64, String),
    pub type_state: i64
}

pub struct NoProtoTypes { }

impl<'a> NoProtoTypes {
    pub fn do_check<T: NoProtoValue<'a> + Default>(type_string: &str, json_schema: &JsonValue)-> std::result::Result<Option<NoProtoSchema>, NoProtoError>{
        if T::is_type(type_string) {
            Ok(Some(NoProtoSchema { 
                kind: Box::new(NoProtoSchemaKinds::Scalar),
                type_data: T::type_idx(),
                type_state: T::schema_state(type_string, json_schema)?
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_type(type_string: &str, json_schema: &JsonValue)-> std::result::Result<NoProtoSchema, NoProtoError> {

        let check = NoProtoTypes::do_check::<NoProtoAny>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NoProtoTypes::do_check::<String>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<NoProtoBytes>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<i8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<i16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<i32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<i64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<u8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<u16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<u32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<u64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<f32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<f64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<bool>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<NoProtoDec>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<NoProtoGeo>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<NoProtoTimeID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NoProtoTypes::do_check::<NoProtoUUID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NoProtoTypes::do_check::<NoProtoDate>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        Err(NoProtoError::new(format!("{} is not a valid type!", type_string).as_str()))
    }
}

/*
pub fn get_standard_types() -> Vec<Box<NoProtoValue>>  {

    
    vec![
        Box::new(NoProtoAny::default()),
        NoProtoValue::new::<String>(),
        NoProtoValue::new::<NoProtoBytes>(),
        NoProtoValue::new::<i8>(),
        NoProtoValue::new::<i16>(),
        NoProtoValue::new::<i32>(),
        NoProtoValue::new::<i64>(),
        NoProtoValue::new::<i64>(),
        NoProtoValue::new::<u8>(),
        NoProtoValue::new::<u16>(),
        NoProtoValue::new::<u32>(),
        NoProtoValue::new::<u64>(),
        NoProtoValue::new::<u64>(),
        NoProtoValue::new::<f32>(),
        NoProtoValue::new::<f64>(),
        NoProtoValue::new::<bool>(),
        NoProtoValue::new::<NoProtoDec>(),
        NoProtoValue::new::<NoProtoGeo>(),
        NoProtoValue::new::<NoProtoTimeID>(),
        NoProtoValue::new::<NoProtoUUID>(),
        NoProtoValue::new::<NoProtoDate>()
    ]
}
*/

impl NoProtoSchema {

    pub fn blank() -> NoProtoSchema {

        NoProtoSchema {
            kind: Box::new(NoProtoSchemaKinds::None),
            type_data: (-1, "".to_owned()),
            type_state: 0
        }
    }

    pub fn from_json(json: JsonValue) -> std::result::Result<Self, NoProtoError> {
        NoProtoSchema::validate_model(&json)
    }

    pub fn validate_model(json_schema: &JsonValue) -> std::result::Result<Self, NoProtoError> {

        let type_string = json_schema["type"].as_str().unwrap_or("");

        if type_string.len() == 0 {
            return Err(NoProtoError::new("Must declare a type for every schema!"));
        }


        // validate required properties are in place for each kind
        match type_string {
            "table" => {
                
                let mut columns: Vec<Option<(u8, String, NoProtoSchema)>> = vec![];

                for _x in 0..255 {
                    columns.push(None);
                }

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["columns"].is_null() || borrowed_schema["columns"].is_array() == false {
                        return Err(NoProtoError::new("Table kind requires 'columns' property as array!"));
                    }

                    let mut index = 0;
                    for column in borrowed_schema["columns"].members() {

                        let column_name = &column[0].to_string();

                        if column_name.len() == 0 {
                            return Err(NoProtoError::new("Table kind requires all columns have a name!"));
                        }

                        let good_schema = NoProtoSchema::validate_model(&column[1])?;

                        let this_index = &column[1]["i"];

                        let use_index = if this_index.is_null() || this_index.is_number() == false {
                            index
                        } else {
                            this_index.as_usize().unwrap_or(index)
                        };

                        if use_index > 255 {
                            return Err(NoProtoError::new("Table cannot have column index above 255!"));
                        }

                        match &columns[use_index] {
                            Some(_x) => {
                                return Err(NoProtoError::new("Table column index numbering conflict!"));
                            },
                            None => {
                                columns[use_index] = Some((use_index as u8, column_name.to_string(), good_schema));
                            }
                        };

                        index += 1;
                    }
                }

                Ok(NoProtoSchema {
                    kind: Box::new(NoProtoSchemaKinds::Table { 
                        columns: columns 
                    }),
                    type_data: NoProtoTable::type_idx(),
                    type_state: 0
                })
            },
            "list" => {

                {
                    let borrowed_schema = json_schema;
                    if borrowed_schema["of"].is_null() || borrowed_schema["of"].is_object() == false {
                        return Err(NoProtoError::new("List kind requires 'of' property as schema object!"));
                    }
                }

                Ok(NoProtoSchema {
                    kind: Box::new(NoProtoSchemaKinds::List { 
                        of: NoProtoSchema::validate_model(&json_schema["of"])? 
                    }),
                    type_data: NoProtoList::type_idx(),
                    type_state: 0
                })
            },
            "map" => {

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["value"].is_null() || borrowed_schema["value"].is_object() == false {
                        return Err(NoProtoError::new("Map kind requires 'value' property as schema object!"));
                    }
                }
                Ok(NoProtoSchema {
                    kind: Box::new(NoProtoSchemaKinds::Map { 
                        value: NoProtoSchema::validate_model(&json_schema["value"])?
                    }),
                    type_data: NoProtoMap::type_idx(),
                    type_state: 0
                })
            },
            "tuple" => {

                let mut schemas: Vec<NoProtoSchema> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["values"].is_null() || borrowed_schema["values"].is_array() == false  {
                        return Err(NoProtoError::new("Tuple type requires 'values' property as array of schema objects!"));
                    }

                    for schema in borrowed_schema["values"].members() {
                        let good_schema = NoProtoSchema::validate_model(schema)?;
                        schemas.push(good_schema);
                    }
                }
            
                Ok(NoProtoSchema {
                    kind: Box::new(NoProtoSchemaKinds::Tuple { 
                        values: schemas
                    }),
                    type_data: NoProtoTuple::type_idx(),
                    type_state: 0
                })
            },
            "option" => { 

                let mut options: Vec<String> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["options"].is_null() || borrowed_schema["options"].is_array() == false  {
                        return Err(NoProtoError::new("Option kind requires 'options' property as array of choices!"));
                    }

                    for option in borrowed_schema["options"].members() {
                        options.push(option.to_string());
                    }
                }

                if options.len() > 255 {
                    return Err(NoProtoError::new("Cannot have more than 255 choices for option type!"));
                }
                Ok(NoProtoSchema {
                    kind: Box::new(NoProtoSchemaKinds::Enum { 
                        choices: options
                    }),
                    type_data: NoProtoOption::type_idx(),
                    type_state: 0
                })
            },
            _ => {
                NoProtoTypes::get_type(type_string, json_schema)
            }
        }
    }
}