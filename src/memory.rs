//! Internal buffer memory management

use crate::{pointer::{NP_Cursor, NP_Cursor_Addr, NP_Cursor_Value}, schema::NP_Parsed_Schema};
use crate::{PROTOCOL_VERSION, error::NP_Error};
use core::cell::UnsafeCell;
use alloc::vec::Vec;

#[derive(Debug)]
#[doc(hidden)]
pub struct NP_Memory<'memory> {
    bytes: UnsafeCell<Vec<u8>>,
    cache: UnsafeCell<Vec<NP_Cursor>>,
    virtual_cursor: UnsafeCell<NP_Cursor>,
    pub schema: &'memory Vec<NP_Parsed_Schema>
}



#[doc(hidden)]
impl<'memory> NP_Memory<'memory> {


    pub fn existing(bytes: Vec<u8>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {
        
        NP_Memory {
            cache: UnsafeCell::new(Vec::with_capacity(bytes.len() / 2)),
            bytes: UnsafeCell::new(bytes),
            virtual_cursor: UnsafeCell::new(NP_Cursor::default()),
            schema: schema
        }
    }



    #[inline(always)]
    pub fn read_address(&self, addr: usize) -> usize {
        if addr == 0 {
            return 0;
        }
        match self.size {
            NP_Size::U8 =>  { u8::from_be_bytes([self.get_1_byte(addr).unwrap_or(0)]) as usize },
            NP_Size::U16 => { u16::from_be_bytes(*self.get_2_bytes(addr).unwrap_or(&[0; 2])) as usize },
            NP_Size::U32 => { u32::from_be_bytes(*self.get_4_bytes(addr).unwrap_or(&[0; 4])) as usize }
        }
    }

    #[inline(always)]
    pub fn read_address_offset(&self, addr: usize, offset: usize) -> usize {
        if addr == 0 {
            return 0;
        }
        match self.size {
            NP_Size::U8 =>  { u8::from_be_bytes([self.get_1_byte(addr + offset).unwrap_or(0)]) as usize },
            NP_Size::U16 => { u16::from_be_bytes(*self.get_2_bytes(addr + (offset * 2)).unwrap_or(&[0; 2])) as usize },
            NP_Size::U32 => { u32::from_be_bytes(*self.get_4_bytes(addr + (offset * 4)).unwrap_or(&[0; 4])) as usize }
        }
    }

    pub fn new(capacity: Option<usize>, schema: &'memory Vec<NP_Parsed_Schema>) -> Self {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);


        new_bytes.push(PROTOCOL_VERSION); // Protocol version (for breaking changes if needed later)

        match &size {
            NP_Size::U32 => {
                new_bytes.push(0); // size key (0 for U32)
                new_bytes.extend(0u32.to_be_bytes().to_vec()); // u32 HEAD for root pointer (starts at zero)
            },
            NP_Size::U16 => {
                new_bytes.push(1); // size key (1 for U16)
                new_bytes.extend(0u16.to_be_bytes().to_vec()); // u16 HEAD for root pointer (starts at zero)
            },
            NP_Size::U8 => {
                new_bytes.push(1); // size key (1 for U8)
                new_bytes.extend(0u8.to_be_bytes().to_vec()); // u16 HEAD for root pointer (starts at zero)
            }
        };

        let addr_size = match size {
            NP_Size::U8 => 1usize,
            NP_Size::U16 => 2,
            NP_Size::U32 => 4
        };

        NP_Memory {
            bytes: UnsafeCell::new(new_bytes),
            virtual_cursor: UnsafeCell::new(NP_Cursor::default()),
            cache: UnsafeCell::new(Vec::with_capacity(use_size / addr_size)),
            schema: schema,
            size: size
        }
    }

    pub fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {
        let self_bytes = unsafe { &mut *self.bytes.get() };

        let location = self_bytes.len();

        let max_sze = match self.size {
            NP_Size::U8 =>   MAX_SIZE_XSMALL,
            NP_Size::U16 =>   MAX_SIZE_SMALL,
            NP_Size::U32 =>   MAX_SIZE_LARGE
        };

        // not enough space left?
        if location + bytes.len() >= max_sze {
            return Err(NP_Error::new("Not enough space available in buffer!"))
        }

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
    pub fn get_cache(&self, index: &NP_Cursor_Addr) -> &mut NP_Cursor {
        match index {
            NP_Cursor_Addr::Virtual => { unsafe { &mut *self.virtual_cursor.get() } }
            NP_Cursor_Addr::Real(addr) => {
                let size = self.addr_size_bytes();
                let self_cache = unsafe { &mut *self.cache.get() };
                &mut self_cache[addr / size]
            }
        }
    }

    #[inline(always)]
    pub fn insert_cache(&self, index: usize, cursor: NP_Cursor) {
        let size = self.addr_size_bytes();
        let self_cache = unsafe { &mut *self.cache.get() };
        self_cache.insert(index / size, cursor);
    }

/*
    pub fn malloc_cursor(&self, value: &NP_Cursor_Value) -> Result<usize, NP_Error> {
        // Get the size of this pointer based it's kind
        match self.size {
            NP_Size::U32 => {
                match value {
                    NP_Cursor_Value::None              =>    { panic!() },
                    NP_Cursor_Value::Standard  { .. }  =>    { self.malloc_borrow(&[0; 4])  },
                    NP_Cursor_Value::TupleItem { .. }  =>    { self.malloc_borrow(&[0; 4])  },
                    NP_Cursor_Value::MapItem   { .. }  =>    { self.malloc_borrow(&[0; 12]) },
                    NP_Cursor_Value::TableItem { .. }  =>    { self.malloc_borrow(&[0; 4])  },
                    NP_Cursor_Value::ListItem  { .. }  =>    { self.malloc_borrow(&[0; 10]) }
                }
            },
            NP_Size::U16 => {
                match value {
                    NP_Cursor_Value::None              =>    { panic!() },
                    NP_Cursor_Value::Standard  { .. }  =>    { self.malloc_borrow(&[0; 2]) },
                    NP_Cursor_Value::TupleItem { .. }  =>    { self.malloc_borrow(&[0; 4]) },
                    NP_Cursor_Value::MapItem   { .. }  =>    { self.malloc_borrow(&[0; 6]) },
                    NP_Cursor_Value::TableItem { .. }  =>    { self.malloc_borrow(&[0; 2]) },
                    NP_Cursor_Value::ListItem  { .. }  =>    { self.malloc_borrow(&[0; 6]) }
                }
            },
            NP_Size::U8 => {
                match value {
                    NP_Cursor_Value::None              =>    { panic!() },
                    NP_Cursor_Value::Standard   { .. } =>    { self.malloc_borrow(&[0; 1]) },
                    NP_Cursor_Value::TupleItem  { .. } =>    { self.malloc_borrow(&[0; 1]) },
                    NP_Cursor_Value::MapItem    { .. } =>    { self.malloc_borrow(&[0; 3]) },
                    NP_Cursor_Value::TableItem  { .. } =>    { self.malloc_borrow(&[0; 1]) },
                    NP_Cursor_Value::ListItem   { .. } =>    { self.malloc_borrow(&[0; 3]) }
                }
            }
        }
    }
*/
    #[inline(always)]
    pub fn ptr_size(&self, cursor: &NP_Cursor) -> usize {
        // Get the size of this pointer based it's kind
        match self.size {
            NP_Size::U32 => {
                match cursor.value {
                    NP_Cursor_Value::None               =>    {  0 },
                    NP_Cursor_Value::Standard   { .. }  =>    {  4 },
                    NP_Cursor_Value::TupleItem  { .. }  =>    {  4 },
                    NP_Cursor_Value::MapItem    { .. }   =>   { 12 },
                    NP_Cursor_Value::TableItem  { .. }  =>    {  4 },
                    NP_Cursor_Value::ListItem   { .. }  =>    { 10 }
                }
            },
            NP_Size::U16 => {
                match cursor.value {
                    NP_Cursor_Value::None               =>    { 0 },
                    NP_Cursor_Value::Standard  { .. }   =>    { 2 },
                    NP_Cursor_Value::TupleItem { .. }   =>    { 4 },
                    NP_Cursor_Value::MapItem   { .. }   =>    { 6 },
                    NP_Cursor_Value::TableItem { .. }   =>    { 2 },
                    NP_Cursor_Value::ListItem  { .. }   =>    { 6 }
                }
            },
            NP_Size::U8 => {
                match cursor.value {
                    NP_Cursor_Value::None              =>    { 0 },
                    NP_Cursor_Value::Standard  { .. }  =>    { 1 },
                    NP_Cursor_Value::TupleItem { .. }  =>    { 1 },
                    NP_Cursor_Value::MapItem   { .. }  =>    { 3 },
                    NP_Cursor_Value::TableItem { .. }  =>    { 1 },
                    NP_Cursor_Value::ListItem  { .. }  =>    { 3 }
                }
            }
        }
    }

    #[inline(always)]
    pub fn write_address(&self, address: usize, val: usize) {

        let self_bytes = unsafe { &mut *self.bytes.get() };

        match self.size {
            NP_Size::U32 => {
                let bytes = (val as u32).to_be_bytes();
                for x in 0..bytes.len() {
                    self_bytes[address + x] = bytes[x];
                }
            },
            NP_Size::U16 => {
                let bytes = (val as u16).to_be_bytes();
                for x in 0..bytes.len() {
                    self_bytes[address + x] = bytes[x];
                }
            },
            NP_Size::U8 => {
                let bytes = (val as u8).to_be_bytes();
                for x in 0..bytes.len() {
                    self_bytes[address + x] = bytes[x];
                }
            }
        };

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