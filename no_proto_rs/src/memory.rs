//! Internal buffer memory management

use crate::{error::NP_Error};
use core::cell::UnsafeCell;
use alloc::vec::Vec;
use crate::schema::{NP_Schema};
use alloc::sync::Arc;

#[doc(hidden)]
#[derive(PartialEq, Debug)]
pub enum NP_Memory_Kind {
    Owned { vec: Vec<u8> },
    Ref { vec: *const [u8] },
    RefMut { vec: *mut [u8], len: usize }
}


#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Memory {
    bytes: UnsafeCell<NP_Memory_Kind>,
    pub root: usize,
    pub schema: Arc<NP_Schema>,
    pub max_size: usize,
    pub is_mutable: bool,
}

unsafe impl Send for NP_Memory {}

impl Clone for NP_Memory {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            max_size: self.max_size,
            bytes: UnsafeCell::new(NP_Memory_Kind::Owned { vec: self.read_bytes().to_vec() }),
            schema: self.schema.clone(),
            is_mutable: true
        }
    }
}

#[doc(hidden)]
impl NP_Memory {

    #[inline(always)]
    pub fn existing_owned(bytes: Vec<u8>, schema: Arc<NP_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(NP_Memory_Kind::Owned { vec: bytes }),
            schema: schema,
            is_mutable: true
        }
    }

    #[inline(always)]
    pub fn existing_ref(bytes: *const [u8], schema: Arc<NP_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: 0,
            bytes: UnsafeCell::new(NP_Memory_Kind::Ref { vec: bytes }),
            schema: schema,
            is_mutable: false
        }
    }

    #[inline(always)]
    pub fn existing_ref_mut(bytes: *mut [u8], len: usize, schema: Arc<NP_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: usize::min(u32::MAX as usize, len),
            bytes: UnsafeCell::new(NP_Memory_Kind::RefMut { vec: bytes, len: len }),
            schema: schema,
            is_mutable: true
        }
    }

    #[inline(always)]
    pub fn new(capacity: Option<usize>, schema: Arc<NP_Schema>, root: usize) -> Self {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // root pointer
        // new_bytes.extend(&[0u8; 4]);

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(NP_Memory_Kind::Owned { vec: new_bytes }),
            schema: schema,
            is_mutable: true
        }
    }

    #[inline(always)]
    pub fn new_ref_mut(bytes: *mut [u8], schema: Arc<NP_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(NP_Memory_Kind::RefMut { vec: bytes, len: 0 }),
            schema: schema,
            is_mutable: true
        }
    }

    pub fn new_empty(&self, capacity: Option<usize>) -> Result<Self, NP_Error> {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // root pointer
        // new_bytes.extend(&[0u8; 4]);

        Ok(Self {
            root: self.root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(NP_Memory_Kind::Owned { vec: new_bytes }),
            schema: self.schema.clone(),
            is_mutable: true
        })
    }

    pub fn is_ref_mut(&self) -> bool {
        let self_bytes = unsafe { &*self.bytes.get() };

        match self_bytes {
            NP_Memory_Kind::RefMut { .. } => true,
            _ => false
        }
    }

    pub fn set_length(&mut self, new_len: usize) -> Result<(), NP_Error> {

        let self_bytes = unsafe { &mut *self.bytes.get() };

        match self_bytes {
            NP_Memory_Kind::Owned { .. } => {
                // NO OP
                Err(NP_Error::Unreachable)
            },
            NP_Memory_Kind::Ref { .. } => {
                // NO OP
                Err(NP_Error::Unreachable)
            },
            NP_Memory_Kind::RefMut { len, .. } => {
                *len = new_len;

                Ok(())
            }
        }
        
    }

    pub fn set_max_length(&mut self, len: usize) {

        let self_bytes = unsafe { &*self.bytes.get() };
        match self_bytes {
            NP_Memory_Kind::Owned { .. } => {
                self.max_size = usize::min(u32::MAX as usize, len);
            },
            NP_Memory_Kind::Ref { .. } => {
                // NO OP
            },
            NP_Memory_Kind::RefMut { .. } => {
                self.max_size = usize::min(u32::MAX as usize, len);
            }
        }
        
    }

    #[inline(always)]
    pub fn length(&self) -> usize {
        let self_bytes = unsafe { &*self.bytes.get() };
        match self_bytes {
            NP_Memory_Kind::Owned { vec} => vec.len(),
            NP_Memory_Kind::Ref { .. } => 0,
            NP_Memory_Kind::RefMut { len, .. } => *len
        }
    }

    #[inline(always)]
    pub fn get_schema(&self) -> &NP_Schema {
        &*self.schema
    }

    // #[inline(always)]
    // pub fn get_schema(&self, idx: usize) -> &NP_Parsed_Schema {
    //     &(unsafe { *(*self.schema).schemas })[idx]
    // }

    #[inline(always)]
    pub fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {

        let location = self.length();

        // not enough space left?
        if location + bytes.len() >= self.max_size {
            return Err(NP_Error::MemoryOutOfSpace)
        }

        let self_bytes = unsafe { &mut *self.bytes.get() };

        match self_bytes {
            NP_Memory_Kind::Owned { vec } => {
                vec.extend_from_slice(bytes);
            },
            NP_Memory_Kind::Ref { .. } => {
                return Err(NP_Error::MemoryReadOnly)
            },
            NP_Memory_Kind::RefMut { vec, len } => {
                let v = unsafe { &mut **vec };
                *len += bytes.len();
                for (x, b) in bytes.iter().enumerate() {
                    v[location + x] = *b;
                }

            }
        }

        
        Ok(location)
    }

    #[inline(always)]
    pub fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error> {
        self.malloc_borrow(&bytes)
    }

    #[inline(always)]
    pub fn read_bytes(&self) -> &[u8] {
        let self_bytes = unsafe { &*self.bytes.get() };
        match self_bytes {
            NP_Memory_Kind::Owned { vec } => &vec[..],
            NP_Memory_Kind::Ref { vec } => unsafe { &**vec },
            NP_Memory_Kind::RefMut { vec, .. } => unsafe { &**vec },
        }
    }   

    #[inline(always)]
    pub fn write_bytes(&self) -> &mut [u8] {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        match self_bytes {
            NP_Memory_Kind::Owned { vec } => &mut vec[..],
            NP_Memory_Kind::Ref { vec } => unsafe {
                let const_ptr = *vec;
                let mut_ptr = const_ptr as *mut [u8];
                &mut *mut_ptr
            },
            NP_Memory_Kind::RefMut { vec, .. } => unsafe { &mut **vec },
        }
    }

    #[inline(always)]
    pub fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }
 
        Some(self.read_bytes()[address])
    }

    #[inline(always)]
    pub fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.read_bytes();

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

        let self_bytes = self.read_bytes();

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

        let self_bytes = self.read_bytes();

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

        let self_bytes = self.read_bytes();

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

        let self_bytes = self.read_bytes();

        if self_bytes.len() < address + 32 {
            return None;
        }

        let slice = &self_bytes[address..(address + 32)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 32]) })
    }

    pub fn dump(self) -> Vec<u8> {
        let bytes = self.bytes.into_inner();
        match bytes {
            NP_Memory_Kind::Owned { vec } => vec,
            NP_Memory_Kind::Ref { vec } => Vec::from(unsafe { &*vec }),
            NP_Memory_Kind::RefMut { vec, ..  } => Vec::from(unsafe { &*vec })
        }
    }
}