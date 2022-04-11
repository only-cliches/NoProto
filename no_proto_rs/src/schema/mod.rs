pub mod ast;
pub mod args;
pub mod parser;
// mod tests;


use core::ops::DerefMut;
use core::ops::Deref;
use crate::error::NP_Error;
use crate::map::NP_OrderedMap;
use alloc::vec::Vec;
use crate::types::NP_Type;
use crate::schema::args::NP_Schema_Args;
use core::str;

#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct AST_STR { 
    pub start: usize,
    pub end: usize
}

#[allow(dead_code)]
impl AST_STR {
    pub fn read<'read>(&self, source: &'read str) -> &'read str {
        &source[self.start..self.end]
    }

    pub fn read_bytes<'read>(&self, source: &'read [u8]) -> &'read str {
        unsafe { str::from_utf8_unchecked(&source[self.start..self.end])}
    }

    pub fn from_bytes(pos: usize, buffer: &[u8]) -> Result<(usize, Self), NP_Error> {

        if pos + 3 > buffer.len() {
            return Err(NP_Error::OutOfBounds)
        }

        let mut new = AST_STR { start: 0, end: 0 };

        let ptr = &buffer[pos];
        new.start = le_bytes_read!(u16, ptr) as usize;
        let length = buffer[pos + 3] as usize;
        new.end = new.start + length;

        Ok((pos + 3, new))
    }

    pub fn to_bytes(&self) -> [u8; 3] {
        let mut result = [0u8; 3];

        let ptr = &mut result[0] as *mut u8;
        let val = &self.start;
        le_bytes_write!(u16, ptr, val);

        result[2] = (self.end - self.start) as u8;

        // let ptr = &mut result[2] as *mut u8;
        // let val = &self.end;
        // le_bytes_write!(u16, ptr, val);

        result
    }
}


#[derive(Default, Debug, Clone, PartialEq)]
pub struct NP_Schem_Kind {
    pub val: NP_Type<usize, AST_STR>
}

impl NP_Schem_Kind {
    pub fn new(val: NP_Type<usize, AST_STR>) -> Self {
        Self { val }
    }
}

impl Deref for NP_Schem_Kind {
    type Target = NP_Type<usize, AST_STR>;
    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl DerefMut for NP_Schem_Kind {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct NP_Schema_Value {
    id: Option<usize>,
    kind: NP_Schem_Kind,
    name: Option<AST_STR>,
    generics: NP_Parsed_Generics,
    args: NP_Schema_Args
}

#[derive(Debug, Clone, PartialEq)]
enum NP_Parsed_Generics {
    None,
    Parent (usize, Vec<AST_STR>), // this index, arguments
    Child (usize, usize) // parent index, argument position
}

impl Default for NP_Parsed_Generics {
    fn default() -> Self {
        Self::None
    }
}

#[allow(dead_code)]
const POINTER_SIZE: u32 = 4u32;

#[derive(Default, Debug, Clone)]
pub struct NP_Schema {
    pub source: Vec<u8>,
    pub schemas: Vec<NP_Schema_Value>,
    pub name_index: NP_OrderedMap<NP_Schema_Index>,
    pub id_index: Vec<NP_Schema_Index>,
    pub unique_id: u32
}



#[derive(Default, Debug, Clone, PartialEq)]
pub struct NP_Schema_Index {
    pub data: usize,
    pub methods: Option<usize>
}