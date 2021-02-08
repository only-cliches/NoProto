//! Internal buffer memory management

use crate::{schema::NP_Parsed_Schema};
use crate::{error::NP_Error};
use core::cell::UnsafeCell;
use alloc::vec::Vec;

#[doc(hidden)]
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum NP_Memory_Kind {
    Owned,
    Ref,
    RefMut { len: usize }
}

#[doc(hidden)]
pub trait NP_Memory {
    fn kind(&self) -> NP_Memory_Kind;
    fn length(&self) -> usize;
    fn set_length(&mut self, _len: usize) -> Result<(), NP_Error> {
        // only called on RefMut memory
        Err(NP_Error::Unreachable)
    }
    fn set_max_length(&mut self, len: usize);
    fn is_mutable(&self) -> bool;
    fn get_root(&self) -> usize;
    fn get_schemas(&self) -> &Vec<NP_Parsed_Schema>;
    fn get_schema(&self, idx: usize) -> &NP_Parsed_Schema;
    fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error>;
    fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error>;
    fn read_bytes(&self) -> &[u8];
    fn write_bytes(&self) -> &mut [u8];
    fn get_1_byte(&self, address: usize) -> Option<u8>;
    fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]>;
    fn get_4_bytes(&self, address: usize) -> Option<&[u8; 4]>;
    fn get_8_bytes(&self, address: usize) -> Option<&[u8; 8]>;
    fn get_16_bytes(&self, address: usize) -> Option<&[u8; 16]>;
    fn get_32_bytes(&self, address: usize) -> Option<&[u8; 32]>;
    fn dump(self) -> Vec<u8>;
}

/// Creat a new empty version of this buffer value
pub trait NP_Mem_New {
    /// create empty
    fn new_empty(&self, capacity: Option<usize>) -> Result<Self, NP_Error> where Self: core::marker::Sized;
}

#[doc(hidden)]
#[derive(Debug, Clone)]
pub enum SchemaVec<'vec> {
    Owned(Vec<NP_Parsed_Schema>),
    Borrowed(&'vec Vec<NP_Parsed_Schema>)
}

impl<'vec> SchemaVec<'vec> {
    /// Borrow the underlying schema vec
    #[inline(always)]
    pub fn get(&self) -> &Vec<NP_Parsed_Schema> {
        match &self {
            SchemaVec::Owned(x) => x,
            SchemaVec::Borrowed(x) => *x
        }
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum WritableBytes<'writable> {
    owned { bytes: Vec<u8>},
    borrowed { bytes: &'writable mut [u8], len: u32 }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Memory_Owned {
    bytes: UnsafeCell<Vec<u8>>,
    pub root: usize,
    pub schema: *const Vec<NP_Parsed_Schema>,
    pub max_size: usize
}

impl Clone for NP_Memory_Owned {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            max_size: self.max_size,
            bytes: UnsafeCell::new(self.read_bytes().to_vec()),
            schema: self.schema.clone()
        }
    }
}

#[doc(hidden)]
impl NP_Memory_Owned {

    #[inline(always)]
    pub fn existing(bytes: Vec<u8>, schema: *const Vec<NP_Parsed_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(bytes),
            schema: schema
        }
    }

    #[inline(always)]
    pub fn new(capacity: Option<usize>, schema: *const Vec<NP_Parsed_Schema>, root: usize) -> Self {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // is_packed, size, root pointer
        new_bytes.extend(&[0u8; 6]);

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(new_bytes),
            schema: schema
        }
    }

}

impl<'memory> NP_Mem_New for NP_Memory_Owned {
    fn new_empty(&self, capacity: Option<usize>) -> Result<Self, NP_Error> {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // is_packed, size, root pointer
        new_bytes.extend(&[0u8; 6]);

        Ok(Self {
            root: self.get_root(),
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(new_bytes),
            schema: self.schema
        })
    }
}

impl<'memory> NP_Memory for NP_Memory_Owned {

    fn set_max_length(&mut self, len: usize) {
        self.max_size = usize::min(u32::MAX as usize, len);
    }

    #[inline(always)]
    fn kind(&self) -> NP_Memory_Kind {
        NP_Memory_Kind::Owned
    }

    #[inline(always)]
    fn length(&self) -> usize {
        unsafe { &*self.bytes.get() }.len()
    }

    #[inline(always)]
    fn is_mutable(&self) -> bool {
        true
    }

    #[inline(always)]
    fn get_root(&self) -> usize {
        self.root
    }

    #[inline(always)]
    fn get_schemas(&self) -> &Vec<NP_Parsed_Schema> {
        unsafe { &*self.schema }
    }

    #[inline(always)]
    fn get_schema(&self, idx: usize) -> &NP_Parsed_Schema {
        &(unsafe { &*self.schema })[idx]
    }

    #[inline(always)]
    fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {
        let self_bytes = unsafe { &mut *self.bytes.get() };

        let location = self_bytes.len();

        // not enough space left?
        if location + bytes.len() >= self.max_size {
            return Err(NP_Error::new("Not enough space available in buffer!"))
        }

        self_bytes.extend_from_slice(bytes);
        Ok(location)
    }

    #[inline(always)]
    fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error> {
        self.malloc_borrow(&bytes)
    }

    #[inline(always)]
    fn read_bytes(&self) -> &[u8] {
        let self_bytes = unsafe { &*self.bytes.get() };
        self_bytes
    }   

    #[inline(always)]
    fn write_bytes(&self) -> &mut [u8] {
        let self_bytes = unsafe { &mut *self.bytes.get() };
        self_bytes
    }

    #[inline(always)]
    fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = unsafe { &*self.bytes.get() };
 
        Some(self_bytes[address])
    }

    #[inline(always)]
    fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]> {

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
    fn get_4_bytes(&self, address: usize) -> Option<&[u8; 4]> {

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
    fn get_8_bytes(&self, address: usize) -> Option<&[u8; 8]> {

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
    fn get_16_bytes(&self, address: usize) -> Option<&[u8; 16]> {

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
    fn get_32_bytes(&self, address: usize) -> Option<&[u8; 32]> {

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

    fn dump(self) -> Vec<u8> {
        self.bytes.into_inner()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Memory_Ref<'memory> {
    bytes: &'memory [u8],
    pub root: usize,
    pub max_size: usize,
    pub schema: &'memory Vec<NP_Parsed_Schema>
}

impl<'memory> Clone for NP_Memory_Ref<'memory> {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            max_size: self.max_size,
            bytes: self.bytes.clone(),
            schema: self.schema
        }
    }
}

impl<'memory> NP_Mem_New for NP_Memory_Ref<'memory> {
    fn new_empty(&self, _capacity: Option<usize>) -> Result<Self, NP_Error> {
        Err(NP_Error::MemoryReadOnly)
    }
}

#[doc(hidden)]
impl<'memory> NP_Memory_Ref<'memory> {


    #[inline(always)]
    pub fn existing(bytes: &'memory [u8], schema: &'memory Vec<NP_Parsed_Schema>, root: usize) -> Self {

        Self {
            root,
            max_size: u32::MAX as usize,
            bytes: bytes,
            schema: schema
        }
    }
}

impl<'memory> NP_Memory for NP_Memory_Ref<'memory> {

    fn set_max_length(&mut self, len: usize) {
        self.max_size = usize::min(u32::MAX as usize, len);
    }

    #[inline(always)]
    fn kind(&self) -> NP_Memory_Kind {
        NP_Memory_Kind::Ref
    }

    #[inline(always)]
    fn length(&self) -> usize {
        self.bytes.len()
    }

    #[inline(always)]
    fn is_mutable(&self) -> bool {
        false
    }

    #[inline(always)]
    fn get_root(&self) -> usize {
        self.root
    }

    #[inline(always)]
    fn get_schemas(&self) -> &Vec<NP_Parsed_Schema> {
        self.schema
    }

    #[inline(always)]
    fn get_schema(&self, idx: usize) -> &NP_Parsed_Schema {
        &self.schema[idx]
    }

    #[inline(always)]
    fn malloc_borrow(&self, _bytes: &[u8])  -> Result<usize, NP_Error> {
        Err(NP_Error::MemoryReadOnly)
    }

    #[inline(always)]
    fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error> {
        self.malloc_borrow(&bytes)
    }

    #[inline(always)]
    fn read_bytes(&self) -> &[u8] {
        self.bytes
    }   

    #[inline(always)]
    fn write_bytes(&self) -> &mut [u8] {
        unsafe {
            let const_ptr = self.bytes as *const [u8];
            let mut_ptr = const_ptr as *mut [u8];
            &mut *mut_ptr
        }
    }

    #[inline(always)]
    fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;
 
        Some(self_bytes[address])
    }

    #[inline(always)]
    fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;

        if self_bytes.len() < address + 2 {
            return None;
        }

        let slice = &self_bytes[address..(address + 2)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 2]) })
    }

    #[inline(always)]
    fn get_4_bytes(&self, address: usize) -> Option<&[u8; 4]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;

        if self_bytes.len() < address + 4 {
            return None;
        }

        let slice = &self_bytes[address..(address + 4)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 4]) })
    }

    #[inline(always)]
    fn get_8_bytes(&self, address: usize) -> Option<&[u8; 8]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;

        if self_bytes.len() < address + 8 {
            return None;
        }

        let slice = &self_bytes[address..(address + 8)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 8]) })
    }

    #[inline(always)]
    fn get_16_bytes(&self, address: usize) -> Option<&[u8; 16]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;

        if self_bytes.len() < address + 16 {
            return None;
        }

        let slice = &self_bytes[address..(address + 16)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 16]) })
    }

    #[inline(always)]
    fn get_32_bytes(&self, address: usize) -> Option<&[u8; 32]> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.bytes;

        if self_bytes.len() < address + 32 {
            return None;
        }

        let slice = &self_bytes[address..(address + 32)];

        Some(unsafe { &*(slice as *const [u8] as *const [u8; 32]) })
    }

    fn dump(self) -> Vec<u8> {
        self.bytes.to_vec()
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub enum Bytes_Ref<'bytes> {
    Value { b: &'bytes mut [u8] },
    Owned { b: Vec<u8> }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct NP_Memory_Ref_Mut<'memory> {
    bytes: UnsafeCell<Bytes_Ref<'memory>>,
    kind: UnsafeCell<NP_Memory_Kind>,
    pub max_size: usize,
    pub root: usize,
    pub schema: SchemaVec<'memory>
}

impl<'memory> Clone for NP_Memory_Ref_Mut<'memory> {
    fn clone(&self) -> Self {
        Self {
            root: self.root,
            max_size: self.max_size,
            bytes: UnsafeCell::new(Bytes_Ref::Owned { b: self.read_bytes().to_vec() }),
            kind: UnsafeCell::new(NP_Memory_Kind::Owned),
            schema: self.schema.clone()
        }
    }
}

impl<'memory> NP_Mem_New for NP_Memory_Ref_Mut<'memory> {
    fn new_empty(&self, capacity: Option<usize>) -> Result<Self, NP_Error> {
        let use_size = match capacity {
            Some(x) => x,
            None => 1024
        };

        let mut new_bytes = Vec::with_capacity(use_size);

        // is_packed, size, root pointer
        new_bytes.extend(&[0u8; 6]);

        Ok(Self {
            root: self.get_root(),
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(Bytes_Ref::Owned { b: new_bytes }),
            kind: UnsafeCell::new(NP_Memory_Kind::Owned),
            schema: SchemaVec::Owned(self.get_schemas().clone())
        })
    }
}

#[doc(hidden)]
impl<'memory> NP_Memory_Ref_Mut<'memory> {

    #[inline(always)]
    pub fn new(bytes: &'memory mut [u8], schema: &'memory Vec<NP_Parsed_Schema>, root: usize) -> Self {

        Self {
            root,
            kind: UnsafeCell::new(NP_Memory_Kind::RefMut { len: 6 }),
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(Bytes_Ref::Value { b: bytes }),
            schema: SchemaVec::Borrowed(schema)
        }
    }

    #[inline(always)]
    pub fn existing(bytes: &'memory mut [u8], len: usize, schema: &'memory Vec<NP_Parsed_Schema>, root: usize) -> Self {
        Self {
            root,
            kind: UnsafeCell::new(NP_Memory_Kind::RefMut { len }),
            max_size: u32::MAX as usize,
            bytes: UnsafeCell::new(Bytes_Ref::Value { b: bytes }),
            schema: SchemaVec::Borrowed(schema)
        }
    }
}

impl<'memory> NP_Memory for NP_Memory_Ref_Mut<'memory> {

    fn set_max_length(&mut self, len: usize) {
        match unsafe { &*self.kind.get() } {
            NP_Memory_Kind::RefMut { .. } => {
                // Picks the smallest of these 3 numbers:
                // 1. address space size (maximum limit, period)
                // 2. The size of the ref mut buffer (also a hard limit)
                // 3. The requested max length from the user
                self.max_size = usize::min(u32::MAX as usize, usize::min(self.read_bytes().len(), len));
            },
            NP_Memory_Kind::Owned => {
                self.max_size = usize::min(u32::MAX as usize, len);
            },
            _ => { } // unreachable
        }
        
    }

    #[inline(always)]
    fn kind(&self) -> NP_Memory_Kind {
        unsafe { *self.kind.get() }
    }

    #[inline(always)]
    fn length(&self) -> usize {
        match unsafe { &*self.kind.get() } {
            NP_Memory_Kind::RefMut { len } => *len,
            NP_Memory_Kind::Owned => self.read_bytes().len(),
            _ => 0 // unreachable
        }
    }

    fn set_length(&mut self, len: usize) -> Result<(), NP_Error> {
        self.kind = UnsafeCell::new(NP_Memory_Kind::RefMut { len: len });
        Ok(())
    }

    #[inline(always)]
    fn is_mutable(&self) -> bool {
        true
    }

    #[inline(always)]
    fn get_root(&self) -> usize {
        self.root
    }

    #[inline(always)]
    fn get_schemas(&self) -> &Vec<NP_Parsed_Schema> {
        self.schema.get()
    }

    #[inline(always)]
    fn get_schema(&self, idx: usize) -> &NP_Parsed_Schema {
        &self.schema.get()[idx]
    }

    #[inline(always)]
    fn malloc_borrow(&self, bytes: &[u8])  -> Result<usize, NP_Error> {
        match unsafe { &mut *self.kind.get() } {
            NP_Memory_Kind::RefMut { len } => {
                let self_bytes = self.write_bytes();

                let location = *len;

                if location + bytes.len() >= self.max_size {
                    return Err(NP_Error::new("Not enough space available in buffer!"))
                }

                for (x, b) in bytes.iter().enumerate() {
                    self_bytes[location + x] = *b;
                }

                *len += bytes.len();

                Ok(location)
            },
            NP_Memory_Kind::Owned => {
                match unsafe { &mut *self.bytes.get() } {
                    Bytes_Ref::Owned { b} => {
                        let location = self.read_bytes().len();

                        if location + bytes.len() >= self.max_size {
                            return Err(NP_Error::new("Not enough space available in buffer!"))
                        }
        
                        b.extend_from_slice(bytes);
        
                        Ok(location)
                    },
                    _ => Err(NP_Error::Unreachable)
                }
            },
            _ => Err(NP_Error::Unreachable)
        }
    }

    #[inline(always)]
    fn malloc(&self, bytes: Vec<u8>) -> Result<usize, NP_Error> {
        self.malloc_borrow(&bytes)
    }

    #[inline(always)]
    fn read_bytes(&self) -> &[u8] {
        match unsafe { &*self.bytes.get() } {
            Bytes_Ref::Value { b} => b,
            Bytes_Ref::Owned { b } => b
        }
    }   

    #[inline(always)]
    fn write_bytes(&self) -> &mut [u8] {
        match unsafe { &mut *self.bytes.get() } {
            Bytes_Ref::Value { b} => *b,
            Bytes_Ref::Owned { b} => b
        }
    }

    #[inline(always)]
    fn get_1_byte(&self, address: usize) -> Option<u8> {

        // empty value
        if address == 0 {
            return None;
        }

        let self_bytes = self.read_bytes();
 
        Some(self_bytes[address])
    }

    #[inline(always)]
    fn get_2_bytes(&self, address: usize) -> Option<&[u8; 2]> {

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
    fn get_4_bytes(&self, address: usize) -> Option<&[u8; 4]> {

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
    fn get_8_bytes(&self, address: usize) -> Option<&[u8; 8]> {

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
    fn get_16_bytes(&self, address: usize) -> Option<&[u8; 16]> {

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
    fn get_32_bytes(&self, address: usize) -> Option<&[u8; 32]> {

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

    fn dump(self) -> Vec<u8> {
        let len = self.length();
        match self.bytes.into_inner() {
            Bytes_Ref::Value { b} => b[0..len].to_vec(),
            Bytes_Ref::Owned { b } => b
        }
    }
}