//! Schemas are JSON used to declare & store the shape of buffer objects.
//! 
//! No Proto Schemas are JSON objects that describe how the data in a NP_ buffer is stored.
//! 
//! Every schema object has at least a "type" key that provides the kind of value stored at that part of the schema.  Additional keys are dependent on the type of schema.
//! 
//! Schemas are validated and sanity checked by the [NP_Factory](../struct.NP_Factory.html) struct upon creation.  You cannot pass an invalid schema into a factory constructor and build/parse buffers with it.
//! 
//! If you're familiar with typescript, schemas can be described by this recursive interface:
//! ```text
//! interface NP_Schema {
//!     type: string;
//!     
//!     // used by list types
//!     of?: NP_Schema
//!     
//!     // used by map types
//!     value?: NP_Schema
//! 
//!     // used by tuple types
//!     values?: NP_Schema[]
//! 
//!     // used by table types
//!     columns?: [string, NP_Schema][]
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
//! Nesting is easy to perform.  For example, this  is a list of tables.  Each table has two columns: id and title.  Both columns are a string type.
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
//! | Type                                   | Rust Type / Struct                                                       | Bytes (Size)   | Limits / Notes                                                           |
//! |----------------------------------------|--------------------------------------------------------------------------|----------------|--------------------------------------------------------------------------|
//! | [`table`](#table)                      | [`NP_Table`](../collection/table/index.html)                             | 4 bytes - ~4GB | Linked list with indexed keys that map against up to 255 named columns.  |
//! | [`list`](#list)                        | [`NP_List`](../collection/list/index.html)                               | 8 bytes - ~4GB | Linked list with up to 65,535 items.                                     |
//! | [`map`](#map)                          | [`NP_Map`](../collection/map/index.html)                                 | 4 bytes - ~4GB | Linked list with `Vec<u8>` keys.                                         |
//! | [`tuple`](#tuple)                      | [`NP_Tuple`](../collection/tuple/index.html)                             | 4 bytes - ~4GB | Static sized collection of specific values.                              |
//! | [`any`](#any)                          | [`NP_Any`](https://doc.rust-lang.org/std/string/struct.String.html)      | 4 bytes - ~4GB | Generic type.                                                            |
//! | [`string`](#string)                    | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      | 4 bytes - ~4GB | Utf-8 formatted string.                                                  |
//! | [`bytes`](#bytes)                      | [`NP_Bytes`](https://doc.rust-lang.org/std/vec/struct.Vec.html)          | 4 bytes - ~4GB | Arbitrary bytes.                                                         |
//! | [`int8`](#int8-int16-int32-int64)      | [`i8`](https://doc.rust-lang.org/std/primitive.i8.html)                  | 1 byte         | -127 to 127                                                              |
//! | [`int16`](#int8-int16-int32-int64)     | [`i16`](https://doc.rust-lang.org/std/primitive.i16.html)                | 2 bytes        | -32,768 to 32,768                                                        |
//! | [`int32`](#int8-int16-int32-int64)     | [`i32`](https://doc.rust-lang.org/std/primitive.i32.html)                | 4 bytes        | -2,147,483,648 to 2,147,483,648                                          |
//! | [`int64`](#int8-int16-int32-int64)     | [`i64`](https://doc.rust-lang.org/std/primitive.i64.html)                | 8 bytes        | -9.22e18 to 9.22e18                                                      |
//! | [`uint8`](#uint8-uint16-uint32-uint64) | [`u8`](https://doc.rust-lang.org/std/primitive.u8.html)                  | 1 byte         | 0 - 255                                                                  |
//! | [`uint16`](#uint8-uint16-uint32-uint64)| [`u16`](https://doc.rust-lang.org/std/primitive.u16.html)                | 2 bytes        | 0 - 65,535                                                               |
//! | [`uint32`](#uint8-uint16-uint32-uint64)| [`u32`](https://doc.rust-lang.org/std/primitive.u32.html)                | 4 bytes        | 0 - 4,294,967,295                                                        |
//! | [`uint64`](#uint8-uint16-uint32-uint64)| [`u64`](https://doc.rust-lang.org/std/primitive.u64.html)                | 8 bytes        | 0 - 1.84e19                                                              |
//! | [`float`](#float-double)               | [`f32`](https://doc.rust-lang.org/std/primitive.f32.html)                | 4 bytes        | -3.4e38 to 3.4e38                                                        |
//! | [`double`](#float-double)              | [`f64`](https://doc.rust-lang.org/std/primitive.f64.html)                | 8 bytes        | -1.7e308 to 1.7e308                                                      |
//! | [`option`](#option)                    | [`NP_Option`](https://doc.rust-lang.org/std/string/.html)                | 1 byte         | Up to 255 strings in schema.                                             |
//! | [`bool`](#bool)                        | [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)              | 1 byte         |                                                                          |
//! | [`dec64`](#dec64)                      | [`NP_Dec`](..pointer/struct.NP_Dec.html)                                 | 9 bytes        | Big Integer Decimal format.                                              |
//! | [`geo4`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/struct.NP_Geo.html)                                | 4 bytes        | 1.5km resolution (city) geographic coordinate                            |
//! | [`geo8`](#geo4-geo8-geo16)             | [`NP_Geo`](../pointer/struct.NP_Geo.html)                                | 8 bytes        | 16mm resolution (marble) geographic coordinate                           |
//! | [`geo16`](#geo4-geo8-geo16)            | [`NP_Geo`](../pointer/struct.NP_Geo.html)                                | 16 bytes       | 3.5nm resolution (flea) geographic coordinate                            |
//! | [`tid`](#tid)                          | [`NP_TimeID`](../pointer/struct.NP_TimeID.html)                          | 16 bytes       | 8 byte u64 for time with 8 bytes of random numbers.                      |
//! | [`uuid`](#uuid)                        | [`NP_UUID`](../pointer/struct.NP_UUID.html)                              | 16 bytes       | v4 UUID, 2e37 possible UUID v4s                                          |
//! | [`date`](#date)                        | [`NP_Date`](https://doc.rust-lang.org/std/primitive.u64.html)            | 8 bytes        | Good to store unix epoch (in seconds) until the year 584,942,417,355     |
//!  
//! # table
//! 
//! # list
//! 
//! # map
//! 
//! # tuple
//! 
//! # any
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
use crate::pointer::NP_ValueInto;
use crate::pointer::any::NP_Any;
use crate::pointer::misc::NP_Date;
use crate::pointer::misc::NP_UUID;
use crate::pointer::misc::NP_TimeID;
use crate::pointer::misc::NP_Geo;
use crate::pointer::misc::NP_Dec;
use crate::collection::tuple::NP_Tuple;
use crate::pointer::bytes::NP_Bytes;
use crate::collection::{list::NP_List, table::NP_Table, map::NP_Map};
use crate::pointer::{misc::NP_Option, NP_Value};
use json::*;
use crate::error::NP_Error;


#[doc(hidden)]
pub enum NP_SchemaKinds {
    None,
    Scalar,
    Table { columns: Vec<Option<(u8, String, NP_Schema)>> },
    List { of: NP_Schema },
    Map { value: NP_Schema },
    Enum { choices: Vec<String> },
    Tuple { values: Vec<NP_Schema>}
}

/*
#[derive(Debug)]
pub enum NP_SchemaKinds {
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
    Table { columns: Vec<Option<(u8, String, NP_Schema)>> },
    List { of: NP_Schema },
    Map { value: NP_Schema },
    Enum { choices: Vec<String> },
    Tuple { values: Vec<NP_Schema>}
}
*/

#[doc(hidden)]
// These are just used for runtime type comparison, the type information is never stored in the buffer.
// When you cast a pointer to some type, this enum is used as comparing numbers is very efficient.
pub enum NP_TypeKeys {
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

#[doc(hidden)]
pub struct NP_Schema {
    pub kind: Box<NP_SchemaKinds>,
    pub type_data: (i64, String),
    pub type_state: i64
}

#[doc(hidden)]
pub struct NP_Types { }

impl<'a> NP_Types {
    pub fn do_check<T: NP_Value + Default + NP_ValueInto<'a>>(type_string: &str, json_schema: &JsonValue)-> std::result::Result<Option<NP_Schema>, NP_Error>{
        if T::is_type(type_string) {
            Ok(Some(NP_Schema { 
                kind: Box::new(NP_SchemaKinds::Scalar),
                type_data: T::type_idx(),
                type_state: T::schema_state(type_string, json_schema)?
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_type(type_string: &str, json_schema: &JsonValue)-> std::result::Result<NP_Schema, NP_Error> {

        let check = NP_Types::do_check::<NP_Any>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NP_Types::do_check::<String>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Bytes>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<i64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u8>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u16>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<u64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<f32>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<f64>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<bool>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Dec>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Geo>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_TimeID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };
    
        let check = NP_Types::do_check::<NP_UUID>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        let check = NP_Types::do_check::<NP_Date>(type_string, json_schema)?;
        match check { Some(x) => return Ok(x), None => {} };

        Err(NP_Error::new(format!("{} is not a valid type!", type_string).as_str()))
    }
}

#[doc(hidden)]
impl NP_Schema {

    pub fn blank() -> NP_Schema {

        NP_Schema {
            kind: Box::new(NP_SchemaKinds::None),
            type_data: (-1, "".to_owned()),
            type_state: 0
        }
    }

    pub fn from_json(json: JsonValue) -> std::result::Result<Self, NP_Error> {
        NP_Schema::validate_model(&json)
    }

    pub fn validate_model(json_schema: &JsonValue) -> std::result::Result<Self, NP_Error> {

        let type_string = json_schema["type"].as_str().unwrap_or("");

        if type_string.len() == 0 {
            return Err(NP_Error::new("Must declare a type for every schema!"));
        }


        // validate required properties are in place for each kind
        match type_string {
            "table" => {
                
                let mut columns: Vec<Option<(u8, String, NP_Schema)>> = vec![];

                for _x in 0..255 {
                    columns.push(None);
                }

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["columns"].is_null() || borrowed_schema["columns"].is_array() == false {
                        return Err(NP_Error::new("Table kind requires 'columns' property as array!"));
                    }

                    let mut index = 0;
                    for column in borrowed_schema["columns"].members() {

                        let column_name = &column[0].to_string();

                        if column_name.len() == 0 {
                            return Err(NP_Error::new("Table kind requires all columns have a name!"));
                        }

                        let good_schema = NP_Schema::validate_model(&column[1])?;

                        let this_index = &column[1]["i"];

                        let use_index = if this_index.is_null() || this_index.is_number() == false {
                            index
                        } else {
                            this_index.as_usize().unwrap_or(index)
                        };

                        if use_index > 255 {
                            return Err(NP_Error::new("Table cannot have column index above 255!"));
                        }

                        match &columns[use_index] {
                            Some(_x) => {
                                return Err(NP_Error::new("Table column index numbering conflict!"));
                            },
                            None => {
                                columns[use_index] = Some((use_index as u8, column_name.to_string(), good_schema));
                            }
                        };

                        index += 1;
                    }
                }

                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Table { 
                        columns: columns 
                    }),
                    type_data: NP_Table::type_idx(),
                    type_state: 0
                })
            },
            "list" => {

                {
                    let borrowed_schema = json_schema;
                    if borrowed_schema["of"].is_null() || borrowed_schema["of"].is_object() == false {
                        return Err(NP_Error::new("List kind requires 'of' property as schema object!"));
                    }
                }

                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::List { 
                        of: NP_Schema::validate_model(&json_schema["of"])? 
                    }),
                    type_data: NP_List::type_idx(),
                    type_state: 0
                })
            },
            "map" => {

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["value"].is_null() || borrowed_schema["value"].is_object() == false {
                        return Err(NP_Error::new("Map kind requires 'value' property as schema object!"));
                    }
                }
                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Map { 
                        value: NP_Schema::validate_model(&json_schema["value"])?
                    }),
                    type_data: NP_Map::type_idx(),
                    type_state: 0
                })
            },
            "tuple" => {

                let mut schemas: Vec<NP_Schema> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["values"].is_null() || borrowed_schema["values"].is_array() == false  {
                        return Err(NP_Error::new("Tuple type requires 'values' property as array of schema objects!"));
                    }

                    for schema in borrowed_schema["values"].members() {
                        let good_schema = NP_Schema::validate_model(schema)?;
                        schemas.push(good_schema);
                    }
                }
            
                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Tuple { 
                        values: schemas
                    }),
                    type_data: NP_Tuple::type_idx(),
                    type_state: 0
                })
            },
            "option" => { 

                let mut options: Vec<String> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["options"].is_null() || borrowed_schema["options"].is_array() == false  {
                        return Err(NP_Error::new("Option kind requires 'options' property as array of choices!"));
                    }

                    for option in borrowed_schema["options"].members() {
                        options.push(option.to_string());
                    }
                }

                if options.len() > 255 {
                    return Err(NP_Error::new("Cannot have more than 255 choices for option type!"));
                }
                Ok(NP_Schema {
                    kind: Box::new(NP_SchemaKinds::Enum { 
                        choices: options
                    }),
                    type_data: NP_Option::type_idx(),
                    type_state: 0
                })
            },
            _ => {
                NP_Types::get_type(type_string, json_schema)
            }
        }
    }
}