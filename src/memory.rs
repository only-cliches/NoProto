use crate::pointer::NP_PtrKinds;
use crate::error::NP_Error;
use core::cell::UnsafeCell;
use alloc::vec::Vec;

pub struct NP_Memory {
    bytes: UnsafeCell<Vec<u8>>
}

const MAX_SIZE: u64 = core::u32::MAX as u64;

impl NP_Memory {

    pub fn new(bytes: Vec<u8>) -> Self {
        NP_Memory {
            bytes: UnsafeCell::new(bytes),
        }
    }

    pub fn malloc(&self, bytes: Vec<u8>) -> core::result::Result<u32, NP_Error> {

        let self_bytes = unsafe { &mut *self.bytes.get() };

        let location: u32 = self_bytes.len() as u32;

        // not enough space left?
        if (location + bytes.len() as u32) as u64 >= MAX_SIZE {
            return Err(NP_Error::new("Out of memory!"))
        }

        self_bytes.extend(bytes);
        Ok(location)
    }

    pub fn read_bytes(&self) -> &Vec<u8> {
        let self_bytes = unsafe { &*self.bytes.get() };
        self_bytes
    }

    pub fn write_bytes(&self) -> &mut Vec<u8> {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        self_bytes
    }

    pub fn set_value_address(&self, address: u32, val: u32, kind: &NP_PtrKinds) -> NP_PtrKinds {

        let addr_bytes = val.to_le_bytes();

        let self_bytes = unsafe { &mut *self.bytes.get() };
    
        for x in 0..addr_bytes.len() {
            self_bytes[(address + x as u32) as usize] = addr_bytes[x as usize];
        }

        match kind {
            NP_PtrKinds::None => {
                NP_PtrKinds::None
            }
            NP_PtrKinds::Standard { value: _ } => {
                NP_PtrKinds::Standard { value: val}
            },
            NP_PtrKinds::MapItem { value: _, key,  next  } => {
                NP_PtrKinds::MapItem { value: val, key: *key, next: *next }
            },
            NP_PtrKinds::TableItem { value: _, i, next  } => {
                NP_PtrKinds::TableItem { value: val, i: *i, next: *next }
            },
            NP_PtrKinds::ListItem { value: _, i, next  } => {
                NP_PtrKinds::ListItem { value: val, i: *i, next: *next }
            }
        }
    }

    pub fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };
 
        Some(self_bytes[address])
    }

    pub fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };

        if self_bytes.len() < address + 2 {
            return None;
        }

        let slice = &self_bytes[address..(address + 2)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 2]) })
    }

    pub fn get_4_bytes(&self, address: usize) -> Option<&[u8; 4]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };

        if self_bytes.len() < address + 4 {
            return None;
        }

        let slice = &self_bytes[address..(address + 4)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 4]) })
    }

    pub fn get_8_bytes(&self, address: usize) -> Option<&[u8; 8]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };

        if self_bytes.len() < address + 8 {
            return None;
        }

        let slice = &self_bytes[address..(address + 8)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 8]) })
    }

    pub fn get_16_bytes(&self, address: usize) -> Option<&[u8; 16]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };

        if self_bytes.len() < address + 16 {
            return None;
        }

        let slice = &self_bytes[address..(address + 16)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 16]) })
    }

    pub fn get_32_bytes(&self, address: usize) -> Option<&[u8; 32]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };

        if self_bytes.len() < address + 32 {
            return None;
        }

        let slice = &self_bytes[address..(address + 32)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 32]) })
    }

    pub fn dump(self) -> Vec<u8> {
        self.bytes.into_inner()
    }
}