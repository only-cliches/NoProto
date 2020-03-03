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

pub struct NoProtoSchema {
    kind: String,

    // only used by table kind
    columns: Option<Vec<(String, NoProtoSchema)>>,

    // only used by table columns
    i: Option<u8>,

    // only used by list kind
    of: Option<Box<NoProtoSchema>>,

    // only used by map kind
    key: Option<Box<NoProtoSchema>>,
    value: Option<Box<NoProtoSchema>>,

    // only used for enum / option kinds
    options: Option<Vec<String>>,

    // only use for tuple kinds
    size: Option<u8>,
    values: Option<Vec<NoProtoSchema>>,
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

impl NoProtoSchema {
    
    pub fn from_json(json: JsonValue) -> std::result::Result<Rc<RefCell<NoProtoSchema>>, &'static str> {
        Ok(Rc::new(RefCell::new(NoProtoSchema { 
            kind: "".to_owned(), 
            columns: None, 
            i: None,
            of: None,
            key: None,
            value: None,
            options: None,
            size: None,
            values: None 
        })))
    }
/*
    pub fn from_struct(schema: NoProtoSchema) -> Rc<RefCell<NoProtoSchema>> {
        Rc::new(RefCell::new(NoProtoSchema { 
            kind: "".to_owned(), 
            columns: None, 
            i: None,
            of: None,
            key: None,
            value: None,
            options: None,
            size: None,
            values: None 
        }))
    }
*/
    pub fn validate_model(&self, jsonSchema: JsonValue) -> std::result::Result<NoProtoSchema, &'static str> {
        let kind_string = jsonSchema["type"].as_str().unwrap_or("");

        let mut matched_kind = false;
        for i in 0..VALID_KINDS_COLLECTIONS.len() {
            let kind = VALID_KINDS_COLLECTIONS[i];
            if kind == kind_string {
                matched_kind = true;
            }
        };

        for i in 0..VALID_KINDS_SCALAR.len() {
            let kind = VALID_KINDS_SCALAR[i];
            if kind == kind_string {
                matched_kind = true;
            }
        };

        // validate that we have a valid kind
        if matched_kind == false {
            return Err("Not a valid kind!");
        }

        // validate required properties are in place for each kind
        match kind_string {
            "table" => {
                if jsonSchema["columns"].is_null() || jsonSchema["columns"].is_array() == false {
                    return Err("Table kind requires 'columns' property as array!");
                }
                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: None,
                    size: None,
                    values: None 
                })
            },
            "list" => {
                if jsonSchema["of"].is_null() || jsonSchema["of"].is_object() == false {
                    return Err("List kind requires 'of' property as schema object!");
                }

                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: None,
                    size: None,
                    values: None 
                })
            },
            "map" => {
                if jsonSchema["value"].is_null() || jsonSchema["value"].is_object() == false {
                    return Err("Map kind requires 'value' property as schema object!");
                }
                if jsonSchema["key"].is_null() || jsonSchema["key"].is_object() == false {
                    return Err("Map kind requires 'key' property as schema object!");
                }

                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: None,
                    size: None,
                    values: None 
                })
            },
            "tuple" => {
                if jsonSchema["size"].is_number() == false  {
                    return Err("Tuple kind requires 'size' property as number!");
                }
                if jsonSchema["values"].is_null() || jsonSchema["values"].is_array() == false  {
                    return Err("Tuple kind requires 'values' property as array of schema objects!");
                }

                let schemas: Vec<NoProtoSchema> = vec![];
/*
                for schema in jsonSchema["values"].members() {
                    let goodSchema = self.validate_model(schema)?;
                    schemas.push();
                }
*/

                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: None,
                    size: None,
                    values: None 
                })
            },
            "option" => { 
                if jsonSchema["options"].is_null() || jsonSchema["options"].is_array() == false  {
                    return Err("Option kind requires 'options' property as array of choices!");
                }

                let mut options: Vec<String> = vec![];

                for option in jsonSchema["options"].members() {
                    options.push(option.to_string());
                }

                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: Some(options),
                    size: None,
                    values: None 
                })
            },
            _ => { // scalar type, nothing special to do
                Ok(NoProtoSchema { 
                    kind: kind_string.to_owned(), 
                    columns: None, 
                    i: None,
                    of: None,
                    key: None,
                    value: None,
                    options: None,
                    size: None,
                    values: None 
                })
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
        if (location + bytes.len() as u32) as u64 > 4294967296u64 {
            return None;
        }

        &self.bytes.extend(bytes);
        Some(location)
    }
}

pub struct NoProtoBuffer {
    pub memory: Rc<RefCell<NoProtoMemory>>,
    rootModel: Rc<RefCell<JsonValue>>
}

impl NoProtoBuffer {

    pub fn new(model: Rc<RefCell<JsonValue>>, capcity: Option<usize>) -> Self { // make new buffer

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

    pub fn load(model: Rc<RefCell<JsonValue>>, bytes: Vec<u8>) -> Self { // load existing buffer
        NoProtoBuffer {
            memory: Rc::new(RefCell::new(NoProtoMemory { bytes: bytes})),
            rootModel: model
        }
    }

    pub fn get_root(&self) -> NoProtoPointer {        
        NoProtoPointer::new_standard(1, Rc::clone(&self.rootModel), Rc::clone(&self.memory))
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
        let buffer = NoProtoBuffer::new(Rc::new(RefCell::new(json::parse(r#"
            {
                "type": "uuid"
            }
        "#).unwrap())), None);

        let mut root = buffer.get_root();

     
        root.set_uuid(NoProtoUUID::generate());

        println!("VALUE: {:?}", root.to_uuid().unwrap());
        println!("BYTES: {:?}", buffer.memory.borrow().bytes);

        assert_eq!(2 + 2, 4);
    }
}
