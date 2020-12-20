//! Internal buffer memory management

use crate::{schema::NP_Parsed_Schema};
use crate::{error::NP_Error};
use core::cell::UnsafeCell;
use alloc::vec::Vec;


#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Memory<'memory> {
    bytes: UnsafeCell<Vec<u8>>,
    pub schema: &'memory Vec<NP_Parsed_Schema>
}



#[doc(hidden)]
impl<'memory> NP_Memory<'memory> {

    #[inline(always)]
    pub fn get_schema(&self) -> &'memory Vec<NP_Parsed_Schema> {
        self.schema
    }

    #[inline(always)]
    pub fn existing(bytes: Vec<u8>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {

        Self {
            bytes: UnsafeCell::new(bytes),
            schema: schema
        }
    }

    #[inline(always)]
    pub fn new(capacity: Option<usize>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // size, root pointer
        new_bytes.extend(&[0u8; 3]);

        NP_Memory {
            bytes: UnsafeCell::new(new_bytes),
            schema: schema,
        }
    }

    #[inline(always)]
    pub fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {
        let self_bytes = unsafe { &mut *self.bytes.get() };

        let location = self_bytes.len();

        // not enough space left?
        if location + bytes.len() >= core::u16::MAX as usize {
            return Err(NP_Error::new("Not enough space available in buffer!"))
        }

        self_bytes.extend(bytes);
        Ok(location)
    }

    #[inline(always)]
    pub fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error> {
        self.malloc_borrow(&bytes)
    }

    #[inline(always)]
    pub fn read_bytes(&self) -> &Vec<u8> {
        let self_bytes = unsafe { &*self.bytes.get() };
        self_bytes
    }   

    #[inline(always)]
    pub fn write_bytes(&self) -> &mut Vec<u8> {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        self_bytes
    }

    #[inline(always)]
    pub fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };
 
        Some(self_bytes[address])
    }

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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

    #[inline(always)]
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