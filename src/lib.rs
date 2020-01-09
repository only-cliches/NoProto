use json::*;
use fnv::FnvHasher;
use std::hash::Hasher;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


pub struct NoProtoMapModel {
    keyType: String,
    model: Box<NoProtoDataModel>
}

pub struct NoProtoDataModel {
    colKey: String,
    colType: String,
    options: JsonValue,
    table: Option<Box<Vec<NoProtoDataModel>>>, // nested type (table)
    list: Option<Box<NoProtoDataModel>>, // nested type (list)
    map: Option<Box<NoProtoMapModel>> // nested map type
}

pub struct NoProtoFactory {
    loaded: bool,
    dataModel: Box<NoProtoDataModel>
}

impl NoProtoFactory {

    pub fn new() -> Self {
        NoProtoFactory {
            loaded: false,
            dataModel: Box::new(NoProtoDataModel {
                colKey: "".to_string(),
                colType: "".to_string(),
                options: object!{},
                table: None,
                list: None,
                map: None
            })
        }
    }

    pub fn from_object(&self, model: JsonValue) -> Result<Self> {
        Ok(NoProtoFactory {
            loaded: true,
            dataModel: Box::new(NoProtoDataModel {
                colKey: "root".to_string(),
                colType: "root".to_string(),
                options: object!{},
                table: self.load_model_table(model),
                list: None,
                map: None
            })
        })
    }

    pub fn from_string(&self, model: &str) -> Result<Self> {
        match json::parse(model) {
            Ok(x) => self.from_object(x),
            Err(e) => Err(e) 
        }
    }

    fn load_model_table(&self, model: JsonValue) -> Option<Box<Vec<NoProtoDataModel>>> {

        let mut i: usize = 0;

        let length = model.len();

        let mut columns = vec![];

        loop {
            if i < length {

                let model_row: &JsonValue = &model[i];

                let row_key = &model_row[0].as_str().unwrap_or("").to_owned();
                let row_type = &model_row[1].as_str().unwrap_or("").to_owned();
                let row_options = if model_row[2].is_null() { object!{} } else { model_row[2].clone() };

                columns.push(self.load_model_single(row_key.clone(), row_type.clone(), row_options));

                i += 1;
            } else {
                break;
            }
        }

        return Some(Box::new(columns));
    }

    fn load_model_single(&self, row_key: String, row_type: String, options: JsonValue) -> NoProtoDataModel {

        let isList = row_type.rfind("[]");
        let isTable = row_type.eq("table");
        let isMap = row_type.find("map<");

        if isList.is_some() { // list type
            let listType = &row_type[0..isList.unwrap_or(0)];

            NoProtoDataModel {
                colKey: row_key,
                colType: "list".to_owned(),
                options: options.clone(),
                table: None,
                list: Some(Box::new(self.load_model_single("*".to_string(), listType.to_owned() , options.clone()))),
                map: None
            }

        } else if isTable == true { // table type

            NoProtoDataModel {
                colKey: row_key,
                colType: row_type,
                options: options.clone(),
                table: if options["model"].is_null() { None } else { self.load_model_table(options["model"].clone()) },
                list: None,
                map: None
            }

        } else if isMap.is_some() { // map type

            // find comma between vales, may be nested maps....
            let mut comma = 0;
            let mut level = 0;
            for (i, chr) in row_type.chars().enumerate() {
                let c_str = chr.to_string();

                if c_str == "<" {
                    level += 1;
                } else if c_str == ">" {
                    level -= 1;
                } else if c_str == "," {
                    if level == 1 {
                        comma = i;
                    }
                };
            }

            let keyType = &row_type[4..comma];
            let valueType = &row_type[(comma + 1)..(row_type.len() - 1)];

            NoProtoDataModel {
                colKey: row_key,
                colType: "map".to_owned(),
                options: options.clone(),
                table: None,
                list: None,
                map: Some(Box::new(NoProtoMapModel {
                    keyType: keyType.to_owned(),
                    model: Box::new(self.load_model_single("*".to_owned(), valueType.to_owned(), options.clone()))
                }))
            }

        } else {

            NoProtoDataModel { // scalar type
                colKey: row_key,
                colType: row_type,
                options: options.clone(),
                table: None,
                list: None,
                map: None
            }

        }
    }

    pub fn new_buffer(&self, length: Option<usize>) -> Option<NoProtoBuffer> {

        if self.loaded == false {
            None
        } else {
            Some(NoProtoBuffer::new(self.dataModel.as_ref(), length, None))
        }
    }

    pub fn parse_buffer(&self, in_buffer: Vec<u8>) -> Option<NoProtoBuffer> {
 
        if self.loaded == false {
            None
        } else {
            Some(NoProtoBuffer::new(self.dataModel.as_ref(), None, Some(in_buffer)))
        }
    }
}

// signed int: -2^(n - 1) to 2^(n - 1)
// unsigned int: (2^n) - 1
enum NoProtoBufferScalar {
    table {
        headPtr: u32 // points to first tableItem
    }, 
    tableItem {
        index: u8, // which index this value is at in the table
        valuePtr: u32, // points to value of this item
        nextPtr: u32 // points to next tableItem
    }, 
    list {
        headPtr: u32,  // first listItem on list (0 if empty)
        tailPtr: u32, // last listItem on list (0 if empty)
        length: u16 // number of elements in list
    },
    listItem {
        valuePtr: u32, // points to value of this item
        prevPtr: u32, // previouse listItem (0 if beginning)
        nextPtr: u32 // next listItem (0 if end)
    },
    map {
        headPtr: u32 // points to first map item
    },
    mapItem {
        key: u64, // hash
        valuePtr: u32, // pointer to value
        nextPtr: u32 // pointer to next mapItem
    },
    utf8_string {
        length: u32, // length of string
        value: Vec<u8> // sting encoded into bytes
    },
    bytes { 
        length: u32, // length of byte array
        value: Vec<u8> // bytes
    },
    int1 { value: i64 }, // int
    int8 { value: i8 },
    int16 { value: i16 },
    int32 { value: i32 },
    int64 { value: i64 },
    uint8 { value: u8 },
    uint16 { value: u16 },
    uint32 { value: u32 },
    uint64 { value: u64 },
    float { value: f32 },
    double { value: f64 },
    float32 { value: f32 }, // -3.4E+38 to +3.4E+38
    float64 { value: f64 }, // -1.7E+308 to +1.7E+308
    enum1 { value: u8 }, // enum
    boolean { value: bool },
    geo { lat: i32, lon: i32 },
    geo0 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo1 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo2 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: [u8; 32] }, // 32 bytes
    time_id { id: [u8; 24], time: u64 }, // 24 + 8 bytes
    date { value: u64 } // 8 bytes
}

pub struct NoProtoBufferItem<'a> {
    address: u32,
    item_ref: Box<&'a NoProtoDataModel>,
    data: NoProtoBufferScalar
}

pub struct NoProtoBuffer<'a> {
    bytes: Box<Vec<u8>>,
    ptr: u32,
    parsed: NoProtoBufferItem<'a>
}

impl<'a> NoProtoBuffer<'a> {

    pub fn new(baseModel: &'a NoProtoDataModel, length: Option<usize>, in_buffer: Option<Vec<u8>>) -> Self {

        match in_buffer { // parse existing buffer
            Some(x) => {
                let mut head: [u8; 4] = [0; 4];
                head.copy_from_slice(&x[0..4]);

                NoProtoBuffer {
                    ptr: x.len() as u32,
                    bytes: Box::new(x),
                    parsed: NoProtoBufferItem {
                        item_ref: Box::new(baseModel),
                        address: 0,
                        data: NoProtoBufferScalar::table { headPtr: u32::from_le_bytes(head) }
                    }
                }
            },
            None => { // make a new one
                let len = length.unwrap_or(1024); // 1kb default starting size

                NoProtoBuffer {
                    ptr: 0,
                    bytes: Box::new(Vec::with_capacity(len)),
                    parsed: NoProtoBufferItem {
                        item_ref: Box::new(baseModel),
                        address: 0,
                        data: NoProtoBufferScalar::table { headPtr: 0 }
                    }
                }
            }
        }
    }

    pub fn malloc(&mut self, memory: Vec<u8>) -> u32 {
        let location: u32 = self.ptr;
        self.ptr += memory.len() as u32;
        self.bytes.extend(memory);
        return location;
    }

    pub fn root(&self) {

    }

    pub fn get_bytes(&self)->&Vec<u8> {
        self.bytes.as_ref()
    }

    pub fn compact(&self) {

    }

    pub fn getWastedBytes(&self) -> i32 {
        return 0;
    }

    pub fn maybeCompact<F>(&self, mut callback: F) -> bool 
        where F: FnMut(i32) -> bool 
    {
        let doCompaction = callback(self.getWastedBytes());

        /*let mut hasher = FnvHasher::default();
        let bytes: [u8; 4] = [0, 0, 0, 0];
        hasher.write(&bytes);
        let hash: u64 = hasher.finish();*/

        if doCompaction {
            self.compact();
            true
        } else {
            false
        }
    }

}