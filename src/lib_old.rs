use std::result;
use json::*;
use fnv::FnvHasher;
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}


pub struct NoProtoMapModel {
    keyType: String,
    valueType: String
}

pub struct NoProtoDataModel {
    colKey: String,
    colType: String,
    options: JsonValue,
    table: Option<Box<HashMap<String, NoProtoDataModel>>>, // nested type (table)
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

    pub fn from_object(&self, model: JsonValue) -> Self {
        NoProtoFactory {
            loaded: true,
            dataModel: Box::new(NoProtoDataModel {
                colKey: "root".to_string(),
                colType: "root".to_string(),
                options: object!{},
                table: self.load_model_table(model),
                list: None,
                map: None
            })
        }
    }

    pub fn from_string(&self, model: &str) -> Result<Self> {
        match json::parse(model) {
            Ok(x) => Ok(self.from_object(x)),
            Err(e) => Err(e) 
        }
    }

    fn load_model_table(&self, model: JsonValue) -> Option<Box<HashMap<String, NoProtoDataModel>>> {

        let mut i: usize = 0;

        let length = model.len();

        let mut columns: HashMap<String, NoProtoDataModel> = HashMap::new();

        loop {
            if i < length {

                let model_row: &JsonValue = &model[i];

                let row_key = &model_row[0].as_str().unwrap_or("").to_owned();
                let row_type = &model_row[1].as_str().unwrap_or("").to_owned();
                let row_options = if model_row[2].is_null() { object!{} } else { model_row[2].clone() };

                columns.insert(row_key.to_string(), self.load_model_single(row_key.clone(), row_type.clone(), row_options));

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

            /* // can't figure out a good api for nested mps
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
            }*/

            let mut comma = 0;
            for (i, chr) in row_type.chars().enumerate() {
                let c_str = chr.to_string();
                if c_str == "," {
                    comma = i;
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
                    valueType: valueType.to_owned()
                    // model: Box::new(self.load_model_single("*".to_owned(), valueType.to_owned(), options.clone()))
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

enum NoProtoPointerTypes {
    table = 0,
    list = 1,
    map = 2,
    linked_item = 3, 
    utf8_string = 4,
    bytes = 5,
    int8 = 6,
    int16 = 7,
    int32 = 8,
    int64 = 9,
    uint8 = 10,
    uint16 = 11,
    uint32 = 12,
    uint64 = 13,
    float = 14, // -3.4E+38 to +3.4E+38
    double = 15, // -1.7E+308 to +1.7E+308
    option = 16, // enum
    boolean = 17,
    geo_64 = 18, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_32 = 19, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_16 = 20, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid = 21, // 32 bytes
    time_id = 22, // 24 + 8 bytes
    date = 23 // 8 bytes
}

pub enum NoProtoCollection {
    table = 0,
    list = 1,
    map = 2
}

// signed int: -2^(n - 1) to 2^(n - 1)
// unsigned int: (2^n) - 1
pub enum NoProtoScalar {
    utf8_string { value: String },
    bytes { value: Vec<u8> },
    int8 { value: i8 },
    int32 { value: i32 },
    int64 { value: i64 },
    uint8 { value: u8 },
    uint16 { value: u16 },
    uint32 { value: u32 },
    uint64 { value: u64 },
    float { value: f32 }, // -3.4E+38 to +3.4E+38
    double { value: f64 }, // -1.7E+308 to +1.7E+308
    option { value: u8 }, // enum
    boolean { value: bool },
    geo_64 { lat: f64, lon: f64 }, // (3.5nm resolution): two 64 bit float (16 bytes)
    geo_32 { lat: i32, lon: i32 }, // (16mm resolution): two 32 bit integers (8 bytes) Deg*10000000
    geo_16 { lat: i16, lon: i16 }, // (1.5km resolution): two 16 bit integers (4 bytes) Deg*100
    uuid { value: String }, // 32 bytes
    time_id { id: String, time: u64 }, // 24 + 8 bytes
    date { value: u64 } // 8 bytes
}

pub struct NoProtoPointer {
    address: u32,
    pointerType: NoProtoPointerTypes
}

pub struct NoProtoTable<'a> {
    buffer: &'a NoProtoBuffer<'a>,
    pointer: NoProtoPointer,
    head: u32,
    model: Box<&'a NoProtoDataModel>
}

impl<'a> NoProtoTable<'a> {

    fn new(buffer: &'a NoProtoBuffer, address: u32, model: Box<&'a NoProtoDataModel>, bytes: &Vec<u8>) -> Self {
        
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];
        head.copy_from_slice(&bytes[addr..(addr+4)]);
        

        NoProtoTable {
            buffer: buffer,
            pointer: NoProtoPointer {
                address: address,
                pointerType: NoProtoPointerTypes::table
            },
            head: u32::from_le_bytes(head),
            model: model
        }
    }

    fn set(&self, key: String, value: NoProtoScalar) -> std::result::Result<bool, &'static str> {
        Ok(true)
    }

    fn get(&self, key: String) -> Option<NoProtoScalar> {
        None
    }

    fn collection(&self, key: String) -> Option<NoProtoCollections> {
        Some(NoProtoCollections {
            list: None,
            map: None,
            table: None
        })
    }

    fn delete(&self, key: String) -> bool {
        false
    }

    fn clear(&self) {

    }

    fn has(&self, key: String) {

    }

}

pub struct NoProtoMap<'a> {
    pointer: NoProtoPointer,
    head: u32,
    model: Box<&'a NoProtoDataModel>
}

impl<'a> NoProtoMap<'a> {

    fn new(address: u32, model: Box<&'a NoProtoDataModel>, bytes: &Vec<u8>) -> Self {
        
        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];
        head.copy_from_slice(&bytes[addr..(addr+4)]);

         NoProtoMap {
            pointer: NoProtoPointer {
                address: address,
                pointerType: NoProtoPointerTypes::table
            },
            head: u32::from_le_bytes(head),
            model: model
        }
    }

    fn set(&self, key: Vec<u8>, value: NoProtoScalar) -> std::result::Result<bool, &'static str> {
        Ok(true)
    }

    fn get(&self, key: Vec<u8>) -> Option<NoProtoScalar> {
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

pub struct NoProtoList<'a> {
    pointer: NoProtoPointer,
    head: u32,
    size: u16,
    model: Box<&'a NoProtoDataModel>
}

impl<'a> NoProtoList<'a> {
    fn push() { // append

    }
    fn unshift() { // prepend
 
    }
    fn pop() { // pop off back

    }
    fn shift() { // shift off front

    }
    fn clear() {

    }
    fn entries() {

    }
    fn keys() {

    }
    fn values() {

    }
    fn forEach() {

    }
    fn splice(&self, start: u32, deleteCount: u32, items: Vec<NoProtoScalar>) {

    }
}

pub struct NoProtoCollections<'a> {
    list: Option<NoProtoList<'a>>,
    map: Option<NoProtoMap<'a>>,
    table: Option<NoProtoTable<'a>>
}

pub struct NoProtoBuffer<'a> {
    version: u8, // no proto buffer version (incase of bugs in specific version)
    bytes: Vec<u8>,
    ptr: u32,
    rootModel: &'a NoProtoDataModel
    // root: Option<Box<NoProtoTable<'a>>>
}

impl<'a> NoProtoBuffer<'a> {

     
    pub fn new(baseModel: &'a NoProtoDataModel, length: Option<usize>, in_buffer: Option<Vec<u8>>) -> Self {

        match in_buffer { // parse existing buffer
            Some(x) => {

                NoProtoBuffer {
                    version: *x.first().unwrap_or(&0), // get version number of buffer 
                    ptr: x.len() as u32, // get length of buffer
                    bytes: x, // store bytes,
                    rootModel: baseModel
                }
            },
            None => { // make a new one
                let len = length.unwrap_or(1024); // 1kb default starting size
                let mut x = Vec::with_capacity(len);

                NoProtoBuffer {
                    version: 0, // current NoProto protocol version
                    ptr: 5, // [version (u8) 1 byte, tableHead (u32) 4 bytes]
                    bytes: x, // store bytes,
                    rootModel: baseModel
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

    /*
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

        let address = self.alloc(vec![NoProtoPointerTypes::int8 as u8]);

        self.alloc(value.to_le_bytes().to_vec());

        NoProtoPointer {
            address: address,
            pointerType: NoProtoPointerTypes::int8
        }
    }

    pub fn mallocInt64(&mut self, value: i64) -> NoProtoPointer {
        self.mallocInt(value)
    }

    pub fn parseInt(&self, pointer: NoProtoPointer) -> std::result::Result<i64, &'static str> {
        let resolvedPtr = self.maybeIterType(pointer);

        let isInt64 = match resolvedPtr.pointerType {
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
*/
    pub fn parse_scalar(&self, pointer: &NoProtoPointer) -> std::result::Result<NoProtoScalar, &'static str> {

        match &pointer.pointerType {
            linked_item => {

            },
            utf8_string => {

            },
            bytes => {
                
            },
            int8 => {
                
            },
            int16 => {
                
            },
            int32 => {
                
            },
            int64 => {
                
            },
            uint8 => {
                
            },
            uint16 => {
                
            },
            uint32 => {
                
            },
            uint64 => {
                
            },
            float => {
                
            },
            double => {
                
            },
            float32 => {
                
            }, 
            float64 => {
                
            },
            option => {
                
            },
            boolean => {
                
            },
            geo_64  => {
                
            },
            geo_32  => {
                
            },
            geo_16  => {
                
            },
            uuid  => {
                
            },
            time_id  => {
                
            },
            date  => {
                
            },
            _ => {
                return Err("Wrong pointer type, not a scalar!");
            }
        };

        Ok(NoProtoScalar::boolean {value: false})
    }

    pub fn malloc_scalar(&self, scalar: &NoProtoScalar) -> std::result::Result<NoProtoPointer, &'static str> {

        let addr = self.ptr;

        match scalar {
            NoProtoScalar::utf8_string { value } => {

            }
            NoProtoScalar::bytes { value } => {

            },
            NoProtoScalar::int8 { value } => {

            },
            NoProtoScalar::int32 { value } => {

            },
            NoProtoScalar::int64 { value } => {

            },
            NoProtoScalar::uint8 { value } => {

            },
            NoProtoScalar::uint16 { value } => {

            },
            NoProtoScalar::uint32 { value } => {

            },
            NoProtoScalar::uint64 { value } => {

            },
            NoProtoScalar::float { value } => {

            },
            NoProtoScalar::double  { value } => {

            },
            NoProtoScalar::option { value } => {

            },
            NoProtoScalar::boolean { value } => {

            },
            NoProtoScalar::geo_64 { lat, lon } => {

            },
            NoProtoScalar::geo_32 { lat, lon } => {

            },
            NoProtoScalar::geo_16 { lat, lon } => {

            },
            NoProtoScalar::uuid { value } => {

            },
            NoProtoScalar::time_id { id, time } => {

            },
            NoProtoScalar::date { value } => {

            },
            _ => {
                return Err("Wrong pointer type, not a scalar!");
            }
        };

        Ok(NoProtoPointer {
            address: 0,
            pointerType: NoProtoPointerTypes::int8
        })
    }

    pub fn parse_collection(&self, pointer: &NoProtoPointer) -> NoProtoCollections {
        NoProtoCollections {
            list: None,
            map: None,
            table: None
        }
    }

    pub fn malloc_collection(&self, collectionType: &NoProtoCollection) -> NoProtoCollections {
        NoProtoCollections {
            list: None,
            map: None,
            table: None
        }
    }

    pub fn root(&self) -> NoProtoTable {
        NoProtoTable::new(&self, 1, Box::new(&self.rootModel), &self.bytes)
    }

    pub fn dump_bytes(self)->Vec<u8> {
        self.bytes
    }

    pub fn compact(&self)->&Vec<u8>  {
        self.bytes.as_ref()
    }

    pub fn calc_wasted_bytes(&self) -> u32 {
        return 0;
    }

    pub fn maybe_compact<F>(&self, mut callback: F) -> bool 
        where F: FnMut(u32) -> bool 
    {
        let do_compaction = callback(self.calc_wasted_bytes());

        if do_compaction {
            self.compact();
            true
        } else {
            false
        }
    }

}

fn main() {

    // build buffer factory (generates and parses buffers of this type)
    let user_factory = NoProtoFactory::new().from_object(array![
        array!["id",       "uuid", object!{}],
        array!["first",  "string", object!{}],
        array!["last",   "string", object!{}],
        array!["address", "table", object!{ 
            "model" => array![
                array!["street",  "string"],
                array!["city",    "string"],
                array!["state",   "string"],
                array!["zip",     "string"]
            ]
        }],
        array!["tags", "string[]"]
    ]);

    let mut jsonTest = object!{"key" => "value"};
    let val = &jsonTest["key"];
    assert_eq!(val, "value");
    

    // create new buffer of user type
    let userBuffer = user_factory.new_buffer(None).unwrap();
    
    // get buffer root table
    let root = &userBuffer.root();

    // set values 
    root.set("first".to_owned(), NoProtoScalar::utf8_string { value: "Billy".to_owned()});
    root.set("last".to_owned(), NoProtoScalar::utf8_string { value: "Joel".to_owned()});


    let bytes = userBuffer.dump_bytes();

    let newBuffer = user_factory.parse_buffer(bytes).unwrap();

    let firstName = newBuffer.root().get("first".to_owned()).unwrap();
    

    // let stringPtr = newBuffer.malloc_scalar(&NoProtoScalar::utf8_string { value: "hello".to_owned()}).unwrap();

    // newBuffer.root().set("column".to_owned(), NoProtoScalar::utf8_string { value: "hello".to_owned()});




    /*
    // create map in buffer
    let newMap = buf.mallocMap(tableItem).unwrap();

    // set values in map
    newMap.set("key".as_bytes().to_vec(), "value".to_owned());
    newMap.set("key2".as_bytes().to_vec(), "value2".to_owned());

    // assign map to table column
    root.set("map column".to_owned(), newMap.pointer);

    // let existingMap = buf.parseMap(root.get("map column".to_owned()).unwrap(), tableItem).unwrap();

    existingMap.set("key3".as_bytes().to_vec(), "value3".to_owned());

    // let string = buf.parseString(tableItem.pointer);

    // buf.alloc(NoProtoTable);*/
}