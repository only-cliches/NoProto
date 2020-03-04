use crate::pointer::NoProtoPointer;
use std::cell::BorrowMutError;
use std::cell::Cell;
use std::cell::RefMut;
use std::cell::Ref;
use json::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::result;

const PROTOCOL_VERSION: u8 = 0;

mod pointer;
mod collection;

pub struct NoProtoFactory {
    schema: NoProtoSchema
}

impl NoProtoFactory {
    pub fn new(json_schema: JsonValue) -> std::result::Result<NoProtoFactory, &'static str> {
        let mut new_schema = NoProtoSchema::init();

        let valid_schema = new_schema.from_json(json_schema)?;

        Ok(NoProtoFactory {
            schema: valid_schema
        })
    }
    /*
    pub fn creat_buffer() -> NoProtoBuffer {

    }

    pub fn parse_buffer() -> NoProtoBuffer {

    }
    */
}

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
    Dec32,
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
    Tuple { size: u8, values: Vec<NoProtoSchema>}
}

const VALID_KINDS_COLLECTIONS: [&str; 4] = [
    "table",
    "map",
    "list",
    "tuple",
];

const VALID_KINDS_SCALAR: [&str; 22] = [
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
    "dec32",
    "dec64",
    "boolean",
    "geo4",
    "geo8",
    "geo16",
    "uuid",
    "tid",
    "date"
];

pub struct NoProtoSchema {
    kind: Box<NoProtoSchemaKinds>
}

impl NoProtoSchema {

    pub fn init() -> NoProtoSchema {
        NoProtoSchema {
            kind: Box::new(NoProtoSchemaKinds::None)
        }
    }

    pub fn from_json(&mut self, json: JsonValue) -> std::result::Result<NoProtoSchema, &'static str> {
        self.validate_model(&json)
    }

    pub fn validate_model(&self, json_schema: &JsonValue) -> std::result::Result<NoProtoSchema, &'static str> {

        let kind_string = json_schema["type"].as_str().unwrap_or("");


        // validate required properties are in place for each kind
        match kind_string {
            "table" => {
                
                let mut columns: Vec<Option<(u8, String, NoProtoSchema)>> = vec![];

                for _x in 0..255 {
                    columns.push(None);
                }

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["columns"].is_null() || borrowed_schema["columns"].is_array() == false {
                        return Err("Table kind requires 'columns' property as array!");
                    }

                    let mut index = 0;
                    for column in borrowed_schema["columns"].members() {

                        let column_name = &column[0].to_string();

                        if column_name.len() == 0 {
                            return Err("Table kind requires all columns have a name!");
                        }

                        let good_schema = self.validate_model(&column[1])?;

                        let this_index = &column[1]["i"];

                        let use_index = if this_index.is_null() || this_index.is_number() == false {
                            index
                        } else {
                            this_index.as_usize().unwrap_or(index)
                        };

                        if (use_index > 255) {
                            return Err("Table cannot have column index above 255!");
                        }

                        match &columns[use_index] {
                            Some(x) => {
                                return Err("Table column index numbering conflict!");
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
                        return Err("List kind requires 'of' property as schema object!");
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
                        return Err("Map kind requires 'value' property as schema object!");
                    }
                    if borrowed_schema["key"].is_null() || borrowed_schema["key"].is_object() == false {
                        return Err("Map kind requires 'key' property as schema object!");
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
                        return Err("Map 'key' property must be a scalar type, not a collection type!");
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
                let mut size = 0;

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["size"].is_number() == false  {
                        return Err("Tuple kind requires 'size' property as number!");
                    }

                    size = borrowed_schema["size"].as_u8().unwrap_or(0);

                    if borrowed_schema["values"].is_null() || borrowed_schema["values"].is_array() == false  {
                        return Err("Tuple kind requires 'values' property as array of schema objects!");
                    }

                    for schema in borrowed_schema["values"].members() {
                        let good_schema = self.validate_model(schema)?;
                        schemas.push(good_schema);
                    }
                }
            
                Ok(NoProtoSchema { 
                    kind: Box::new(NoProtoSchemaKinds::Tuple { 
                        size: size,
                        values: schemas
                    })
                })
            },
            "option" => { 

                let mut options: Vec<String> = vec![];

                {
                    let borrowed_schema = json_schema;

                    if borrowed_schema["options"].is_null() || borrowed_schema["options"].is_array() == false  {
                        return Err("Option kind requires 'options' property as array of choices!");
                    }

                    for option in borrowed_schema["options"].members() {
                        options.push(option.to_string());
                    }
                }

                if options.len() > 255 {
                    return Err("Cannot have more than 255 choices for option type!");
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
            "dec32" => {
                Ok(NoProtoSchema { kind: Box::new(NoProtoSchemaKinds::Dec32) })
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
                Err("Not a valid kind!")
            }
        }
    }
}

pub struct NoProtoMemory {
    pub bytes: Vec<u8>
}

impl NoProtoMemory {
    pub fn malloc(&mut self, bytes: Vec<u8>) -> Option<u32> {
        let location: u32 = self.bytes.len() as u32;

        // not enough space left?
        if (location + bytes.len() as u32) as u64 >= std::u32::MAX as u64 {
            return None;
        }

        &self.bytes.extend(bytes);
        Some(location)
    }
}

struct NoProtoBuffer<'a> {
    pub memory: Rc<RefCell<NoProtoMemory>>,
    rootModel: &'a NoProtoSchema
}

impl<'a> NoProtoBuffer<'a> {

    pub fn new(model: &'a NoProtoSchema, capcity: Option<usize>) -> Self { // make new buffer

        let capacity = match capcity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes: Vec<u8> = Vec::with_capacity(capacity);

        new_bytes.extend(vec![
            PROTOCOL_VERSION, // Protocol version (for breaking changes if needed later)
            0, 0, 0, 0        // u32 HEAD for root value (starts at zero)
        ]); 

        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: new_bytes })),
            rootModel: model
        }
    }

    pub fn load(model: &'a NoProtoSchema, bytes: Vec<u8>) -> Self { // load existing buffer
        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: bytes})),
            rootModel: model
        }
    }

    pub fn get_root(&self) -> NoProtoPointer {        
        NoProtoPointer::new_standard(1, self.rootModel, Rc::clone(&self.memory))
    }

    pub fn compact(&self)  {
        
    }

    pub fn calc_wasted_bytes(&self) -> u32 {

        let total_bytes = self.memory.borrow().bytes.len() as u32;

        return 0;
    }

    pub fn maybe_compact<F>(&self, mut callback: F) -> bool 
        where F: FnMut(f32, f32) -> bool // wasted bytes, percent of waste
    {
        let wasted_bytes = self.calc_wasted_bytes() as f32;

        let total_bytes = self.memory.borrow().bytes.len() as f32;

        let size_without_waste = total_bytes - wasted_bytes;

        if callback(wasted_bytes, (total_bytes / size_without_waste) as f32) {
            self.compact();
            true
        } else {
            false
        }
    }

}

#[cfg(test)]
mod tests {
    use crate::{pointer::NoProtoGeo, NoProtoBuffer, pointer::NoProtoUUID};
    use json::*;
    use std::{rc::Rc, cell::RefCell};

    #[test]
    fn it_works() {
        /*let buffer = NoProtoBuffer::new(Rc::new(RefCell::new(json::parse(r#"
            {
                "type": "uuid"
            }
        "#).unwrap())), None);

        let mut root = buffer.get_root();

     
        root.set_uuid(NoProtoUUID::generate());

        println!("VALUE: {:?}", root.to_uuid().unwrap());
        println!("BYTES: {:?}", buffer.memory.borrow().bytes);

        assert_eq!(2 + 2, 4);*/
    }
}
