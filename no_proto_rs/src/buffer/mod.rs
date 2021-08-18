use crate::memory::NP_Memory;
use crate::buffer::type_parser::{NP_Buffer_Type, NP_Types, buffer_rpc};
use crate::error::NP_Error;
use alloc::prelude::v1::String;

pub mod type_parser;

#[derive(Debug, Clone)]
pub struct NP_Buffer {
    memory: NP_Memory,
    root: NP_Buffer_Type,
    pub mutable: bool
}

impl NP_Buffer {

    //! Allows you to print the buffer type, useful if you need to verify or check the data type in this buffer
    //!
    pub fn print_buffer_type(&self) -> String {
        return self.root.generate_string(&self.memory.schema)
    }

    #[doc(hidden)]
    pub fn _new(data_type: &str, mut memory: NP_Memory) -> Result<Self, NP_Error> { // make new buffer

        // parse type
        let root = NP_Error::unwrap(NP_Buffer_Type::parse_type(&buffer_rpc::none, data_type, &memory.schema)?)?;

        // write type into buffer
        let (type_len, type_bytes) = root.get_bytes()?;
        memory.malloc_borrow(&[type_len])?;
        memory.malloc_borrow(&type_bytes[0..(type_len as usize)])?;

        // root pointer
        memory.malloc_borrow(&[0u8; 4])?;

        memory.root = (type_len + 1) as usize;

        Ok(Self {
            mutable: memory.is_mutable,
            root: root,
            memory: memory
        })
    }

    #[doc(hidden)]
    pub fn _existing(mut memory: NP_Memory) -> Result<Self, NP_Error> { // make new buffer

        // get type length
        let type_len = NP_Error::unwrap(memory.get_1_byte(0))? as usize;

        // should have at least space for schema and root pointer
        if type_len + 5 >= memory.length() {
            return Err(NP_Error::OutOfBounds)
        }

        // parse type from buffer
        let root = NP_Buffer_Type::from_bytes(&memory.read_bytes()[1..(type_len + 1)], &memory.schema)?.1;

        memory.root = (type_len + 1) as usize;

        Ok(Self{
            mutable: memory.is_mutable,
            root: root,
            memory: memory
        })
    }
}