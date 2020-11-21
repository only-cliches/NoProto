//! Internal buffer memory management

use crate::pointer::NP_PtrKinds;
use crate::{PROTOCOL_VERSION, error::NP_Error};
use core::cell::UnsafeCell;
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
pub struct NP_Memory {
    bytes: UnsafeCell<Vec<u8>>,
    iterators: UnsafeCell<Vec<Option<u64>>>,
    pub size: NP_Size
}

const MAX_SIZE_LARGE: usize = core::u32::MAX as usize;
const MAX_SIZE_SMALL: usize = core::u16::MAX as usize;
const MAX_SIZE_XSMALL: usize = core::u8::MAX as usize;

#[doc(hidden)]
impl<'a> NP_Memory {

    pub fn existing(bytes: Vec<u8>) -> Self {

        let size = bytes[1];
        
        NP_Memory {
            bytes: UnsafeCell::new(bytes),
            iterators: UnsafeCell::new(Vec::new()),
            size: match size {
                0 => NP_Size::U32,
                1 => NP_Size::U16,
                2 => NP_Size::U8,
                _ => NP_Size::U16
            }
        }
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

    pub fn new(capacity: Option<usize>, size: NP_Size) -> Self {
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
        }


        NP_Memory {
            bytes: UnsafeCell::new(new_bytes),
            iterators: UnsafeCell::new(Vec::new()),
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

    pub fn read_bytes(&self) -> &Vec<u8> {
        let self_bytes = unsafe { &*self.bytes.get() };
        self_bytes
    }

    pub fn new_it(&self, value: u64) -> usize {
        let self_it = unsafe { &mut *self.iterators.get() };
        let mut x: usize = 0;
        for it in self_it.iter_mut() {
            if let None = it {
                *it = Some(value);
                return x;
            }
            x += 1;
        }
        let len = self_it.len();
        self_it.push(Some(value));
        len
    }

    pub fn del_it(&self, index: usize) {
        let self_it = unsafe { &mut *self.iterators.get() };
        if self_it.len() <= index {
            self_it[index] = None;
        }
    }

    pub fn it_write(&self, index: usize, value: u64) {
        let self_it = unsafe { &mut *self.iterators.get() };
        self_it[index] = Some(value);
    }

    pub fn it_read(&self, index: usize) -> Option<u64> {
        let self_it = unsafe { &*self.iterators.get() };
        self_it[index]
    }

    pub fn write_bytes(&self) -> &mut Vec<u8> {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        self_bytes
    }

    pub fn ptr_size(&self, ptr: &NP_PtrKinds) -> usize {
        // Get the size of this pointer based it's kind
        match self.size {
            NP_Size::U32 => {
                match ptr {
                    NP_PtrKinds::None                                    =>    {  0 },
                    NP_PtrKinds::Standard  { addr: _ }                   =>    {  4 },
                    NP_PtrKinds::TupleItem { addr: _, i:_  }             =>    {  4 },
                    NP_PtrKinds::MapItem   { addr: _, key: _,  next: _ } =>    { 12 },
                    NP_PtrKinds::TableItem { addr: _, i:_ ,    next: _ } =>    {  9 },
                    NP_PtrKinds::ListItem  { addr: _, i:_ ,    next: _ } =>    { 10 }
                }
            },
            NP_Size::U16 => {
                match ptr {
                    NP_PtrKinds::None                                    =>    { 0 },
                    NP_PtrKinds::Standard  { addr: _ }                   =>    { 2 },
                    NP_PtrKinds::TupleItem { addr: _, i:_  }             =>    { 4 },
                    NP_PtrKinds::MapItem   { addr: _, key: _,  next: _ } =>    { 6 },
                    NP_PtrKinds::TableItem { addr: _, i:_ ,    next: _ } =>    { 5 },
                    NP_PtrKinds::ListItem  { addr: _, i:_ ,    next: _ } =>    { 6 }
                }
            },
            NP_Size::U8 => {
                match ptr {
                    NP_PtrKinds::None                                    =>    { 0 },
                    NP_PtrKinds::Standard  { addr: _ }                   =>    { 1 },
                    NP_PtrKinds::TupleItem { addr: _, i:_  }             =>    { 1 },
                    NP_PtrKinds::MapItem   { addr: _, key: _,  next: _ } =>    { 3 },
                    NP_PtrKinds::TableItem { addr: _, i:_ ,    next: _ } =>    { 3 },
                    NP_PtrKinds::ListItem  { addr: _, i:_ ,    next: _ } =>    { 3 }
                }
            }
        }
    }

    pub fn blank_ptr_bytes(&self, ptr: &NP_PtrKinds) -> Vec<u8> {
        let size = self.ptr_size(ptr);
        let mut empty_bytes = Vec::with_capacity(size as usize);
        for _x in 0..size {
            empty_bytes.push(0);
        }
        empty_bytes
    }

    pub fn set_value_address(&self, address: usize, val: usize, kind: &NP_PtrKinds) -> NP_PtrKinds {

        let addr_bytes = match self.size {
            NP_Size::U32 => val.to_be_bytes().to_vec(),
            NP_Size::U16 => (val as u16).to_be_bytes().to_vec(),
            NP_Size::U8 => (val as u8).to_be_bytes().to_vec()
        };

        let self_bytes = unsafe { &mut *self.bytes.get() };
    
        for x in 0..addr_bytes.len() {
            self_bytes[address + x] = addr_bytes[x as usize];
        }

        match kind {
            NP_PtrKinds::None => {
                NP_PtrKinds::None
            }
            NP_PtrKinds::Standard { addr: _ } => {
                NP_PtrKinds::Standard { addr: val }
            },
            NP_PtrKinds::TupleItem { addr: _, i} => {
                NP_PtrKinds::TupleItem { addr: val, i: *i }
            },
            NP_PtrKinds::MapItem { addr: _, key,  next  } => {
                NP_PtrKinds::MapItem { addr: val, key: *key, next: *next }
            },
            NP_PtrKinds::TableItem { addr: _, i, next  } => {
                NP_PtrKinds::TableItem { addr: val, i: *i, next: *next }
            },
            NP_PtrKinds::ListItem { addr: _, i, next  } => {
                NP_PtrKinds::ListItem { addr: val, i: *i, next: *next }
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