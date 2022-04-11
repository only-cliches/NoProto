use crate::map::NP_OrderedMap;
use core::fmt::Debug;
use alloc::vec::Vec;
use alloc::boxed::Box;


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NP_String_Casing {
    None,
    Uppercase,
    Lowercase
}

impl Default for NP_String_Casing {
    fn default() -> Self {
        Self::None
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum NP_Type<CHILD: Debug + PartialEq + Default, STR: Debug + PartialEq + Default> {
    Unknown,
    None,
    Any,
    Info,
    String      { default: STR, casing: NP_String_Casing, max_len: Option<usize> },
    Char        { default: char },
    Int8        { default: i8, min: Option<i8>, max: Option<i8> }, 
    Int16       { default: i16, min: Option<i16>, max: Option<i16> }, 
    Int32       { default: i32, min: Option<i32>, max: Option<i32> },
    Int64       { default: i64, min: Option<i64>, max: Option<i64> },
    Uint8       { default: u8, min: Option<u8>, max: Option<u8> }, 
    Uint16      { default: u16, min: Option<u16>, max: Option<u16> },
    Uint32      { default: u32, min: Option<u32>, max: Option<u32> },
    Uint64      { default: u64, min: Option<u64>, max: Option<u64> },
    Float32     { default: f32, min: Option<f32>, max: Option<f32> }, 
    Float64     { default: f64, min: Option<f64>, max: Option<f64> },
    Exp32       { default: i32, e: i8, min: Option<i32>, max: Option<i32>,  },
    Exp64       { default: i64, e: i16, min: Option<i64>, max: Option<i64>,  }, 
    Bool        { default: bool },
    Geo32       { default: (i16, i16) },
    Geo64       { default: (i32, i32) },
    Geo128      { default: (i64, i64) },
    Date        { default: u64 },
    Uuid, 
    Ulid,
    Vec         { of: Box<CHILD>, max_len: Option<usize> },
    List        { of: Box<CHILD> },
    Map         { of: Box<CHILD> },
    Box         { of: Box<CHILD> },
    Result      { ok: Box<CHILD>, err: Box<CHILD> },
    Option      { some: Box<CHILD> },
    Tuple       { children: Vec<CHILD> },
    Array       { of: Box<CHILD>, len: u16 },
    Struct      { children: NP_OrderedMap<CHILD> },
    Enum        { children: NP_OrderedMap<Option<CHILD>>, default: usize },
    Simple_Enum { children: Vec<STR>, default: usize },

    // Only used by NP_Buffer_Type
    RPC_Call    { id: u32, args: Vec<CHILD> },
    RPC_Return  { id: u32, value: Box<CHILD> },

    // Only used by NP_Schema_Type
    Impl        { methods: NP_OrderedMap<CHILD> },
    Method      { id: u32, args: NP_OrderedMap<CHILD>, returns: Box<CHILD> },
    Custom      { parent_schema_addr: usize, generic_args: Option<Vec<usize>> },
    Generic     { parent_schema_addr: usize, parent_generic_idx: usize },
    This        { parent_schema_addr: usize }
}

impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> Default for NP_Type<CHILD, STR> {
    fn default() -> Self {
        return NP_Type::Unknown
    }
}

impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> From<u8> for NP_Type<CHILD, STR> {
    fn from(value: u8) -> Self {
        match value {
            0  => NP_Type::Unknown,
            1  => NP_Type::None,
            2  => NP_Type::Any,
            3  => NP_Type::Info,
            4  => NP_Type::String        { default: Default::default(), casing: Default::default(), max_len: Default::default() },
            5  => NP_Type::Char          { default: Default::default() },
            6  => NP_Type::Int8          { default: Default::default(), min: Default::default(), max: Default::default() },
            7  => NP_Type::Int16         { default: Default::default(), min: Default::default(), max: Default::default() },
            8  => NP_Type::Int32         { default: Default::default(), min: Default::default(), max: Default::default() },
            9  => NP_Type::Int64         { default: Default::default(), min: Default::default(), max: Default::default() },
            10 => NP_Type::Uint8         { default: Default::default(), min: Default::default(), max: Default::default() },
            11 => NP_Type::Uint16        { default: Default::default(), min: Default::default(), max: Default::default() },
            12 => NP_Type::Uint32        { default: Default::default(), min: Default::default(), max: Default::default() },
            13 => NP_Type::Uint64        { default: Default::default(), min: Default::default(), max: Default::default() },
            14 => NP_Type::Float32       { default: Default::default(), min: Default::default(), max: Default::default() },
            15 => NP_Type::Float64       { default: Default::default(), min: Default::default(), max: Default::default() },
            16 => NP_Type::Exp32         { default: Default::default(), e: Default::default(), min: Default::default(), max: Default::default() },
            17 => NP_Type::Exp64         { default: Default::default(), e: Default::default(), min: Default::default(), max: Default::default() },
            18 => NP_Type::Bool          { default: Default::default() },
            19 => NP_Type::Geo32         { default: Default::default() },
            20 => NP_Type::Geo64         { default: Default::default() },
            21 => NP_Type::Geo128        { default: Default::default() },
            22 => NP_Type::Date          { default: Default::default() },
            23 => NP_Type::Uuid,
            24 => NP_Type::Ulid,
            25 => NP_Type::Vec           { of: Default::default(), max_len: Default::default() },
            26 => NP_Type::List          { of: Default::default() },
            27 => NP_Type::Map           { of: Default::default() },
            28 => NP_Type::Box           { of: Default::default() },
            29 => NP_Type::Result        { ok: Default::default(), err: Default::default() },
            30 => NP_Type::Option        { some: Default::default() },
            31 => NP_Type::Tuple         { children: Default::default() },
            32 => NP_Type::Array         { of: Default::default(), len: Default::default() },
            33 => NP_Type::Struct        { children: Default::default() },
            34 => NP_Type::Enum          { children: Default::default(), default: Default::default() },
            35 => NP_Type::Simple_Enum   { children: Default::default(), default: Default::default() },
            36 => NP_Type::RPC_Call      { id: Default::default(), args: Default::default() },
            37 => NP_Type::RPC_Return    { id: Default::default(), value: Default::default() },
            38 => NP_Type::Impl          { methods: Default::default() },
            39 => NP_Type::Method        { id: Default::default(), args: Default::default(), returns: Default::default() },
            40 => NP_Type::Custom        { parent_schema_addr: Default::default(), generic_args: Default::default() },
            41 => NP_Type::Generic       { parent_schema_addr: Default::default(), parent_generic_idx: Default::default() },
            _  => NP_Type::Unknown
        }
    }
}

impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> From<NP_Type<CHILD, STR>> for u8 {
    fn from(value: NP_Type<CHILD, STR>) -> Self {
        match value {
            NP_Type::Unknown             =>  0,
            NP_Type::None                =>  1,
            NP_Type::Any                 =>  2,
            NP_Type::Info                =>  3,
            NP_Type::String       { .. } =>  4,
            NP_Type::Char         { .. } =>  5,
            NP_Type::Int8         { .. } =>  6,
            NP_Type::Int16        { .. } =>  7,
            NP_Type::Int32        { .. } =>  8,
            NP_Type::Int64        { .. } =>  9,
            NP_Type::Uint8        { .. } => 10,
            NP_Type::Uint16       { .. } => 11,
            NP_Type::Uint32       { .. } => 12,
            NP_Type::Uint64       { .. } => 13,
            NP_Type::Float32      { .. } => 14,
            NP_Type::Float64      { .. } => 15,
            NP_Type::Exp32        { .. } => 16,
            NP_Type::Exp64        { .. } => 17,
            NP_Type::Bool         { .. } => 18,
            NP_Type::Geo32        { .. } => 19,
            NP_Type::Geo64        { .. } => 20,
            NP_Type::Geo128       { .. } => 21,
            NP_Type::Date         { .. } => 22,
            NP_Type::Uuid                => 23,
            NP_Type::Ulid                => 24,
            NP_Type::Vec          { .. } => 25,
            NP_Type::List         { .. } => 26,
            NP_Type::Map          { .. } => 27,
            NP_Type::Box          { .. } => 28,
            NP_Type::Result       { .. } => 29,
            NP_Type::Option       { .. } => 30,
            NP_Type::Tuple        { .. } => 31,
            NP_Type::Array        { .. } => 32,
            NP_Type::Struct       { .. } => 33,
            NP_Type::Enum         { .. } => 34,
            NP_Type::Simple_Enum  { .. } => 35,
            NP_Type::RPC_Call     { .. } => 36,
            NP_Type::RPC_Return   { .. } => 37,
            NP_Type::Impl         { .. } => 38,
            NP_Type::Method       { .. } => 39,
            NP_Type::Custom       { .. } => 40,
            NP_Type::Generic      { .. } => 41,
            NP_Type::This         { .. } => 42
        }
    }
}

impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> From<&str> for NP_Type<CHILD, STR> {
    fn from(value: &str) -> Self {
        match value {
            "?"       => NP_Type::Unknown,
            "none"    => NP_Type::None,
            "any"     => NP_Type::Any,
            "info"    => NP_Type::Info,
            "String"  => NP_Type::String        { default: Default::default(), casing: Default::default(), max_len: Default::default() },
            "char"    => NP_Type::Char          { default: Default::default() },
            "i8"      => NP_Type::Int8          { default: Default::default(), min: Default::default(), max: Default::default() },
            "i16"     => NP_Type::Int16         { default: Default::default(), min: Default::default(), max: Default::default() },
            "i32"     => NP_Type::Int32         { default: Default::default(), min: Default::default(), max: Default::default() },
            "i64"     => NP_Type::Int64         { default: Default::default(), min: Default::default(), max: Default::default() },
            "u8"      => NP_Type::Uint8         { default: Default::default(), min: Default::default(), max: Default::default() },
            "u16"     => NP_Type::Uint16        { default: Default::default(), min: Default::default(), max: Default::default() },
            "u32"     => NP_Type::Uint32        { default: Default::default(), min: Default::default(), max: Default::default() },
            "u64"     => NP_Type::Uint64        { default: Default::default(), min: Default::default(), max: Default::default() },
            "f32"     => NP_Type::Float32       { default: Default::default(), min: Default::default(), max: Default::default() },
            "f64"     => NP_Type::Float64       { default: Default::default(), min: Default::default(), max: Default::default() },
            "d32"     => NP_Type::Exp32         { default: Default::default(), e: Default::default(), min: Default::default(), max: Default::default() },
            "d64"     => NP_Type::Exp64         { default: Default::default(), e: Default::default(), min: Default::default(), max: Default::default() },
            "bool"    => NP_Type::Bool          { default: Default::default() },
            "g32"     => NP_Type::Geo32         { default: Default::default() },
            "g64"     => NP_Type::Geo64         { default: Default::default() },
            "g128"    => NP_Type::Geo128        { default: Default::default() },
            "date"    => NP_Type::Date          { default: Default::default() },
            "uuid"    => NP_Type::Uuid,
            "ulid"    => NP_Type::Ulid,
            "Vec"     => NP_Type::Vec           { of: Default::default(), max_len: Default::default() },
            "List"    => NP_Type::List          { of: Default::default() },
            "Map"     => NP_Type::Map           { of: Default::default() },
            "Box"     => NP_Type::Box           { of: Default::default() },
            "Result"  => NP_Type::Result        { ok: Default::default(), err: Default::default() },
            "Option"  => NP_Type::Option        { some: Default::default() },
            "struct"  => NP_Type::Struct        { children: Default::default() },
            "enum"    => NP_Type::Enum          { children: Default::default(), default: Default::default() },
            "impl"    => NP_Type::Impl          { methods: Default::default() },
            "self"    => NP_Type::This          { parent_schema_addr: Default::default() },
            _         => NP_Type::Unknown
            /*
            "enum"    => NP_Type::Simple_Enum   { children: Default::default() },
            _         => NP_Type::Tuple         { children: Default::default() },
            _         => NP_Type::Array         { of: Default::default(), len: Default::default() },
            _         => NP_Type::RPC_Call      { id: Default::default(), args: Default::default() },
            _         => NP_Type::RPC_Return    { id: Default::default(), value: Default::default() },
            _         => NP_Type::Method        { id: Default::default(), args: Default::default(), returns: Default::default() },
            _         => NP_Type::Custom        { parent_schema_addr: Default::default() },
            _         => NP_Type::Generic       { parent_schema_addr: Default::default(), parent_generic_idx: Default::default() },
             */
        }
    }
}



impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> From<&NP_Type<CHILD, STR>> for &str {
    fn from(value: &NP_Type<CHILD, STR>) -> Self {
        match value {
            NP_Type::Unknown             => "?",
            NP_Type::None                => "none",
            NP_Type::Any                 => "any",
            NP_Type::Info                => "info",
            NP_Type::String       { .. } => "String",
            NP_Type::Char         { .. } => "char",
            NP_Type::Int8         { .. } => "i8",
            NP_Type::Int16        { .. } => "i16",
            NP_Type::Int32        { .. } => "i32",
            NP_Type::Int64        { .. } => "i64",
            NP_Type::Uint8        { .. } => "u8",
            NP_Type::Uint16       { .. } => "u16",
            NP_Type::Uint32       { .. } => "u32",
            NP_Type::Uint64       { .. } => "u64",
            NP_Type::Float32      { .. } => "f32",
            NP_Type::Float64      { .. } => "f64",
            NP_Type::Exp32        { .. } => "d32",
            NP_Type::Exp64        { .. } => "d64",
            NP_Type::Bool         { .. } => "bool",
            NP_Type::Geo32        { .. } => "g32",
            NP_Type::Geo64        { .. } => "g64",
            NP_Type::Geo128       { .. } => "g128",
            NP_Type::Date         { .. } => "date",
            NP_Type::Uuid                => "uuid",
            NP_Type::Ulid                => "ulid",
            NP_Type::Vec          { .. } => "Vec",
            NP_Type::List         { .. } => "List",
            NP_Type::Map          { .. } => "Map",
            NP_Type::Box          { .. } => "Box",
            NP_Type::Result       { .. } => "Result",
            NP_Type::Option       { .. } => "Option",
            NP_Type::Tuple        { .. } => "Tuple",
            NP_Type::Array        { .. } => "Array",
            NP_Type::Struct       { .. } => "struct",
            NP_Type::Enum         { .. } => "enum",
            NP_Type::Simple_Enum  { .. } => "enum",
            NP_Type::RPC_Call     { .. } => "RPC Call",
            NP_Type::RPC_Return   { .. } => "RPC Return",
            NP_Type::Impl         { .. } => "impl",
            NP_Type::Method       { .. } => "method",
            NP_Type::Custom       { .. } => "custom",
            NP_Type::Generic      { .. } => "generic",
            NP_Type::This         { .. } => "self"
        }
    }
}

impl<CHILD: Default + Debug + PartialEq, STR: Debug + PartialEq + Default> NP_Type<CHILD, STR> {
    pub fn get_str(&self) -> &str {
        self.into()
    }
}

// impl NP_Types_Outer {

//     pub fn get_response_type_for_request(&self) -> Result<Self, NP_Error> {
//         match self.kind {
//             NP_Types::rpc_call { uid, func, .. } => {
//                 Ok(Self {
//                     kind: NP_Types::rpc_return { uid, func, of: Default::default() },
//                     schema_idx: self.schema_idx
//                 })
//             },
//             _ => Err(NP_Error::Custom { message: String::from("Attempted to generate response buffer from non request buffer!") })
//         }
//     }

//     pub fn generate_string(&self, schema: &Arc<NP_Schema>) -> String {
//         let mut result = String::from("");

//         let type_str: &str = self.kind.into();

//         let mut is_array: bool = false;

//         match type_str {
//             "tuple" => {
//                 is_array = true;
//                 result.push_str("(");
//                 if let Some(generics) = &self.generics {
//                     result.push_str(generics.iter().map(|item| item.generate_string(&schema)).collect::<Vec<String>>().join(", ").as_str());
//                 }
//                 result.push_str(")");
//             }
//             "rpc" => {
//                 if let NP_Types::rpc_request { idx, func, uid } = self.kind {
//                     if let Some(type_data) = schema.id_index.get(idx) {
//                         let parsed_schema = &schema.schemas[type_data.data];
//                         if let Some(name) = parsed_schema.name {
//                             result.push_str(name.read_bytes(&schema.source));
//                         }
//                     }
//                 }
//                 if let NP_Types::rpc_response { idx, func, uid } = self.kind {
//                     if let Some(type_data) = schema.id_index.get(idx) {
//                         let parsed_schema = &schema.schemas[type_data.data];
//                         if let Some(name) = parsed_schema.name {
//                             result.push_str(name.read_bytes(&schema.source));
//                         }
//                     }
//                 }
//             },
//             "array" => {
//                 is_array = true;
//                 result.push_str("[");
//                 if let Some(gen) = &self.generics {
//                     result.push_str(gen[0].generate_string(&schema).as_str());
//                     result.push_str("; ");
//                 }
//                 if let NP_Types::array { len, .. } = self.kind {
//                     result.push_str(len.to_string().as_str());
//                 }
//                 if let NP_Types::small_array { len, .. } = self.kind {
//                     result.push_str(len.to_string().as_str());
//                 }
//                 result.push_str("]");
//             },
//             "custom" => {
//                 if let NP_Types::custom { idx } = self.kind {
//                     if let Some(type_data) = schema.id_index.get(idx) {
//                         let parsed_schema = &schema.schemas[type_data.data];
//                         if let Some(name) = parsed_schema.name {
//                             result.push_str(name.read_bytes(&schema.source));
//                         }
//                     }
//                 }
//                 if let NP_Types::small_custom { idx } = self.kind {
//                     if let Some(type_data) = schema.id_index.get(idx) {
//                         let parsed_schema = &schema.schemas[type_data.data];
//                         if let Some(name) = parsed_schema.name {
//                             result.push_str(name.read_bytes(&schema.source));
//                         }
//                     }
//                 }
//             },
//             _=> {
//                 result.push_str(type_str);
//             }
//         }

//         if is_array == false {
//             if let Some(generics) = &self.generics {
//                 result.push_str("<");
//                 result.push_str(generics.iter().map(|item| item.generate_string(&schema)).collect::<Vec<String>>().join(", ").as_str());
//                 result.push_str(">");
//             }
//         }

//         if type_str == "rpc" {
//             result.push_str(".");
//             if let NP_Types::rpc_request { idx, func, uid } = self.kind {
//                 if let Some(type_data) = schema.id_index.get(idx) {
//                     if let Some(methods) = type_data.methods {
//                         if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
//                             for (id, (hash, key)) in children.keys().iter().enumerate() {
//                                 if id == func {
//                                     result.push_str(key.as_str());
//                                 }
//                             }
//                         }
//                     }

//                 }
//             }
//             if let NP_Types::rpc_response { idx, func, uid } = self.kind {
//                 if let Some(type_data) = schema.id_index.get(idx) {
//                     if let Some(methods) = type_data.methods {
//                         if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
//                             for (id, (hash, key)) in children.keys().iter().enumerate() {
//                                 if id == func {
//                                     result.push_str(key.as_str());
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }

//         result
//     }

//     pub fn get_bytes(&self) -> Result<(u8, [u8; 16]), NP_Error> { // length, (bytes)
//         let mut length = 1usize;
//         let mut bytes: [u8; 16] = Default::default();

//         bytes[0] = self.kind.into();

//         match &self.kind {
//             NP_Types::array { len, size } => {
//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u16, b_ptr, len);
//                 length += 2;
//             },
//             NP_Types::small_array { len, size } => {
//                 bytes[length] = *len;
//                 length += 1;
//             },
//             NP_Types::custom { idx } => {
//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u16, b_ptr, idx);
//                 length += 2;
//             },
//             NP_Types::small_custom { idx } => {
//                 bytes[length] = *idx as u8;
//                 length += 1;
//             },
//             NP_Types::rpc_request { idx, func, uid } => {
//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u32, b_ptr, uid);
//                 length += 4;

//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u16, b_ptr, idx);
//                 length += 2;

//                 bytes[length] = *func as u8;
//                 length += 1;
//             },
//             NP_Types::rpc_response { idx, func, uid } => {
//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u32, b_ptr, uid);
//                 length += 4;

//                 let b_ptr = &mut bytes[length];
//                 le_bytes_write!(u16, b_ptr, idx);
//                 length += 2;

//                 bytes[length] = *func as u8;
//                 length += 1;
//             },
//             NP_Types::tuple { len, size } => {
//                 bytes[length] = *len;
//                 length += 1;
//             },
//             _ => { }
//         }


//         if let Some(generics) = &self.generics {
//             for (_idx, g) in generics.iter().enumerate() {
//                 let (new_length , new_bytes) = g.get_bytes()?;
//                 if new_length as usize + length >= bytes.len() {
//                     return Err(NP_Error::Custom { message: String::from("Too many buffer types, buffer schema overflow!") })
//                 }
//                 let mut i: usize = 0;
//                 while i < new_length as usize {
//                     bytes[length] = new_bytes[i];
//                     length += 1;
//                     i += 1;
//                 }
//             }
//         }

//         Ok((length as u8, bytes))
//     }

//     pub fn from_bytes(bytes: &[u8], schema: &Arc<NP_Schema>) -> Result<(usize, Self), NP_Error> {

//         if bytes.len() == 0 {
//             return Err(NP_Error::OutOfBounds)
//         }

//         let mut index = 0usize;

//         let mut kind: NP_Types = bytes[index].into();

//         index += 1;

//         match &mut kind {
//             NP_Types::tuple { len, size } => {
//                 if bytes.len() < index + 1 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 *len = bytes[index];
//                 index += 1;
//             },
//             NP_Types::rpc_request { idx, func, uid } => {
//                 if bytes.len() < index + 7 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 let ptr = &bytes[index];
//                 *uid = le_bytes_read!(u32, ptr);
//                 index += 4;

//                 let ptr = &bytes[index];
//                 *idx = le_bytes_read!(u16, ptr) as usize;
//                 index += 2;

//                 *func = bytes[index] as usize;
//                 index += 1;
//             },
//             NP_Types::rpc_response { idx, func, uid } => {
//                 if bytes.len() < index + 7 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 let ptr = &bytes[index];
//                 *uid = le_bytes_read!(u32, ptr);
//                 index += 4;

//                 let ptr = &bytes[index];
//                 *idx = le_bytes_read!(u16, ptr) as usize;
//                 index += 2;

//                 *func = bytes[index] as usize;
//                 index += 1;
//             },
//             NP_Types::custom { idx } => {
//                 if bytes.len() < index + 2 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 let ptr = &bytes[index];
//                 *idx = le_bytes_read!(u16, ptr) as usize;
//                 index += 2;

//             },
//             NP_Types::small_custom { idx } => {
//                 if bytes.len() < index + 1 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 *idx = bytes[index] as usize;
//                 index += 1;
//             },
//             NP_Types::array { len, size } => {
//                 if bytes.len() < index + 2 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 let ptr = &bytes[index];
//                 *len = le_bytes_read!(u16, ptr);
//                 index += 2;
//             },
//             NP_Types::small_array { len, size } => {
//                 if bytes.len() < index + 1 {
//                     return Err(NP_Error::OutOfBounds)
//                 }

//                 *len = bytes[index];
//                 index += 1;
//             },
//             NP_Types::none => {
//                 return Err(NP_Error::Custom { message: String::from("Error parsing buffer type: unknown data type!") })
//             }
//             _ => { }
//         }

//         let mut generic_length: usize = Self::read_generic_length(&kind, schema);

//         // parse generics
//         if generic_length > 0 {
//             let mut generics: Vec<NP_Types_Outer> = Vec::with_capacity(generic_length);

//             while generic_length > 0 {

//                 let (add_len, parsed) = Self::from_bytes(&bytes[index..], schema)?;
//                 index += add_len;
//                 generics.push(parsed);

//                 generic_length -= 1;
//             }

//             Ok((index, Self {
//                 kind,
//                 generics: Some(generics)
//             }))
//         } else  {

//             Ok((index, Self {
//                 kind,
//                 generics: None
//             }))
//         }

//     }

//     #[inline(always)]
//     fn read_generic_length(kind: &NP_Types, schema: &Arc<NP_Schema>) -> usize {
//         match kind {
//             NP_Types::vec { .. } => 1,
//             NP_Types::map { .. } => 1,
//             NP_Types::_box { .. } => 1,
//             NP_Types::result { .. } => 2,
//             NP_Types::option { .. } => 1,
//             NP_Types::array { .. } => 1,
//             NP_Types::small_array { .. } => 1,
//             NP_Types::tuple { len, size } => *len as usize,
//             NP_Types::rpc_response { idx, func, uid } => {
//                 if let Some(custom) = schema.id_index.get(*idx) {
//                     let custom_type = &schema.schemas[custom.data];
//                     if let NP_Parsed_Generics::Arguments(idx, args) = &custom_type.generics {
//                         args.len()
//                     } else {
//                         0
//                     }
//                 } else {
//                     0
//                 }
//             }
//             NP_Types::rpc_request { idx, func, uid } => {
//                 if let Some(custom) = schema.id_index.get(*idx) {
//                     let custom_type = &schema.schemas[custom.data];
//                     if let NP_Parsed_Generics::Arguments(idx, args) = &custom_type.generics {
//                         args.len()
//                     } else {
//                         0
//                     }
//                 } else {
//                     0
//                 }
//             },
//             NP_Types::small_custom { idx } => {
//                 if let Some(custom) = schema.id_index.get(*idx) {
//                     let custom_type = &schema.schemas[custom.data];
//                     if let NP_Parsed_Generics::Arguments(idx, args) = &custom_type.generics {
//                         args.len()
//                     } else {
//                         0
//                     }
//                 } else {
//                     0
//                 }
//             },
//             NP_Types::custom { idx } => {
//                 if let Some(custom) = schema.id_index.get(*idx) {
//                     let custom_type = &schema.schemas[custom.data];
//                     if let NP_Parsed_Generics::Arguments(idx, args) = &custom_type.generics {
//                         args.len()
//                     } else {
//                         0
//                     }
//                 } else {
//                     0
//                 }
//             },
//             _ => 0,
//         }
//     }

//     pub fn parse_type_prc(rpc_type: &buffer_rpc, data_type: &str, schema: &Arc<NP_Schema>) -> Result<Option<Self>, NP_Error> {

//         let mut dot_pos: Option<usize> = None;

//         for (idx, char) in data_type.chars().enumerate() {
//             if char == '.' {
//                 if None == dot_pos {
//                     dot_pos = Some(idx);
//                 } else {
//                     return Err(NP_Error::Custom { message: String::from("Multiple dot paths detected in rpc call.") })
//                 }
//             }
//         }



//         if let Some(idx) = dot_pos {
//             let mut root_type = NP_Error::unwrap(Self::parse_type( &data_type[0..idx], schema)?)?;

//             let custom_type_idx = match &root_type.kind {
//                 NP_Types::custom { idx } => { *idx },
//                 NP_Types::small_custom { idx } => { *idx },
//                 _ => {
//                     return Err(NP_Error::Custom { message: String::from("RPC request did not find custom type!") })
//                 }
//             };

//             match &rpc_type {
//                 buffer_rpc::request => {
//                     root_type.kind = NP_Types::rpc_request { idx: custom_type_idx, func: 0, uid: 0 };
//                 },
//                 buffer_rpc::response => {
//                     root_type.kind = NP_Types::rpc_response { idx: custom_type_idx, func: 0, uid: 0 };
//                 },
//                 _ => { }
//             }

//             let method_name = &data_type[(idx + 1)..data_type.len()];

//             if let NP_Types::rpc_request { idx, func, uid } = &mut root_type.kind {
//                 *uid = schema.unique_id;

//                 if let Some(type_data) = schema.id_index.get(*idx) {
//                     if let Some(methods) = type_data.methods {
//                         if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
//                             for (id, (hash, key)) in children.keys().iter().enumerate() {
//                                 if key == method_name {
//                                     *func = id;
//                                 }
//                             }
//                         }
//                     }

//                 }
//             }
//             if let NP_Types::rpc_response { idx, func, uid } = &mut root_type.kind {
//                 *uid = schema.unique_id;

//                 if let Some(type_data) = schema.id_index.get(*idx) {
//                     if let Some(methods) = type_data.methods {
//                         if let NP_Schema_Type::Impl { children } = &schema.schemas[methods].data_type {
//                             for (id, (hash, key)) in children.keys().iter().enumerate() {
//                                 if key == method_name {
//                                     *func = id;
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }


//             Ok(Some(root_type))
//         } else {
//             Err(NP_Error::Custom { message: String::from("No method call found in rpc request!") })
//         }
//     }

//     pub fn parse_type(data_type: &str, schema: &Arc<NP_Schema>) -> Result<Option<Self>, NP_Error> {

//         if data_type.len() > 255 {
//             return Err(NP_Error::Custom { message: String::from("Buffer schemas cannot be longer than 255 characters!") })
//         }

//         if data_type.trim() == "" {
//             return Ok(None);
//         }

//         // unit type
//         if data_type.trim() == "()" {
//             return Ok(Some(Self {
//                 kind: NP_Types::tuple { len : 0, size: 0 },
//                 generics: None
//             }));
//         }

//         let mut has_generics = false;
//         let mut angle_counter: isize = 0;
//         for char in data_type.chars() {
//             if char == '<' || char == '[' || char == '(' {
//                 angle_counter += 1;
//                 has_generics = true;
//             }
//             if char == '>' || char == ']' || char == ')' {
//                 angle_counter -= 1;
//             }
//         }

//         if angle_counter != 0 {
//             return Err(NP_Error::Custom { message: String::from("Missing matching brackets!")})
//         }



//         if has_generics { // slow path :(

//             let mut size = 0u32;

//             let mut result = Self {
//                 kind: NP_Types::none,
//                 generics: Some(Vec::new())
//             };

//             #[derive(Debug, PartialEq)]
//             enum parse_state {
//                 searching,
//                 angle_bracket,
//                 square_bracket,
//                 parans
//             }

//             let mut angle_step = 0isize;
//             let mut square_step = 0isize;
//             let mut paran_step = 0isize;

//             let mut p_state = parse_state::searching;

//             let mut parse_cursor: (usize, usize) = (0, 0); // (start_idx, end_idx)

//             let chars = data_type.as_bytes();
//             while parse_cursor.1 < data_type.len() {

//                 match chars[parse_cursor.1] as char {
//                     '(' => {
//                         if paran_step == 0 && p_state == parse_state::searching {
//                             result.kind = NP_Types::tuple { len: 0, size: 0 };
//                             parse_cursor.0 = parse_cursor.1 + 1;
//                             p_state = parse_state::parans;
//                         }
//                         paran_step += 1;
//                     },
//                     ')' => {
//                         paran_step -= 1;

//                         if paran_step == 0 && p_state == parse_state::parans {
//                             let inner_type = Self::parse_type(data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
//                             if let Some(generics) = &mut result.generics {
//                                 if let Some(i_type) = inner_type {
//                                     size += i_type.kind.get_size(&schema);
//                                     generics.push(i_type);
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;

//                             p_state = parse_state::searching;
//                         }
//                     },
//                     '<' => { // generic xxx<xxx, xxxx, xxxx>

//                         if angle_step == 0 && p_state == parse_state::searching {
//                             let str_kind = data_type[parse_cursor.0..parse_cursor.1].trim();
//                             result.kind = str_kind.into();

//                             if let NP_Types::custom { idx } = &mut result.kind {
//                                 if let Some(custom_kind) = schema.name_index.get(str_kind) {
//                                     if let Some(id) = schema.schemas[custom_kind.data].id {
//                                         *idx = id as usize;
//                                     }

//                                 } else {
//                                     let mut msg = String::from("Unknown type found!: ");
//                                     msg.push_str(str_kind);
//                                     return Err(NP_Error::Custom { message: msg });
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;

//                             p_state = parse_state::angle_bracket;
//                         }

//                         angle_step += 1;

//                     },
//                     '>' => {
//                         angle_step -= 1;

//                         if angle_step == 0 && p_state == parse_state::angle_bracket {
//                             let inner_type = Self::parse_type(data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
//                             if let Some(generics) = &mut result.generics {
//                                 if let Some(i_type) = inner_type {
//                                     size += i_type.kind.get_size(&schema);
//                                     generics.push(i_type);
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;

//                             p_state = parse_state::searching;
//                         }
//                     },
//                     ';' => {
//                         if square_step == 1 && p_state == parse_state::square_bracket {
//                             parse_cursor.0 += 1;
//                             let inner_type = Self::parse_type(data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
//                             if let Some(generics) = &mut result.generics {
//                                 if let Some(i_type) = inner_type {
//                                     size += i_type.kind.get_size(&schema);
//                                     generics.push(i_type);
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;
//                         }
//                     },
//                     ',' => {
//                         if paran_step == 0 && angle_step == 1 && p_state == parse_state::angle_bracket {
//                             let inner_type = Self::parse_type( data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
//                             if let Some(generics) = &mut result.generics {
//                                 if let Some(i_type) = inner_type {
//                                     size += i_type.kind.get_size(&schema);
//                                     generics.push(i_type);
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;
//                         }

//                         if angle_step == 0 && paran_step == 1 && p_state == parse_state::parans {
//                             let inner_type = Self::parse_type( data_type[parse_cursor.0..parse_cursor.1].trim(), schema)?;
//                             if let Some(generics) = &mut result.generics {
//                                 if let Some(i_type) = inner_type {
//                                     size += i_type.kind.get_size(&schema);
//                                     generics.push(i_type);
//                                 }
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;
//                         }
//                     },
//                     '[' => { // array [X; number]
//                         if square_step == 0 && p_state == parse_state::searching {
//                             parse_cursor.0 = parse_cursor.1;
//                             p_state = parse_state::square_bracket;
//                         }
//                         square_step +=1;
//                     },
//                     ']' => {
//                         square_step -=1;
//                         if square_step == 0 && p_state == parse_state::square_bracket {
//                             if let Ok(count) = data_type[parse_cursor.0..parse_cursor.1].trim().parse::<u16>() {
//                                 result.kind = NP_Types::array { len: count, size: size * (count as u32) };
//                             } else {
//                                 return Err(NP_Error::Custom { message: String::from("Error parsing array length!")})
//                             }

//                             parse_cursor.0 = parse_cursor.1 + 1;
//                             p_state = parse_state::searching;
//                         }
//                     },
//                     _ => { }
//                 }

//                 parse_cursor.1 += 1;
//             }

//             if square_step != 0 {
//                 return Err(NP_Error::Custom { message: String::from("Missing matching square brackets!")});
//             }

//             if angle_step != 0 {
//                 return Err(NP_Error::Custom { message: String::from("Missing matching angle brackets!")});
//             }

//             if paran_step != 0 {
//                 return Err(NP_Error::Custom { message: String::from("Missing matching parentheses!")});
//             }

//             let gen_count = if let Some(x) = &result.generics {
//                 x.len()
//             } else {
//                 0
//             };

//             if gen_count == 0 {
//                 result.generics = None;
//             }

//             if let NP_Types::tuple { len, size: tuple_size } = &mut result.kind {
//                 *len = gen_count as u8;
//                 *tuple_size = size;
//             }

//             let gen_length = Self::read_generic_length(&result.kind, &schema);

//             if gen_length != gen_count {
//                 let mut msg = String::from("Wrong number of generic params. Type requires this many params:");
//                 msg.push_str(gen_length.to_string().as_str());
//                 return Err(NP_Error::Custom { message: msg});
//             }



//             match result.kind.clone() {
//                 NP_Types::custom { idx } => {
//                     if idx < 255 {
//                         result.kind = NP_Types::small_custom { idx };
//                     }
//                 },
//                 NP_Types::array { len, size } => {
//                     if len < 255 {
//                         result.kind = NP_Types::small_array { len: len as u8, size };
//                     }
//                 },
//                 _ => { }
//             }

//             Ok(Some(result))
//         } else { // fast path
//             let mut this_type: NP_Types = data_type.into();


//             if let NP_Types::custom { idx } = &mut this_type {
//                 if let Some(custom_kind) = schema.name_index.get(data_type.trim()) {
//                     if let Some(id) = schema.schemas[custom_kind.data].id {
//                         *idx = id as usize;
//                     } else {
//                         return Err(NP_Error::Custom { message: String::from("Cannot use custom types that don't have an id!")})
//                     }
//                 } else {
//                     let mut msg = String::from("Unknown type found!: ");
//                     msg.push_str(data_type);
//                     return Err(NP_Error::Custom { message: msg });
//                 }
//             }

//             let gen_length = Self::read_generic_length(&this_type, &schema);

//             // should we have generic params?
//             if gen_length != 0 {
//                 let mut msg = String::from("Generic params required but none provided. Type requires this many params: ");
//                 msg.push_str(gen_length.to_string().as_str());
//                 return Err(NP_Error::Custom { message: msg});
//             }




//             if let NP_Types::custom { idx } = this_type.clone() {
//                 if idx < 255 {
//                     this_type = NP_Types::small_custom { idx };
//                 }
//             }

//             Ok(Some(Self {
//                 kind: this_type,
//                 generics: None
//             }))
//         }
//     }
// }
//
// #[cfg(test)]
// mod schema_tests {
//     use crate::schema::NP_Schema;
//     use alloc::sync::Arc;
//     use crate::error::NP_Error;
//     use crate::buffer::type_parser::{NP_Type, NP_Types};
//     use crate::buffer::buffer_rpc;
//
//
//     fn type_parse_schema() -> Result<Arc<NP_Schema>, NP_Error> {
//         let schema = r##"
//             info [
//                 id: "my-spec",
//                 version: 2.0,
//                 email: "someone@gmail.com",
//                 nothing: null
//             ]
//
//             struct myType<X> [id: 10] {
//                 username: string [max_len: 16],
//                 password: string
//             }
//
//             impl myType<X> {
//                 get(ulid) -> Option<self>,
//                 set(self) -> Result<(), string>
//             }
//
//             struct anotherType [id: 9] {
//                 username: string [list: [0, 1, 2, 3, 4], values: [key: true, another: false]]
//             }
//
//             struct genericCity<A, B, C, D, E, F, G, H> [id: 11] {
//                 emaill: string
//             }
//
//             struct bigType [id: 500] {
//                 username: string
//             }
//
//             impl bigType {
//                 get(ulid) -> Option<self>,
//                 set(self) -> Result<(), string>
//             }
//         "##;
//         let parsed = Arc::new(NP_Schema::parse(schema)?);
//
//         // unsafe { core::str::from_utf8_unchecked(&parsed.to_bytes()?) }
//         // &parsed.to_bytes()?
//         // println!("{:?} {} {}", &parsed.to_bytes()?, parsed.to_bytes()?.len(), schema.len());
//         // println!("{:#?}", schema);
//
//         Ok(parsed)
//     }
//
//     #[test]
//     fn simple_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "myType<u32>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_custom { idx: 10 },
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::u32,
//                 generics: None
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "myType<u32>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn vec_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Vec<u32>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::vec,
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::u32,
//                 generics: None
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Vec<u32>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn crazy_nesting_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Vec<Vec<Vec<Vec<u32>>>>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::vec,
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::vec,
//                 generics: Some(vec![NP_Type {
//                     kind: NP_Types::vec,
//                     generics: Some(vec![NP_Type {
//                         kind: NP_Types::vec,
//                         generics: Some(vec![NP_Type {
//                             kind: NP_Types::u32,
//                             generics: None
//                         }])
//                     }])
//                 }])
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Vec<Vec<Vec<Vec<u32>>>>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn super_simple_custom_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "anotherType", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_custom { idx: 9 },
//             generics: None
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         assert_eq!(from_bytes_type.generate_string(&schema), "anotherType");
//
//         Ok(())
//     }
//
//     #[test]
//     fn simple_custom_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "bigType", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::custom { idx: 500 },
//             generics: None
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "bigType");
//
//         Ok(())
//     }
//
//     #[test]
//     fn simple_array_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "[bool; 20]", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_array { len: 20, size: 0 },
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::bool,
//                 generics: None
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "[bool; 20]");
//
//         Ok(())
//     }
//
//     #[test]
//     fn large_array_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "[bool; 500]", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::array { len: 500, size: 0 },
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::bool,
//                 generics: None
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "[bool; 500]");
//
//         Ok(())
//     }
//
//     #[test]
//     fn custom_nested_array_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "myType<[bool; 20]>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_custom { idx: 10 },
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::small_array { len: 20, size: 0 },
//                 generics: Some(vec![NP_Type {
//                     kind: NP_Types::bool,
//                     generics: None
//                 }])
//             }])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "myType<[bool; 20]>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn crazy_generics_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "genericCity<u32, i64, bool, u64, string, uuid, ulid, date>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_custom { idx: 11 },
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::u32,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::i64,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::bool,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::u64,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::uuid,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::ulid,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::date,
//                     generics: None
//                 },
//             ])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "genericCity<u32, i64, bool, u64, string, uuid, ulid, date>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn crazy_generics_type_test_2() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "genericCity<u32, i64, myType<[bool; 20]>, u64, string, uuid, ulid, date>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::small_custom { idx: 11 },
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::u32,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::i64,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::small_custom { idx: 10 },
//                     generics: Some(vec![NP_Type {
//                         kind: NP_Types::small_array { len: 20, size: 0 },
//                         generics: Some(vec![NP_Type {
//                             kind: NP_Types::bool,
//                             generics: None
//                         }])
//                     }])
//                 },
//                 NP_Type {
//                     kind: NP_Types::u64,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::uuid,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::ulid,
//                     generics: None
//                 },
//                 NP_Type {
//                     kind: NP_Types::date,
//                     generics: None
//                 },
//             ])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "genericCity<u32, i64, myType<[bool; 20]>, u64, string, uuid, ulid, date>");
//
//         Ok(())
//     }
//
//
//
//     #[test]
//     fn result_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Result<[bool; 20], string>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::result,
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::small_array { len: 20, size: 0 },
//                     generics: Some(vec![NP_Type {
//                         kind: NP_Types::bool,
//                         generics: None
//                     }])
//                 },
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 },
//             ])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Result<[bool; 20], string>");
//
//         Ok(())
//     }
//
//
//     #[test]
//     fn tuple_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "([bool; 20], string)", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::tuple { len: 2, size: 0 },
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::small_array { len: 20, size: 0 },
//                     generics: Some(vec![NP_Type {
//                         kind: NP_Types::bool,
//                         generics: None
//                     }])
//                 },
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 },
//             ])
//         });
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "([bool; 20], string)");
//
//         Ok(())
//     }
//
//     #[test]
//     fn complex_nested_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Vec<([bool; 20], string)>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::vec,
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::tuple { len: 2, size: 0 },
//                 generics: Some(vec![
//                     NP_Type {
//                         kind: NP_Types::small_array { len: 20, size: 0 },
//                         generics: Some(vec![NP_Type {
//                             kind: NP_Types::bool,
//                             generics: None
//                         }])
//                     },
//                     NP_Type {
//                         kind: NP_Types::string,
//                         generics: None
//                     },
//                 ])
//             }])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Vec<([bool; 20], string)>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn complex_nested_type_test_2() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Vec<([bool; 20], string)>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::vec,
//             generics: Some(vec![NP_Type {
//                 kind: NP_Types::tuple { len: 2, size: 0 },
//                 generics: Some(vec![
//                     NP_Type {
//                         kind: NP_Types::small_array { len: 20, size: 0 },
//                         generics: Some(vec![NP_Type {
//                             kind: NP_Types::bool,
//                             generics: None
//                         }])
//                     },
//                     NP_Type {
//                         kind: NP_Types::string,
//                         generics: None
//                     },
//                 ])
//             }])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Vec<([bool; 20], string)>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn complex_nested_type_test_3() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Result<([bool; 20], string), string>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::result,
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::tuple { len: 2, size: 0 },
//                     generics: Some(vec![
//                         NP_Type {
//                             kind: NP_Types::small_array { len: 20, size: 0 },
//                             generics: Some(vec![NP_Type {
//                                 kind: NP_Types::bool,
//                                 generics: None
//                             }])
//                         },
//                         NP_Type {
//                             kind: NP_Types::string,
//                             generics: None
//                         },
//                     ])
//                 },
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 }
//             ])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Result<([bool; 20], string), string>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn unit_type_test() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "( )", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::tuple { len: 0, size: 0 },
//             generics: None
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "()");
//
//         Ok(())
//     }
//
//     #[test]
//     fn unit_type_test_2() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type( "Vec<()>", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::vec,
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::tuple { len: 0, size: 0 },
//                     generics: None
//                 }
//             ])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "Vec<()>");
//
//         Ok(())
//     }
//
//     #[test]
//     fn rpc_type_test_1() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type_prc(&buffer_rpc::request, "bigType.set", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::rpc_request { idx: 500, func: 1, uid: 4204945332 },
//             generics: None
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "bigType.set");
//
//         Ok(())
//     }
//
//     #[test]
//     fn rpc_type_test_2() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type_prc(&buffer_rpc::request, "myType<string>.set", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::rpc_request { idx: 10, func: 1, uid: 4204945332 },
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 }
//             ])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "myType<string>.set");
//
//         Ok(())
//     }
//
//     #[test]
//     fn rpc_type_test_3() -> Result<(), NP_Error> {
//         let schema = type_parse_schema()?;
//
//         let buffer_type = NP_Error::unwrap(NP_Type::parse_type_prc(&buffer_rpc::response, "myType<string>.set", &schema)?)?;
//         assert_eq!(buffer_type, NP_Type {
//             kind: NP_Types::rpc_response { idx: 10, func: 1, uid: 4204945332 },
//             generics: Some(vec![
//                 NP_Type {
//                     kind: NP_Types::string,
//                     generics: None
//                 }
//             ])
//         });
//
//         let (length, bytes) = &buffer_type.get_bytes()?;
//         // println!("{:?}", &bytes[0..(*length as usize)]);
//         let from_bytes_type = NP_Type::from_bytes(&bytes[0..(*length as usize)], &schema)?.1;
//         assert_eq!(buffer_type, from_bytes_type);
//         assert_eq!(from_bytes_type.generate_string(&schema), "myType<string>.set");
//
//         Ok(())
//     }
// }

