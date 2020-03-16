
use crate::pointer::NoProtoPointerKinds;
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

    pub fn set_value_address(&mut self, address: u32, val: u32, kind: &NoProtoPointerKinds) -> std::result::Result<NoProtoPointerKinds, NoProtoError> {

        let addr_bytes = val.to_le_bytes();
    
        for x in 0..addr_bytes.len() {
            self.bytes[(address + x as u32) as usize] = addr_bytes[x as usize];
        }

        Ok(match kind {
            NoProtoPointerKinds::None => {
                NoProtoPointerKinds::None
            }
            NoProtoPointerKinds::Standard { value: _ } => {
                NoProtoPointerKinds::Standard { value: val}
            },
            NoProtoPointerKinds::MapItem { value: _, key,  next  } => {
                NoProtoPointerKinds::MapItem { value: val, key: *key, next: *next }
            },
            NoProtoPointerKinds::TableItem { value: _, i, next  } => {
                NoProtoPointerKinds::TableItem { value: val, i: *i, next: *next }
            },
            NoProtoPointerKinds::ListItem { value: _, i, next  } => {
                NoProtoPointerKinds::ListItem { value: val, i: *i, next: *next }
            }
        })
    }

    pub fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }
 
        Some(self.bytes[address])
    }

    pub fn get_2_bytes(&self, address: usize) -> Option<[u8; 2]> {

        // empty value
        if address == 0 {
            return None;
        }

        let mut bytes: [u8; 2] = [0; 2];

        bytes.copy_from_slice(&self.bytes[address..(address + 2)]);

        Some(bytes)
    }

    pub fn get_4_bytes(&self, address: usize) -> Option<[u8; 4]> {

        // empty value
        if address == 0 {
            return None;
        }

        let mut bytes: [u8; 4] = [0;  4];

        bytes.copy_from_slice(&self.bytes[address..(address + 4)]);

        Some(bytes)
    }

    pub fn get_8_bytes(&self, address: usize) -> Option<[u8; 8]> {

        // empty value
        if address == 0 {
            return None;
        }

        let mut bytes: [u8; 8] = [0;  8];

        bytes.copy_from_slice(&self.bytes[address..(address + 8)]);

        Some(bytes)
    }

    pub fn get_16_bytes(&self, address: usize) -> Option<[u8; 16]> {

        // empty value
        if address == 0 {
            return None;
        }

        let mut bytes: [u8; 16] = [0;  16];

        bytes.copy_from_slice(&self.bytes[address..(address + 16)]);

        Some(bytes)
    }

    pub fn get_32_bytes(&self, address: usize) -> Option<[u8; 32]> {

        // empty value
        if address == 0 {
            return None;
        }

        let mut bytes: [u8; 32] = [0;  32];

        bytes.copy_from_slice(&self.bytes[address..(address + 32)]);

        Some(bytes)
    }

    pub fn dump(self) -> Vec<u8> {
        self.bytes
    }
}