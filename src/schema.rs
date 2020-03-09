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
    Map { key: NoProtoSchema, value: NoProtoSchema },
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
*/

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
            NoProtoSchemaKinds::Map { key: _, value: _ } => "map",
            NoProtoSchemaKinds::Enum { choices: _ } => "option",
            NoProtoSchemaKinds::Tuple { values: _ } => "tuple",
            _ => "Uknonw"
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
                    if borrowed_schema["key"].is_null() || borrowed_schema["key"].is_object() == false {
                        return Err(NoProtoError::new("Map kind requires 'key' property as schema object!"));
                    }
    
                    let key_kind = borrowed_schema["key"]["kind"].to_string();
                    let mut key_kind_is_scalar = false;
    
                    for i in 0..VALID_KINDS_SCALAR.len() {
                        let kind = VALID_KINDS_SCALAR[i];
                        if kind == key_kind {
                            key_kind_is_scalar = true;
                        }
                    };
    
                    if key_kind_is_scalar == false {
                        return Err(NoProtoError::new("Map 'key' property must be a scalar type, not a collection type!"));
                    }
                }


                Ok(NoProtoSchema { 
                    kind: Box::new(NoProtoSchemaKinds::Map { 
                        key: self.validate_model(&json_schema["key"])?,
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