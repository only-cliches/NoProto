pub mod ast_parser;
pub mod schema_args;
pub mod schema_parser;
mod tests;

use alloc::prelude::v1::{String, Vec};
use crate::error::NP_Error;
use crate::hashmap::NP_HashMap;
use crate::schema::ast_parser::{AST_STR, AST};
use crate::schema::schema_args::NP_Schema_Args;


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum NP_String_Casing {
    None,
    Uppercase,
    Lowercase
}

impl Default for NP_String_Casing {
    fn default() -> Self {
        Self::None
    }
}


#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
enum NP_Schema_Type {
    None       ,
    Any        { size: u32 },
    Info       ,
    String     { size: u32, default: AST_STR, casing: NP_String_Casing, max_len: Option<usize> },
    Char       { size: u32, default: char },
    Int8       { size: u32, default: i8, min: Option<i8>, max: Option<i8> },
    Int16      { size: u32, default: i16, min: Option<i16>, max: Option<i16> },
    Int32      { size: u32, default: i32, min: Option<i32>, max: Option<i32> },
    Int64      { size: u32, default: i64, min: Option<i64>, max: Option<i64> },
    Uint8      { size: u32, default: u8, min: Option<u8>, max: Option<u8> },
    Uint16     { size: u32, default: u16, min: Option<u16>, max: Option<u16> },
    Uint32     { size: u32, default: u32, min: Option<u32>, max: Option<u32>  },
    Uint64     { size: u32, default: u64, min: Option<u64>, max: Option<u64>  },
    f32        { size: u32, default: f32, min: Option<f32>, max: Option<f32>  },
    f64        { size: u32, default: f64, min: Option<f64>, max: Option<f64>  },
    Dec32      { size: u32, default: i32, exp: i16, min: Option<i32>, max: Option<i32>  },
    Dec64      { size: u32, default: i64, exp: i16, min: Option<i64>, max: Option<i64>  },
    Boolean    { size: u32, default: bool },
    Geo32      { size: u32, default: (i16, i16) },
    Geo64      { size: u32, default: (i32, i32) },
    Geo128     { size: u32, default: (i64, i64) },
    Uuid       { size: u32 },
    Ulid       { size: u32 },
    Date       { size: u32, default: u64 },
    Enum       { size: u32, children: NP_HashMap<Option<usize>>, default: usize },
    Struct     { size: u32, children: NP_HashMap<usize> },
    Map        { size: u32 },
    Vec        { size: u32, max_len: Option<u64> },
    Result     { size: u32 },
    Option     { size: u32 },
    Array      { size: u32, len: usize },
    Tuple      { size: u32, children: Vec<usize> },
    Impl       { children: NP_HashMap<usize> },
    Fn_Self    ,
    Method     { args: Vec<(Option<String>, usize)>, returns: usize },
    Generic    { size: u32, parent_scham_addr: usize, generic_idx: usize },
    Custom     { size: u32, type_idx: usize }
}

#[allow(dead_code)]
const POINTER_SIZE: u32 = 4u32;

#[allow(dead_code)]
impl NP_Schema_Type {
    pub fn type_info(&self) -> (usize, &str, u32) {
        match self {
            NP_Schema_Type::None             => ( 0, "none", 0),
            NP_Schema_Type::Any { size }       => ( 1, "any", *size),
            NP_Schema_Type::Info             => ( 2, "info", 0),
            NP_Schema_Type::String { size, .. }    => ( 3, "string", *size),
            NP_Schema_Type::Char { size,.. }      => ( 4, "char", *size),
            NP_Schema_Type::Int8 { size,.. }      => ( 5, "i8", *size),
            NP_Schema_Type::Int16 { size, .. }     => ( 6, "i16", *size),
            NP_Schema_Type::Int32 { size, .. }     => ( 7, "i32", *size),
            NP_Schema_Type::Int64 { size, .. }     => ( 8, "i64", *size),
            NP_Schema_Type::Uint8 { size, .. }     => ( 9, "u8", *size),
            NP_Schema_Type::Uint16 { size, .. }    => (10, "u16", *size),
            NP_Schema_Type::Uint32 { size, .. }    => (11, "u32", *size),
            NP_Schema_Type::Uint64 { size, .. }    => (12, "u64", *size),
            NP_Schema_Type::f32 { size, .. }       => (13, "f32", *size),
            NP_Schema_Type::f64 { size, .. }       => (14, "f64", *size),
            NP_Schema_Type::Dec32 { size, .. }     => (15, "dec32", *size),
            NP_Schema_Type::Dec64 { size, .. }     => (16, "dec64", *size),
            NP_Schema_Type::Boolean { size, .. }   => (17, "bool", *size),
            NP_Schema_Type::Geo32 { size, .. }     => (18, "geo32", *size),
            NP_Schema_Type::Geo64 { size, .. }     => (19, "geo64", *size),
            NP_Schema_Type::Geo128 { size, .. }    => (20, "geo128", *size),
            NP_Schema_Type::Uuid { size, .. }      => (21, "uuid", *size),
            NP_Schema_Type::Ulid { size, .. }      => (22, "ulid", *size),
            NP_Schema_Type::Date { size, .. }      => (23, "date", *size),
            NP_Schema_Type::Enum { size, .. }      => (24, "enum", *size),
            NP_Schema_Type::Struct { size, .. }    => (25, "struct", *size),
            NP_Schema_Type::Map { size, .. }       => (26, "Map", *size),
            NP_Schema_Type::Vec { size, .. }       => (27, "Vec", *size),
            NP_Schema_Type::Result { size, .. }    => (28, "Result", *size),
            NP_Schema_Type::Option  { size, .. }   => (29, "Option", *size),
            NP_Schema_Type::Array { size, .. }     => (30, "Array", *size),
            NP_Schema_Type::Tuple { size, .. }     => (31, "tuple", *size),
            NP_Schema_Type::Impl { .. }      => (32, "impl", 0),
            NP_Schema_Type::Fn_Self          => (33, "self", 0),
            NP_Schema_Type::Method {  .. }    => (34, "method", 0),
            NP_Schema_Type::Generic { size, .. }   => (35, "generic", *size),
            NP_Schema_Type::Custom { size, .. }    => (36, "custom", *size),
        }
    }

}

impl Default for NP_Schema_Type {
    fn default() -> Self {
        NP_Schema_Type::None
    }
}


#[derive(Default, Debug, Clone)]
pub struct NP_Schema {
    source: Vec<u8>,
    schemas: Vec<NP_Parsed_Schema>,
    name_index: NP_HashMap<NP_Schema_Index>,
    id_index: Vec<NP_Schema_Index>
}

#[derive(Debug, Clone)]
struct NP_Parsed_Schema {
    id: Option<u16>,
    offset: usize,
    name: Option<AST_STR>,
    data_type: NP_Schema_Type,
    use_generics: Option<Vec<usize>>,
    self_generics: Option<(usize, Vec<AST_STR>)>,
    arguments: NP_Schema_Args
}

impl Default for NP_Parsed_Schema {
    fn default() -> Self {
        Self {
            id: None,
            offset: 0,
            name: None,
            data_type: NP_Schema_Type::None,
            use_generics: None,
            self_generics: None,
            arguments: NP_Schema_Args::NULL
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct NP_Schema_Index {
    data: usize,
    methods: Option<usize>
}

impl Default for NP_Schema_Index {
    fn default() -> Self {
        Self { data: 0, methods: None }
    }
}




