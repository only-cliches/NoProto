use alloc::string::String;
use crate::types::NP_Type;


#[derive(Debug, Clone)]
struct NP_Cursor {
    pub buffer_addr: usize,
    pub schema_addr: usize
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct NP_Buffer_Type {
    kind: NP_Type<NP_Buffer_Type, String>,
    size: NP_Type_Size
}

#[derive(Debug, Clone, PartialEq)]
pub enum NP_Type_Size {
    pointer,
    pointer_and (u32),
    fixed       (u32)
}

impl Default for NP_Type_Size {
    fn default() -> Self { NP_Type_Size::pointer }
}

// #[derive(Debug, Clone)]
// pub struct NP_Buffer {
//     memory: NP_Memory,
//     root: NP_Types_Outer,
//     cursor: NP_Cursor,
//     pub mutable: bool
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum buffer_rpc {
//     request,
//     response,
//     none
// }

// impl NP_Buffer {

//     #[doc(hidden)]
//     pub fn _generate_response_buffer(&self, mut memory: NP_Memory) -> Result<Self, NP_Error> {

//         let root = self.root.get_response_type_for_request()?;

//         // write type into buffer
//         let (type_len, type_bytes) = root.get_bytes()?;
//         memory.malloc_borrow(&[type_len])?;
//         memory.malloc_borrow(&type_bytes[0..(type_len as usize)])?;

//         // root data
//         let root_size = root.kind.get_size(&memory.schema) as usize;
//         memory.malloc_borrow(&vec![0u8; root_size])?;

//         memory.root = (type_len + 1) as usize;

//         Ok(Self {
//             mutable: memory.is_mutable,
//             root: root,
//             cursor: NP_Cursor { buffer_addr: memory.root, schema_addr: 0 },
//             memory: memory
//         })
//     }

//     #[doc(hidden)]
//     pub fn _new(rpc: buffer_rpc, data_type: &str, mut memory: NP_Memory) -> Result<Self, NP_Error> { // make new buffer

//         // parse type
//         let root = NP_Error::unwrap(match rpc {
//             buffer_rpc::none => NP_Types_Outer::parse_type(data_type, &memory.schema)?,
//             buffer_rpc::request => NP_Types_Outer::parse_type_prc(&rpc, data_type, &memory.schema)?,
//             buffer_rpc::response => NP_Types_Outer::parse_type_prc(&rpc, data_type, &memory.schema)?
//         })?;

//         // write type into buffer
//         let (type_len, type_bytes) = root.get_bytes()?;
//         memory.malloc_borrow(&[type_len])?;
//         memory.malloc_borrow(&type_bytes[0..(type_len as usize)])?;

//         // root data
//         let root_size = root.kind.get_size(&memory.schema) as usize;
//         memory.malloc_borrow(&vec![0u8; root_size])?;

//         memory.root = (type_len + 1) as usize;

//         Ok(Self {
//             mutable: memory.is_mutable,
//             root: root,
//             cursor: NP_Cursor { buffer_addr: memory.root, schema_addr: 0 },
//             memory: memory
//         })
//     }

//     #[doc(hidden)]
//     pub fn _existing(mut memory: NP_Memory) -> Result<Self, NP_Error> { // make new buffer

//         // get type length
//         let type_len = NP_Error::unwrap(memory.get_1_byte(0))? as usize;

//         // should have at least space for schema
//         if type_len >= memory.length() {
//             return Err(NP_Error::OutOfBounds)
//         }

//         // parse type from buffer
//         let root = NP_Types_Outer::from_bytes(&memory.read_bytes()[1..(type_len + 1)], &memory.schema)?.1;

//         memory.root = (type_len + 1) as usize;

//         Ok(Self{
//             mutable: memory.is_mutable,
//             root: root,
//             cursor: NP_Cursor { buffer_addr: memory.root, schema_addr: 0 },
//             memory: memory
//         })
//     }


//     pub fn print_buffer_type(&self) -> String {
//         return self.root.generate_string(&self.memory.schema)
//     }

//     fn query_path(&self, make_path: bool, path: &str) -> Option<usize> {
//         todo!()
//     }

//     pub fn reset_cursor(&mut self) {
//         self.cursor = NP_Cursor { buffer_addr: self.memory.root, schema_addr: 0 };
//     }

//     pub fn move_cursor(&mut self, path: &str) -> Option<()> {
//         todo!()
//     }

//     pub fn data_type(&self, path: &str) -> Option<NP_Schema_Data> {
//         todo!()
//     }

//     pub fn get<X: NP_Value>(&self, path: &str) -> Option<X> {
//         todo!()
//     }

//     pub fn get_bytes(&self, path: &str) -> Option<&[u8]> {
//         todo!()
//     }

//     pub fn set<X: NP_Value>(&mut self, path: &str, value: X) -> Result<(), NP_Error> {
//         todo!()
//     }

//     pub fn clear(&mut self, path: &str) -> Option<()> {
//         todo!()
//     }

//     pub fn calc_size(&self) -> Result<NP_Size_Data, NP_Error> {
//         todo!()
//     }

//     pub fn compact_self(&mut self) -> Result<(), NP_Error> {
//         todo!()
//     }

//     pub fn compact_into(&self) -> Result<Self, NP_Error> {
//         todo!()
//     }

// }