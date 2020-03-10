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
//! | [`list`](#list)     | [`NoProtoList`](../collection/list/index.html)                           | 4 bytes - ~4GB | Linked list with up to 65,535 items.                                     |
//! | [`map`](#map)       | [`NoProtoMap`](../collection/map/index.html)                             | 4 bytes - ~4GB | Linked list with Vec<u8> keys.                                           |
//! | [`tuple`](#tuple)   | [`NoProtoTuple`](../collection/tuple/index.html)                         | 4 bytes - ~4GB | Static sized collection of values.                                       |
//! | [`string`](#string) | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      | 4 bytes - ~4GB | Utf-8 formatted string.                                                  |
//! | [`bytes`](#bytes)   | [`Vec<u8>`](https://doc.rust-lang.org/std/vec/struct.Vec.html)           | 4 bytes - ~4GB | Arbitrary bytes.                                                         |
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
//! | [`option`](#option) | [`String`](https://doc.rust-lang.org/std/string/struct.String.html)      | 1 byte         | Up to 255 strings in schema.                                             |
//! | [`bool`](#bool)     | [`bool`](https://doc.rust-lang.org/std/primitive.bool.html)              | 1 byte         |                                                                          |
//! | [`dec64`](#dec64)   | [`NoProtoDec`](..pointer/struct.NoProtoDec.html)                         | 9 bytes        | Big Integer Decimal format.                                              |
//! | [`geo4`](#geo4)     | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 4 bytes        | 1.5km resolution (city)                                                  |
//! | [`geo8`](#geo8)     | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 8 bytes        | 16mm resolution (marble)                                                 |
//! | [`geo16`](#geo16)   | [`NoProtoGeo`](../pointer/struct.NoProtoGeo.html)                        | 16 bytes       | 3.5nm resolution (flea)                                                  |
//! | [`tid`](#tid)       | [`NoProtoTimeID`](../pointer/struct.NoProtoTimeID.html)                  | 16 bytes       | 8 byte u64 for time with 8 bytes of random numbers.                      |
//! | [`uuid`](#uuid)     | [`NoProtoUUID`](../pointer/struct.NoProtoUUID.html)                      | 16 bytes       | v4 UUID, 2e37 possible UUID v4s                                          |
//! | [`date`](#date)     | [`u64`](https://doc.rust-lang.org/std/primitive.u64.html)                | 8 bytes        | Good to store unix epoch (in seconds) until the year 584,942,417,355     |
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
//! # int8
//! 
//! # int16
//! 
//! # int32
//! 
//! # int64
//! 
//! # uint8
//! 
//! # uint16
//! 
//! # uint32
//! 
//! # uint64
//! 
//! # float
//! 
//! # double
//! 
//! # option
//! 
//! # bool
//! 
//! # dec64
//! 
//! # geo4
//! 
//! # geo8
//! 
//! # geo16
//! 
//! # tid
//! 
//! # uuid
//! 
//! # date
//! 
//!  
use json::*;
use crate::error::NoProtoError;

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

#[derive(Debug)]
pub struct NoProtoSchema {
    pub kind: Box<NoProtoSchemaKinds>
}

impl NoProtoSchema {

    pub fn init() -> NoProtoSchema {
        NoProtoSchema {
            kind: Box::new(NoProtoSchemaKinds::None)
        }
    }

    pub fn get_type_str(&self) -> &str {
        match &*self.kind {
            NoProtoSchemaKinds::None => "None",
            NoProtoSchemaKinds::Utf8String => "string",
            NoProtoSchemaKinds::Bytes => "bytes",
            NoProtoSchemaKinds::Int8 => "int8",
            NoProtoSchemaKinds::Int16 => "int16",
            NoProtoSchemaKinds::Int32 => "int32",
            NoProtoSchemaKinds::Int64 => "int64",
            NoProtoSchemaKinds::Uint8 => "uint8",
            NoProtoSchemaKinds::Uint16 => "uint16",
            NoProtoSchemaKinds::Uint32 => "uint32",
            NoProtoSchemaKinds::Uint64 => "uint64",
            NoProtoSchemaKinds::Float => "float",
            NoProtoSchemaKinds::Double => "double",
            NoProtoSchemaKinds::Dec64 => "dec64",
            NoProtoSchemaKinds::Boolean => "bool",
            NoProtoSchemaKinds::Geo4 => "geo4",
            NoProtoSchemaKinds::Geo8 => "geo8",
            NoProtoSchemaKinds::Geo16 => "geo16",
            NoProtoSchemaKinds::Uuid => "uuid",
            NoProtoSchemaKinds::Tid => "tid",
            NoProtoSchemaKinds::Date => "date",
            NoProtoSchemaKinds::Table { columns: _ } => "table",
            NoProtoSchemaKinds::List { of: _ } => "list",
            NoProtoSchemaKinds::Map { value: _ } => "map",
            NoProtoSchemaKinds::Enum { choices: _ } => "option",
            NoProtoSchemaKinds::Tuple { values: _ } => "tuple",
            // _ => "Uknonw"
        }
    }

    pub fn from_json(&mut self, json: JsonValue) -> std::result::Result<NoProtoSchema, NoProtoError> {
        self.validate_model(&json)
    }

    pub fn validate_model(&self, json_schema: &JsonValue) -> std::result::Result<NoProtoSchema, NoProtoError> {

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

                        let good_schema = self.validate_model(&column[1])?;

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
                    })
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
                        of: self.validate_model(&json_schema["of"])? 
                    })
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
                        value: self.validate_model(&json_schema["value"])?
                    })
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
                        let good_schema = self.validate_model(schema)?;
                        schemas.push(good_schema);
                    }
                }
            
                Ok(NoProtoSchema { 
                    kind: Box::new(NoProtoSchemaKinds::Tuple { 
                        values: schemas
                    })
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
                    })
                })
            },
            "string" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Utf8String) })
            },
            "bytes" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Bytes) })
            },
            "int8" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Int8) })
            },
            "int16" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Int16) })
            },
            "int32" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Int32) })
            },
            "int64" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Int64) })
            },
            "uint8" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Uint8) })
            },
            "uint16" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Uint16) })
            },
            "uint32" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Uint32) })
            },
            "uint64" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Uint64) })
            },
            "float" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Float) })
            },
            "f32" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Float) })
            },
            "double" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Double) })
            },
            "f64" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Double) })
            },
            "dec64" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Dec64) })
            },
            "boolean" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Boolean) })
            },
            "bool" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Boolean) })
            },
            "geo4" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Geo4) })
            },
            "geo8" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Geo8) })
            },
            "geo16" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Geo16) })
            },
            "uuid" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Uuid) })
            },
            "tid" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Tid) })
            },
            "date" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Date) })
            },
            _ => {
                Err(NoProtoError::new("Not a valid type!"))
            }
        }
    }
}