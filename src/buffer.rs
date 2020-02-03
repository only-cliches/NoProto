mod pointer;
pub use self::pointer::{*};



/*
pub struct NoProtoDataModel {
    colKey: String,
    colType: String,
    options: JsonValue,
    table: Option<HashMap<String, NoProtoDataModel>>, // nested type (table)
    list: Option<Box<NoProtoDataModel>>, // nested type (list)
    map: Option<Box<NoProtoDataModel>> // nested map type
}

pub struct NoProtoBuffer {
    ptr: u32,
    buffer: Vec<u8>,
    rootModel: NoProtoDataModel
}

*/