#![warn(missing_docs)]
#![allow(non_camel_case_types)]
#![no_std]


#[cfg(test)]
#[macro_use]
extern crate std;

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

#[allow(missing_docs)]
#[doc(hidden)]
pub mod hashmap;
mod utils;
mod error;
mod json_flex;
mod schema;

#[macro_use]
extern crate alloc;

// #[derive(Debug)]
// pub struct NP_Factory {
//     /// schema data used by this factory
//     pub schema: NP_Schema,
//     schema_bytes: Vec<u8>
// }
//
// unsafe impl Send for NP_Factory {}
// unsafe impl Sync for NP_Factory {}

/// When calling `maybe_compact` on a buffer, this struct is provided to help make a choice on wether to compact or not.
#[derive(Debug, Eq, PartialEq)]
pub struct NP_Size_Data {
    /// The size of the existing buffer
    pub current_buffer: usize,
    /// The estimated size of buffer after compaction
    pub after_compaction: usize,
    /// How many known wasted bytes in existing buffer
    pub wasted_bytes: usize
}


#[test]
fn threading_works() {
    // let fact = NP_Factory::new("string()").unwrap();
    // let buffer = fact.new_buffer(None);
    // std::thread::spawn(move || {
    //     let f = fact.export_schema_bytes();
    //     let b = buffer;
    //     assert_eq!(6, b.calc_bytes().unwrap().current_buffer);
    //     assert_eq!(8, f.len());
    // }).join().unwrap()
}