use std::result;
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

/*
// signed int: -2^(n - 1) to 2^(n - 1)
// unsigned int: (2^n) - 1
enum NoProtoBufferTypes {
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
*/

enum NoProtoPointerTypes {
    table = 0,
    tableItem = 1, 
    list = 2,
    listItem = 3,
    map = 4,
    mapItem = 5,
    utf8_string = 6,
    bytes = 7,
    int1 = 8, // int
    int8 = 9,
    int16 = 10,
    int32 = 11,
    int64 = 12,
    uint8 = 13,
    uint16 = 14,
    uint32 = 15,
    uint64 = 16,
    float = 17,
    double = 18,
    float32 = 19, // -3.4E+38 to +3.4E+38
    float64 = 20, // -1.7E+308 to +1.7E+308
    enum1 = 21, // enum
    boolean = 22,
    geo = 23,
    geo0 = 24, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo1 = 25, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo2 = 26, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid = 27, // 32 bytes
    time_id = 28, // 24 + 8 bytes
    date = 29 // 8 bytes
}

pub struct NoProtoPointer {
    address: u32,
    pointerType: NoProtoPointerTypes
}

pub struct NoProtoTable<'a> {
    pointer: NoProtoPointer,
    head: u32,
    model: Box<&'a NoProtoDataModel>
}

impl<'a> NoProtoTable<'a> {

    fn new(address: u32, model: Box<&'a NoProtoDataModel>, bytes: &Vec<u8>) -> Self {
        
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];
        head.copy_from_slice(&bytes[addr..(addr+4)]);

        NoProtoTable {
            pointer: NoProtoPointer {
                address: address,
                pointerType: NoProtoPointerTypes::table
            },
            head: u32::from_le_bytes(head),
            model: model
        }
    }

    fn set(&self, key: String, value: NoProtoPointer) -> std::result::Result<bool, &'static str> {
        Ok(true)
    }

    fn get(&self, key: String) -> Option<NoProtoPointer> {
        None
    }

    fn delete(&self, key: String) -> bool {
        false
    }

    fn clear(&self) {

    }

    fn has(&self, key: String) {

    }

}


pub struct NoProtoBuffer<'a> {
    version: u8, // no proto buffer version (incase of bugs in specific version)
    bytes: Box<Vec<u8>>,
    ptr: u32,
    root: Box<NoProtoTable<'a>>
}

impl<'a> NoProtoBuffer<'a> {

     
    pub fn new(baseModel: &'a NoProtoDataModel, length: Option<usize>, in_buffer: Option<Vec<u8>>) -> Self {

        match in_buffer { // parse existing buffer
            Some(x) => {

                NoProtoBuffer {
                    version: *x.first().unwrap_or(&0), // get version number of buffer 
                    ptr: x.len() as u32, // get length of buffer
                    root: Box::new(NoProtoTable::new(1, Box::new(baseModel), &x)), // parse root
                    bytes: Box::new(x) // store bytes
                }
            },
            None => { // make a new one
                let len = length.unwrap_or(1024); // 1kb default starting size
                let mut x = Vec::with_capacity(len);

                NoProtoBuffer {
                    version: 0, // current NoProto protocol version
                    ptr: 5, // [version (u8) 1 byte, tableHead (u32) 4 bytes]
                    root: Box::new(NoProtoTable::new(1, Box::new(baseModel), &x)),
                    bytes: Box::new(x)
                }
            }
        }
    }

    fn alloc(&mut self, bytes: Vec<u8>) -> u32 {

        let location: u32 = self.ptr;
        self.ptr += bytes.len() as u32;
        self.bytes.extend(bytes);
        return location;
    }

    fn maybeIterType(&self, ptr: NoProtoPointer) -> NoProtoPointer {
        match ptr.pointerType {
            NoProtoPointerTypes::tableItem => {
                ptr
            },
            NoProtoPointerTypes::mapItem => {
                ptr
            },
            NoProtoPointerTypes::listItem => {
                ptr
            },
            _ => {
                ptr
            }
        }
    }

    // [1 byte type (u8), 4 byte length (u32), string bytes...]
    pub fn mallocString(&mut self, value: String) -> NoProtoPointer {
        let strBytes = value.as_bytes();

        let length: u32 = strBytes.len() as u32;

        // write type to buffer and get address of this value
        let address = self.alloc(vec![NoProtoPointerTypes::utf8_string as u8]);

        // write string length to buffer
        self.alloc(length.to_le_bytes().to_vec());
        
        // write string to buffer
        self.alloc(strBytes.to_vec());

        NoProtoPointer {
            address: address,
            pointerType: NoProtoPointerTypes::utf8_string
        }
    }

    pub fn parseString(&self, pointer: NoProtoPointer) -> std::result::Result<String, &'static str> {
        
        let resolvedPtr = self.maybeIterType(pointer);
        
        match resolvedPtr.pointerType {
            NoProtoPointerTypes::utf8_string => {
                let addr = resolvedPtr.address as usize;
                let thisType = self.bytes[addr];

                if thisType != (NoProtoPointerTypes::utf8_string as u8) {
                    return Err("Attempted to parse string on non string value!");
                }

                let mut size: [u8; 4] = [0; 4];
                size.copy_from_slice(&self.bytes[(addr+1)..(addr+5)]);
                let length = u32::from_le_bytes(size) as usize;

                let mut strBytes: Vec<u8> = Vec::with_capacity(length);
                strBytes.copy_from_slice(&self.bytes[(addr+5)..(addr+length)]);

                match String::from_utf8(strBytes) {
                    Ok(x) => {
                        Ok(x)
                    },
                    Err(x) => {
                        Err("Error parsing utf-8 string!")
                    }
                }
            },
            _ => {
                Err("Can't parse non string!")
            }
        }
    }

    // [1 byte type, (u8), 8 byte value (i64)]
    pub fn mallocInt(&mut self, value: i64) -> NoProtoPointer {

        let address = self.alloc(vec![NoProtoPointerTypes::int1 as u8]);

        self.alloc(value.to_le_bytes().to_vec());

        NoProtoPointer {
            address: address,
            pointerType: NoProtoPointerTypes::int1
        }
    }

    pub fn mallocInt64(&mut self, value: i64) -> NoProtoPointer {
        self.mallocInt(value)
    }

    pub fn parseInt(&self, pointer: NoProtoPointer) -> std::result::Result<i64, &'static str> {
        let resolvedPtr = self.maybeIterType(pointer);

        let isInt64 = match resolvedPtr.pointerType {
            NoProtoPointerTypes::int1 => { true },
            NoProtoPointerTypes::int64 => { true },
            _ => { false }
        };

        if isInt64 {
            let addr = resolvedPtr.address as usize;
            let thisType = self.bytes[addr];

            if thisType != (NoProtoPointerTypes::int1 as u8) && thisType != (NoProtoPointerTypes::int64 as u8) {
                return Err("Attempted to parse int on non int value!");
            }

            let mut vBytes: [u8; 8] = [0; 8];
            vBytes.copy_from_slice(&self.bytes[(addr+1)..(addr+5)]);
            let value = i64::from_le_bytes(vBytes);

            Ok(value)
        } else {
            Err("Can't parse non int value!")
        }
    }

    pub fn root(&self) -> &Box<NoProtoTable> {
       &self.root
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

fn main() {
    let fact = NoProtoFactory::new();

    let buf = fact.new_buffer(None).unwrap();
    let root = &buf.root();
    
    let tableItem = root.get("hello".to_owned()).unwrap();
    // let string = buf.parseString(tableItem.pointer);

    // buf.alloc(NoProtoTable);
}