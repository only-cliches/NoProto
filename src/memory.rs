//! Internal buffer memory management

use crate::pointer::NP_Value;
use crate::{schema::NP_Parsed_Schema, pointer::{NP_Cursor, NP_Cursor_Addr, NP_Cursor_Kinds}};
use crate::{PROTOCOL_VERSION, error::NP_Error};
use core::cell::UnsafeCell;
use alloc::rc::Rc;
use alloc::vec::Vec;

/// The different address sizes availalbe for buffers
/// 
#[derive(Debug, Copy, Clone)]
pub enum NP_Size {
    /// 32 bit address, 4 bytes in size
    U32,
    /// 16 bit address, 2 bytes in size
    U16,
    /// 8 bit address, 1 byte in size
    U8
}


#[derive(Debug)]
#[doc(hidden)]
pub struct NP_Memory<'memory> {
    bytes: Rc<UnsafeCell<Vec<u8>>>,
    pub cursor_cache: Rc<UnsafeCell<Vec<NP_Cursor<'memory>>>>,
    pub virtual_cursor: Rc<UnsafeCell<NP_Cursor<'memory>>>,
    pub schema: &'memory Vec<NP_Parsed_Schema<'memory>>,
    pub size: NP_Size
}

const MAX_SIZE_LARGE: usize = core::u32::MAX as usize;
const MAX_SIZE_SMALL: usize = core::u16::MAX as usize;
const MAX_SIZE_XSMALL: usize = core::u8::MAX as usize;


pub fn blank_ptr_u32_standard()   -> [u8;  4] { [0;  4] }
pub fn blank_ptr_u32_tuple_item() -> [u8;  4] { [0;  4] }
pub fn blank_ptr_u32_map_item()   -> [u8; 12] { [0; 12] }
pub fn blank_ptr_u32_table_item() -> [u8;  9] { [0;  9] }
pub fn blank_ptr_u32_list_item()  -> [u8; 10] { [0; 10] }

pub fn blank_ptr_u16_standard()   -> [u8;  2] { [0;  2] }
pub fn blank_ptr_u16_tuple_item() -> [u8;  2] { [0;  2] }
pub fn blank_ptr_u16_map_item()   -> [u8;  6] { [0;  6] }
pub fn blank_ptr_u16_table_item() -> [u8;  5] { [0;  5] }
pub fn blank_ptr_u16_list_item()  -> [u8;  6] { [0;  6] }

pub fn blank_ptr_u8_standard()    -> [u8;  1] { [0;  1] }
pub fn blank_ptr_u8_tuple_item()  -> [u8;  1] { [0;  1] }
pub fn blank_ptr_u8_map_item()    -> [u8;  3] { [0;  3] }
pub fn blank_ptr_u8_table_item()  -> [u8;  3] { [0;  3] }
pub fn blank_ptr_u8_list_item()   -> [u8;  3] { [0;  3] }


#[doc(hidden)]
impl<'memory> NP_Memory<'memory> {

    pub fn clone(&self) -> Self {
        NP_Memory {
            bytes: Rc::clone(&self.bytes),
            cursor_cache: Rc::clone(&self.cursor_cache),
            virtual_cursor: Rc::clone(&self.virtual_cursor),
            schema: self.schema,
            size: self.size
        }
    }

    pub fn existing(bytes: Vec<u8>, schema: &'memory Vec<NP_Parsed_Schema<'memory>>) -> Self {

        let size = bytes[1];
        
        NP_Memory {
            bytes: Rc::new(UnsafeCell::new(bytes)),
            cursor_cache: Rc::new(UnsafeCell::new(Vec::new())),
            virtual_cursor: Rc::new(UnsafeCell::new(NP_Cursor::default())),
            schema: schema,
            size: match size {
                0 => NP_Size::U32,
                1 => NP_Size::U16,
                2 => NP_Size::U8,
                _ => NP_Size::U16
            }
        }
    }

    pub fn insert_cache(&self, cursor: NP_Cursor<'memory>) {
        let cache = unsafe { &mut *self.cursor_cache.get() };
        cache.insert(cursor.address, cursor);
    }

    pub fn max_addr_size(&self) -> usize {
        match &self.size {
            NP_Size::U32 => MAX_SIZE_LARGE,
            NP_Size::U16 => MAX_SIZE_SMALL,
            NP_Size::U8 => MAX_SIZE_XSMALL
        }
    }

    pub fn addr_size_bytes(&self) -> usize {
        match &self.size {
            NP_Size::U32 => 4,
            NP_Size::U16 => 2,
            NP_Size::U8 => 1
        }
    }


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

    pub fn read_address_offset(&self, addr: usize, u32_off: usize, u16_off: usize, u8_off: usize) -> usize {
        if addr == 0 {
            return 0;
        }
        match self.size {
            NP_Size::U8 =>  { u8::from_be_bytes([self.get_1_byte(addr + u8_off).unwrap_or(0)]) as usize },
            NP_Size::U16 => { u16::from_be_bytes(*self.get_2_bytes(addr + u16_off).unwrap_or(&[0; 2])) as usize },
            NP_Size::U32 => { u32::from_be_bytes(*self.get_4_bytes(addr + u32_off).unwrap_or(&[0; 4])) as usize }
        }
    }

    pub fn new(capacity: Option<usize>, size: NP_Size, schema: &'memory Vec<NP_Parsed_Schema<'memory>>) -> Self {
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


        NP_Memory {
            bytes: Rc::new(UnsafeCell::new(new_bytes)),
            cursor_cache: Rc::new(UnsafeCell::new(Vec::with_capacity(use_size))),
            virtual_cursor: Rc::new(UnsafeCell::new(NP_Cursor::default())),
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

    pub fn read_bytes(&self) -> &Vec<u8> {
        let self_bytes = unsafe { &*self.bytes.get() };
        self_bytes
    }

    pub fn write_bytes(&self) -> &mut Vec<u8> {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        self_bytes
    }

    pub fn ptr_size(&self, cursor: &NP_Cursor) -> usize {
        // Get the size of this pointer based it's kind
        match self.size {
            NP_Size::U32 => {
                match cursor.kind {
                    NP_Cursor_Kinds::None       =>    {  0 },
                    NP_Cursor_Kinds::Standard   =>    {  4 },
                    NP_Cursor_Kinds::Table      =>    {  4 },
                    NP_Cursor_Kinds::Tuple      =>    {  4 },
                    NP_Cursor_Kinds::Map        =>    {  4 },
                    NP_Cursor_Kinds::List       =>    {  4 },
                    NP_Cursor_Kinds::TupleItem  =>    {  4 },
                    NP_Cursor_Kinds::MapItem    =>    { 12 },
                    NP_Cursor_Kinds::TableItem  =>    {  9 },
                    NP_Cursor_Kinds::ListItem   =>    { 10 }
                }
            },
            NP_Size::U16 => {
                match cursor.kind {
                    NP_Cursor_Kinds::None        =>    { 0 },
                    NP_Cursor_Kinds::Standard    =>    { 2 },
                    NP_Cursor_Kinds::Table      =>     { 2 },
                    NP_Cursor_Kinds::Tuple      =>     { 2 },
                    NP_Cursor_Kinds::Map        =>     { 2 },
                    NP_Cursor_Kinds::List       =>     { 2 },
                    NP_Cursor_Kinds::TupleItem   =>    { 4 },
                    NP_Cursor_Kinds::MapItem     =>    { 6 },
                    NP_Cursor_Kinds::TableItem   =>    { 5 },
                    NP_Cursor_Kinds::ListItem    =>    { 6 }
                }
            },
            NP_Size::U8 => {
                match cursor.kind {
                    NP_Cursor_Kinds::None        =>    { 0 },
                    NP_Cursor_Kinds::Standard    =>    { 1 },
                    NP_Cursor_Kinds::Table      =>     { 1 },
                    NP_Cursor_Kinds::Tuple      =>     { 1 },
                    NP_Cursor_Kinds::Map        =>     { 1 },
                    NP_Cursor_Kinds::List       =>     { 1 },
                    NP_Cursor_Kinds::TupleItem   =>    { 1 },
                    NP_Cursor_Kinds::MapItem     =>    { 3 },
                    NP_Cursor_Kinds::TableItem   =>    { 3 },
                    NP_Cursor_Kinds::ListItem    =>    { 3 }
                }
            }
        }
    }

    pub fn blank_ptr_bytes(&self, cursor: &NP_Cursor) -> Vec<u8> {
        let size = self.ptr_size(cursor);
        let mut empty_bytes = Vec::with_capacity(size as usize);
        for _x in 0..size {
            empty_bytes.push(0);
        }
        empty_bytes
    }
/*
    pub fn write_address(&self, addr: usize, value: usize) -> Result<(), NP_Error> {
        
        let addr_bytes = match self.size {
            NP_Size::U32 => value.to_be_bytes().to_vec(),
            NP_Size::U16 => (value as u16).to_be_bytes().to_vec(),
            NP_Size::U8 => (value as u8).to_be_bytes().to_vec()
        };

        if addr + addr_bytes.len() > self.max_addr_size() {
            return Err(NP_Error::new("Attempting to write out of bounds!"));
        }

        let self_bytes = unsafe { &mut *self.bytes.get() };

        for x in 0..addr_bytes.len() {
            self_bytes[addr + x] = addr_bytes[x];
        }

        Ok(())
    }

*/
    pub fn set_value_address(&self, address: usize, val: usize) {

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