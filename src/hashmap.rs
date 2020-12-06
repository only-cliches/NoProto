use alloc::vec::Vec;

use crate::error::NP_Error;


pub static SEED: u32 = 2181155409;

#[derive(Debug)]
pub struct NP_HashMap {
    data: Vec<usize>,
    size: u16
}

impl NP_HashMap {

    pub fn new() -> Self {
        Self { data: Vec::with_capacity(255), size: 0 }
    }

    pub fn do_hash(key: &str) -> u32 {
        murmurhash3_x86_32(key.as_bytes(), SEED)
    }

    pub fn insert_hash(&mut self, hash: u32, value: usize) -> Result<(), NP_Error> {
        if self.size + 1 > 255 {
            return Err(NP_Error::new("Too many records in hash map!"));
        }

        self.size += 1;

        self.data[hash as usize] = value;

        Ok(())
    }

    pub fn insert(&mut self, key: &str, value: usize) -> Result<u32, NP_Error> {
        
        if self.size + 1 > 255 {
            return Err(NP_Error::new("Too many records in hash map!"));
        }

        self.size += 1;

        let hash = murmurhash3_x86_32(key.as_bytes(), SEED);
        self.data[hash as usize] = value;

        Ok(hash)
    }

    pub fn get_hash(&self, key: u32) -> Option<&usize> {
        self.data.get(key as usize)
    }

    pub fn get(&self, key: &str) -> Option<&usize> {
        let hash = murmurhash3_x86_32(key.as_bytes(), SEED) as usize;
        self.data.get(hash)
    }

    pub fn delete(&mut self, key: &str) {
        self.size -= 1;
        let hash = murmurhash3_x86_32(key.as_bytes(), SEED) as usize;
        self.data.remove(hash);
    }
}

// https://github.com/mhallin/murmurhash3-rs
// 
// The MIT License (MIT)

// Copyright (c) 2015 Magnus Hallin

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.


fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;

    return h;
}

fn get_32_block(bytes: &[u8], index: usize) -> u32 {
    let b32: &[u32] = unsafe { core::mem::transmute(bytes) };

    return b32[index];
}

pub fn murmurhash3_x86_32(bytes: &[u8], seed: u32) -> u32 {
    let c1 = 0xcc9e2d51u32;
    let c2 = 0x1b873593u32;
    let read_size = 4;
    let len = bytes.len() as u32;
    let block_count = len / read_size;

    let mut h1 = seed;

    for i in 0..block_count as usize {
        let mut k1 = get_32_block(bytes, i);

        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);

        h1 ^= k1;
        h1 = h1.rotate_left(13);
        h1 = h1.wrapping_mul(5);
        h1 = h1.wrapping_add(0xe6546b64)
    }
    let mut k1 = 0u32;

    if len & 3 == 3 { k1 ^= (bytes[(block_count * read_size) as usize + 2] as u32) << 16; }
    if len & 3 >= 2 { k1 ^= (bytes[(block_count * read_size) as usize + 1] as u32) << 8; }
    if len & 3 >= 1 { k1 ^=  bytes[(block_count * read_size) as usize + 0] as u32;
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(15);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
    }

    h1 ^= bytes.len() as u32;
    h1 = fmix32(h1);

    return h1;
}