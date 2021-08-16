use alloc::string::String;
use alloc::vec::Vec;

use crate::error::NP_Error;
use core::fmt::{Debug, Formatter};

pub static SEED: u32 = 2181155409;

#[derive(Clone)]
pub struct NP_HashMap<V: Debug + PartialEq> {
    data: Vec<Vec<(u32, V)>>,
    keys: Vec<(u32, String)>
}

impl<V: Debug + PartialEq> Default for NP_HashMap<V> {
    fn default() -> Self {
        NP_HashMap::new()
    }
}

impl<V: Debug + PartialEq> PartialEq for NP_HashMap<V> {
    fn eq(&self, other: &Self) -> bool {

        if self.iter_keys().count() != other.iter_keys().count() {
            return false;
        }

        for key in self.iter_keys() {
            if self.get(key) != other.get(key) {
                return false;
            }
        }

        return true;
    }
}

impl<T: Debug + PartialEq> Debug for NP_HashMap<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {

        f.write_str("NP_HashMap:: ")?;
        for (key, value) in self.iter() {
            f.write_str(key.as_str())?;
            f.write_str(": ")?;
            value.fmt(f)?;
            f.write_str("; ")?;
        }

        Ok(())
    }
}

const HASH_SIZE: usize = 4096;

impl<V: Debug + PartialEq> NP_HashMap<V> {

    pub fn empty() -> Self {
        Self { data: Vec::with_capacity(1), keys: Vec::new() }
    }

    pub fn new() -> Self {
        let mut vector = Vec::with_capacity(HASH_SIZE);
        vector.extend((0..HASH_SIZE).map(|_| Vec::with_capacity(4)));
        Self { data: vector, keys: Vec::new() }
    }

    pub fn set(&mut self, key: &str, value: V) -> Result<(), NP_Error> {

        let hash = murmurhash3_x86_32(key.as_bytes(), SEED);
    
        let bucket = hash as usize % HASH_SIZE;

        if self.data[bucket].len() == 0 {
            self.data[bucket].push((hash, value));
            self.keys.push((hash, String::from(key)));
        } else {
            // replace existing value
            for (k, v) in self.data[bucket].iter_mut() {
                if *k == hash {
                    *v = value;
                    return Ok(())
                }
            }
            // add new value
            self.data[bucket].push((hash, value));
            self.keys.push((hash, String::from(key)));
        }

        Ok(())
    }

    fn get_by_hash(&self, hash: u32) -> Option<&V> {
        let bucket = hash as usize % HASH_SIZE;

        match self.data.get(bucket) {
            Some(x) => {
                let len = x.len();
                if len == 0 {
                    return None;
                }
                if len == 1 {
                    return if x[0].0 == hash {
                        Some(&x[0].1)
                    } else {
                        None
                    }
                }
                for (k, v) in x.iter() {
                    if *k == hash {
                        return Some(v)
                    }
                }
                None
            },
            None => None
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        let hash = murmurhash3_x86_32(key.as_bytes(), SEED);
        self.get_by_hash(hash)
    }

    pub fn delete(&mut self, key: &str) {
        let hash = murmurhash3_x86_32(key.as_bytes(), SEED);
        let bucket = hash as usize % HASH_SIZE;
        self.keys.retain(|(h, _key)| hash != *h);
        match self.data.get_mut(bucket) {
            Some(bucket) => {
                bucket.retain(|(k, _v)| *k != hash);
            },
            _ => { }
        }
    }

    pub fn iter(&self) -> NP_HashMap_Iterator<V> {
        NP_HashMap_Iterator { hashmap: self, index: 0, length: self.keys.len() }
    }

    pub fn iter_keys(&self) -> NP_HashMap_Iterator_Keys<V> {
        NP_HashMap_Iterator_Keys { hashmap: self, index: 0, length: self.keys.len() }
    }

    pub fn keys(&self) -> &Vec<(u32, String)> {
        &self.keys
    }

}


pub struct NP_HashMap_Iterator_Keys<'iter, V: Debug + PartialEq> {
    hashmap: &'iter NP_HashMap<V>,
    length: usize,
    index: usize
}

impl<'iter, V: Debug + PartialEq> Iterator for NP_HashMap_Iterator_Keys<'iter, V> {
    type Item = &'iter String;

    fn next(&mut self) -> Option<Self::Item> {

        if self.index >= self.length {
            return None
        }

        let key = &self.hashmap.keys[self.index];

        self.index += 1;

        Some(&key.1)
    }
}

pub struct NP_HashMap_Iterator<'iter, V: Debug + PartialEq> {
    hashmap: &'iter NP_HashMap<V>,
    length: usize,
    index: usize
}

impl<'iter, V: Debug + PartialEq> Iterator for NP_HashMap_Iterator<'iter, V> {
    type Item = (&'iter String, &'iter V);

    fn next(&mut self) -> Option<Self::Item> {

        if self.index >= self.length {
            return None
        }

        let key = &self.hashmap.keys[self.index];

        if let Some(value) = self.hashmap.get_by_hash(key.0) {
            self.index += 1;
            return Some((&key.1, value))
        }

        None
    }
}



// #[test]
// fn hash_map_test() {
//     let mut hash: NP_HashMap<u32> = NP_HashMap::new();
//
//     hash.set("hello", 32);
//     hash.set("world", 52);
//     hash.set("another", 22);
//
//     // println!("{:?}", hash.get("world"));
//
//     for (key, value) in hash.iter() {
//         println!("{} {:?}", key, value);
//     }
//
// }

// https://github.com/mhallin/murmurhash3-rs
// 
// The MIT License (MIT)
// 
// Copyright (c) 2015 Magnus Hallin
// 
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

#[inline(always)]
fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;

    return h;
}

#[inline(always)]
fn get_32_block(bytes: &[u8], index: usize) -> u32 {
    let real_index = index.wrapping_mul(4);
    let u32_bytes = &bytes[real_index..(real_index + 4)];

    return unsafe {
        let bytes = *(u32_bytes as *const [u8] as *const [u8; 4]);
        core::mem::transmute(bytes)
    }
}

#[inline(always)]
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