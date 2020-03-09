
use crate::error::NoProtoError;

pub struct NoProtoMemory {
    pub bytes: Vec<u8>
}

const MAX_SIZE: u64 = std::u32::MAX as u64;

impl NoProtoMemory {
    pub fn malloc(&mut self, bytes: Vec<u8>) -> std::result::Result<u32, NoProtoError> {
        let location: u32 = self.bytes.len() as u32;

        // not enough space left?
        if (location + bytes.len() as u32) as u64 >= MAX_SIZE {
            return Err(NoProtoError::new("Out of memory!"))
        }

        &self.bytes.extend(bytes);
        Ok(location)
    }

    pub fn dump(self) -> Vec<u8> {
        self.bytes
    }
}