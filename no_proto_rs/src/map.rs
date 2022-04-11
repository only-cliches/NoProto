use alloc::string::String;
use alloc::vec::Vec;

use crate::error::NP_Error;
use core::fmt::{Debug, Formatter};

pub static HASH_SEED: u32 = 2181155409;

// #[derive(Clone)]
// pub struct NP_HashMap<V: Debug + PartialEq> {
//     data: Vec<Vec<(u32, V)>>,
//     keys: Vec<(u32, String)>
// }

#[derive(PartialEq, Clone)]
pub struct NP_OrderedMap<V: Debug + PartialEq> {
    pub data: Vec<(String, V)>
}

impl<V: Debug + PartialEq> Default for NP_OrderedMap<V> {
    fn default() -> Self {
        NP_OrderedMap::new()
    }
}

// impl<V: Debug + PartialEq> PartialEq for NP_OrderedMap<V> {
//     fn eq(&self, other: &Self) -> bool {

//         if self._read().len() != other._read().len() {
//             return false;
//         }

//         if self._read() != other._read() {
//             return false;
//         }

//         return true;
//     }
// }

impl<T: Debug + PartialEq> Debug for NP_OrderedMap<T> {
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

// const HASH_SIZE: usize = 4096;

// impl<V: Debug + PartialEq> NP_HashMap<V> {

//     pub fn empty() -> Self {
//         Self { data: Vec::with_capacity(1), keys: Vec::new() }
//     }

//     pub fn new() -> Self {
//         let mut vector = Vec::with_capacity(HASH_SIZE);
//         vector.extend((0..HASH_SIZE).map(|_| Vec::with_capacity(4)));
//         Self { data: vector, keys: Vec::new() }
//     }

//     pub fn set(&mut self, key: &str, value: V) -> Result<(), NP_Error> {

//         let hash = murmurhash3_x86_32(key.as_bytes(), HASH_SEED);

//         let bucket = hash as usize % HASH_SIZE;

//         if self.data[bucket].len() == 0 {
//             self.data[bucket].push((hash, value));
//             self.keys.push((hash, String::from(key)));
//         } else {
//             // replace existing value
//             for (k, v) in self.data[bucket].iter_mut() {
//                 if *k == hash {
//                     *v = value;
//                     return Ok(())
//                 }
//             }
//             // add new value
//             self.data[bucket].push((hash, value));
//             self.keys.push((hash, String::from(key)));
//         }

//         Ok(())
//     }

//     fn get_by_hash(&self, hash: u32) -> Option<&V> {
//         let bucket = hash as usize % HASH_SIZE;

//         match self.data.get(bucket) {
//             Some(x) => {
//                 let len = x.len();
//                 if len == 0 {
//                     return None;
//                 }
//                 if len == 1 {
//                     return if x[0].0 == hash {
//                         Some(&x[0].1)
//                     } else {
//                         None
//                     }
//                 }
//                 for (k, v) in x.iter() {
//                     if *k == hash {
//                         return Some(v)
//                     }
//                 }
//                 None
//             },
//             None => None
//         }
//     }

//     pub fn get(&self, key: &str) -> Option<&V> {
//         let hash = murmurhash3_x86_32(key.as_bytes(), HASH_SEED);
//         self.get_by_hash(hash)
//     }

//     pub fn delete(&mut self, key: &str) {
//         let hash = murmurhash3_x86_32(key.as_bytes(), HASH_SEED);
//         let bucket = hash as usize % HASH_SIZE;
//         self.keys.retain(|(h, _key)| hash != *h);
//         match self.data.get_mut(bucket) {
//             Some(bucket) => {
//                 bucket.retain(|(k, _v)| *k != hash);
//             },
//             _ => { }
//         }
//     }

//     pub fn iter(&self) -> NP_HashMap_Iterator<V> {
//         NP_HashMap_Iterator { hashmap: self, index: 0, length: self.keys.len() }
//     }

//     pub fn iter_keys(&self) -> NP_HashMap_Iterator_Keys<V> {
//         NP_HashMap_Iterator_Keys { hashmap: self, index: 0, length: self.keys.len() }
//     }

//     pub fn keys(&self) -> &Vec<(u32, String)> {
//         &self.keys
//     }

// }


// const HASH_SIZE: usize = 4096;

impl<V: Debug + PartialEq> NP_OrderedMap<V> {

    pub fn empty() -> Self {
        NP_OrderedMap { data: Vec::with_capacity(1) }
    }

    pub fn new() -> Self {
        NP_OrderedMap { data: Vec::with_capacity(1024) }
    }

    pub fn set(&mut self, key: &str, value: V) {

        if self.data.len() == 0 {
            self.data.push((String::from(key), value));
            return
        }

        match self.data.binary_search_by(|(k, _)| k.as_str().cmp(key)) {
            Ok(pos) => { // found in list
                self.data[pos].1 = value;
            },
            Err(pos) => { // not found, but insert position found
                self.data.insert(pos, (String::from(key), value))
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        match self.data.binary_search_by(|(k, _)| k.as_str().cmp(key)) {
            Ok(pos) => Some(&self.data[pos].1),
            Err(_) => None
        }
    }

    pub fn del(&mut self, key: &str) {
        match self.data.binary_search_by(|(k, _)| k.as_str().cmp(key)) {
            Ok(pos) => {
                self.data.remove(pos);
            },
            Err(_) => {
                // do nothing
            }
        };
    }

    pub fn iter(&self) -> NP_HashMap_Iterator<V> {
        NP_HashMap_Iterator { hashmap: self, index: 0, length: self.data.len() }
    }

    pub fn iter_keys(&self) -> NP_HashMap_Iterator_Keys<V> {
        NP_HashMap_Iterator_Keys { hashmap: self, index: 0, length: self.data.len() }
    }

    pub fn _read(&self) -> &Vec<(String, V)> {
        &self.data
    }

}


pub struct NP_HashMap_Iterator_Keys<'iter, V: Debug + PartialEq> {
    hashmap: &'iter NP_OrderedMap<V>,
    length: usize,
    index: usize
}

impl<'iter, V: Debug + PartialEq> Iterator for NP_HashMap_Iterator_Keys<'iter, V> {
    type Item = &'iter String;

    fn next(&mut self) -> Option<Self::Item> {

        if self.index >= self.length {
            return None
        }

        let key = &self.hashmap._read()[self.index].0;

        self.index += 1;

        Some(key)
    }
}

pub struct NP_HashMap_Iterator<'iter, V: Debug + PartialEq> {
    hashmap: &'iter NP_OrderedMap<V>,
    length: usize,
    index: usize
}

impl<'iter, V: Debug + PartialEq> Iterator for NP_HashMap_Iterator<'iter, V> {
    type Item = (&'iter String, &'iter V);

    fn next(&mut self) -> Option<Self::Item> {

        if self.index >= self.length {
            return None
        }

        let (key, value) = &self.hashmap._read()[self.index];

        self.index += 1;
        return Some((key, value));
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

fn fmix64(mut k: u64) -> u64 {
    k ^= k >> 33;
    k = k.wrapping_mul(0xff51afd7ed558ccdu64);
    k ^= k >> 33;
    k = k.wrapping_mul(0xc4ceb9fe1a85ec53u64);
    k ^= k >> 33;

    return k;
}

fn get_128_block(bytes: &[u8], index: usize) -> (u64, u64) {
    let b64: &[u64] = unsafe { core::mem::transmute(bytes) };
    return (b64[index], b64[index + 1]);
}

pub fn murmurhash3_x64_128(bytes: &[u8], seed: u64) -> (u64, u64) {
    let c1 = 0x87c37b91114253d5u64;
    let c2 = 0x4cf5ad432745937fu64;
    let read_size = 16;
    let len = bytes.len() as u64;
    let block_count = len / read_size;

    let (mut h1, mut h2) = (seed, seed);


    for i in 0..block_count as usize {
        let (mut k1, mut k2) = get_128_block(bytes, i * 2);

        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;

        h1 = h1.rotate_left(27);
        h1 = h1.wrapping_add(h2);
        h1 = h1.wrapping_mul(5);
        h1 = h1.wrapping_add(0x52dce729);

        k2 = k2.wrapping_mul(c2);
        k2 = k2.rotate_left(33);
        k2 = k2.wrapping_mul(c1);
        h2 ^= k2;

        h2 = h2.rotate_left(31);
        h2 = h2.wrapping_add(h1);
        h2 = h2.wrapping_mul(5);
        h2 = h2.wrapping_add(0x38495ab5);
    }


    let (mut k1, mut k2) = (0u64, 0u64);

    if len & 15 == 15 { k2 ^= (bytes[(block_count * read_size) as usize + 14] as u64) << 48; }
    if len & 15 >= 14 { k2 ^= (bytes[(block_count * read_size) as usize + 13] as u64) << 40; }
    if len & 15 >= 13 { k2 ^= (bytes[(block_count * read_size) as usize + 12] as u64) << 32; }
    if len & 15 >= 12 { k2 ^= (bytes[(block_count * read_size) as usize + 11] as u64) << 24; }
    if len & 15 >= 11 { k2 ^= (bytes[(block_count * read_size) as usize + 10] as u64) << 16; }
    if len & 15 >= 10 { k2 ^= (bytes[(block_count * read_size) as usize +  9] as u64) <<  8; }
    if len & 15 >=  9 { k2 ^=  bytes[(block_count * read_size) as usize +  8] as u64;
        k2 = k2.wrapping_mul(c2);
        k2 = k2.rotate_left(33);
        k2 = k2.wrapping_mul(c1);
        h2 ^= k2;
    }

    if len & 15 >= 8 { k1 ^= (bytes[(block_count * read_size) as usize + 7] as u64) << 56; }
    if len & 15 >= 7 { k1 ^= (bytes[(block_count * read_size) as usize + 6] as u64) << 48; }
    if len & 15 >= 6 { k1 ^= (bytes[(block_count * read_size) as usize + 5] as u64) << 40; }
    if len & 15 >= 5 { k1 ^= (bytes[(block_count * read_size) as usize + 4] as u64) << 32; }
    if len & 15 >= 4 { k1 ^= (bytes[(block_count * read_size) as usize + 3] as u64) << 24; }
    if len & 15 >= 3 { k1 ^= (bytes[(block_count * read_size) as usize + 2] as u64) << 16; }
    if len & 15 >= 2 { k1 ^= (bytes[(block_count * read_size) as usize + 1] as u64) <<  8; }
    if len & 15 >= 1 { k1 ^=  bytes[(block_count * read_size) as usize + 0] as u64;
        k1 = k1.wrapping_mul(c1);
        k1 = k1.rotate_left(31);
        k1 = k1.wrapping_mul(c2);
        h1 ^= k1;
    }

    h1 ^= bytes.len() as u64;
    h2 ^= bytes.len() as u64;

    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    h1 = fmix64(h1);
    h2 = fmix64(h2);

    h1 = h1.wrapping_add(h2);
    h2 = h2.wrapping_add(h1);

    return (h1, h2);
}