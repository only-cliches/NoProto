//! Internal buffer memory management

use crate::{pointer::{NP_Cursor, NP_Cursor_Addr}, schema::NP_Parsed_Schema, utils::opt_out};
use crate::{error::NP_Error};
use core::cell::UnsafeCell;
use alloc::vec::Vec;


#[doc(hidden)]
pub struct NP_Memory<'memory> {
    bytes: UnsafeCell<Vec<u8>>,
    parsed: UnsafeCell<Vec<NP_Cursor<'memory>>>,
    virtual_cursor: UnsafeCell<NP_Cursor<'memory>>,
    pub schema: &'memory Vec<NP_Parsed_Schema>
}



#[doc(hidden)]
impl<'memory> NP_Memory<'memory> {


    pub fn existing(bytes: Vec<u8>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {
        
        let mut parsed_vec: Vec<NP_Cursor> = Vec::with_capacity(bytes.len() / 2);
        parsed_vec.extend((0..(bytes.len() / 2)).map(|_| NP_Cursor::new_virtual()));

        NP_Memory {
            parsed: UnsafeCell::new(parsed_vec),
            bytes: UnsafeCell::new(bytes),
            virtual_cursor: UnsafeCell::new(NP_Cursor::new_virtual()),
            schema: schema
        }
    }


    pub fn new(capacity: Option<usize>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // root pointer
        new_bytes.extend(&[0u8; 2]);

        let mut parsed_vec: Vec<NP_Cursor> = Vec::with_capacity(use_size / 2);
        parsed_vec.push(NP_Cursor::new_virtual());

        NP_Memory {
            bytes: UnsafeCell::new(new_bytes),
            virtual_cursor: UnsafeCell::new(NP_Cursor::new_virtual()),
            parsed: UnsafeCell::new(parsed_vec),
            schema: schema,
        }
    }

    pub fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        let self_parsed = unsafe { &mut *self.parsed.get() };

        let location = self_bytes.len();

        // not enough space left?
        if location + bytes.len() >= core::u16::MAX as usize {
            return Err(NP_Error::new("Not enough space available in buffer!"))
        }

        self_parsed.extend((0..((bytes.len() / 2) + 1)).map(|_| NP_Cursor::new_virtual()));
        self_bytes.extend(bytes);
        Ok(location)
    }

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
    pub fn get_parsed(&self, index: &NP_Cursor_Addr) -> &mut NP_Cursor<'memory> {
        match index {
            NP_Cursor_Addr::Virtual => { unsafe { &mut *self.virtual_cursor.get() } }
            NP_Cursor_Addr::Real(addr) => {
                let self_cache = unsafe { &mut *self.parsed.get() };
                &mut self_cache[addr / 2]
            }
        }
    }

    pub fn get_schema(&self, index: &NP_Cursor_Addr) -> &'memory NP_Parsed_Schema {
        match index {
            NP_Cursor_Addr::Virtual => { 
                let cursor = unsafe { &mut *self.virtual_cursor.get() };
                &self.schema[cursor.schema_addr]
            },
            NP_Cursor_Addr::Real(addr) => {
                let self_cache = unsafe { &mut *self.parsed.get() };
                let cursor = &self_cache[addr / 2];
                &self.schema[cursor.schema_addr]
            }
        }
    }

    #[inline(always)]
    pub fn insert_parsed(&self, index: usize, cursor: NP_Cursor<'memory>) {
        let self_cache = unsafe { &mut *self.parsed.get() };
        let insert_index = index / 2;
        self_cache[insert_index] = cursor;
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