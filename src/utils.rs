use core::str;
use alloc::string::String;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use crate::{error::NP_Error, schema::NP_TypeKeys};

const KX: u32 = 123456789;
const KY: u32 = 362436069;
const KZ: u32 = 521288629;
const KW: u32 = 88675123;

#[inline(always)]
pub fn to_unsigned(byte: u8) -> u8 {
    if byte >= 128 { byte - 128 } else { byte + 128 }
}

#[inline(always)]
pub fn to_signed(byte: u8) -> u8 {
    if byte < 128 { byte + 128 } else { byte - 128 }
}

pub struct Rand {
    x: u32, y: u32, z: u32, w: u32
}

impl Rand {
    pub fn new(seed: u32) -> Rand {
        Rand{
            x: KX^seed, y: KY^seed,
            z: KZ, w: KW
        }
    }

    // Xorshift 128, taken from German Wikipedia
    pub fn rand(&mut self) -> u32 {
        let t = self.x^self.x.wrapping_shl(11);
        self.x = self.y; self.y = self.z; self.z = self.w;
        self.w ^= self.w.wrapping_shr(19)^t^t.wrapping_shr(8);
        return self.w;
    }

    pub fn gen_range(&mut self, a: i32, b: i32) -> i32 {
        let m = (b-a+1) as u32;
        return a+(self.rand()%m) as i32;
    }
}

pub fn from_utf8_lossy(input: &[u8]) -> String {
    let mut empty = String::from("");

    loop {
        match str::from_utf8(input) {
            Ok(valid) => {
                empty.push_str(valid);
                break
            }
            Err(error) => {
                let (valid, _after_valid) = input.split_at(error.valid_up_to());
                unsafe {
                    empty.push_str(str::from_utf8_unchecked(valid))
                }
                empty.push_str("\u{FFFD}");

                if let Some(_invalid_sequence_length) = error.error_len() {
                    empty.push_str("?");
                } else {
                    break
                }
            }
        }
    }

    empty
}

pub fn overflow_error(kind: &str, path: &Vec<&str>, path_index: usize) -> Result<(), NP_Error> {

    if path.len() > 0 && (path.len() - 1) < path_index {
        let mut err = "Error in ".to_owned();
        err.push_str(kind);
        err.push_str(", this method does not work for collection types. Path: ");
        err.push_str(print_path(&path, path_index).as_str());
        return Err(NP_Error::new(err));
    }

    Ok(())
}

pub fn type_error(schema_type: &(u8, String, NP_TypeKeys), casting_type: &(u8, String, NP_TypeKeys), path: &Vec<&str>, path_index: usize) -> Result<(), NP_Error> {
    if schema_type.0 != casting_type.0 {
        let mut err = "TypeError: Attempted to get value for type (".to_owned();
        err.push_str(casting_type.1.as_str());
        err.push_str(") from schema of type (");
        err.push_str(schema_type.1.as_str());
        err.push_str(") Path: ");
        err.push_str(print_path(&path, path_index).as_str());
        return Err(NP_Error::new(err));
    }

    Ok(())
}

pub fn print_path(path: &Vec<&str>, path_index: usize) -> String {
    let mut path_str: String = "".to_owned();
    let mut ct: usize = 0;
    path.iter().for_each(|v| {
        if ct == path_index {
            path_str.push_str(">");
        }
        path_str.push_str(v);
        if ct == path_index {
            path_str.push_str("<");
        }
        path_str.push_str(" ");
        ct += 1;
    });
    path_str
}

pub fn to_base32(num: u128, length: i32) -> String {

    let mut result: Vec<&str> = Vec::with_capacity(length as usize);
    for _x in 0..length {
        result.push("");
    }

    let base_values: [&str; 32] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "A", "B", "C", "D", "E", "F", "G", "H", "J", "K", "M", "N", "P", "Q", "R", "S", "T", "V", "W", "X", "Y", "Z"];

    let mut value = num;
    let i = length - 1;
    for x in 0..i {
        let modulus = value % 32; 
        result[(i - x) as usize] = base_values[modulus as usize];
        value = (value - modulus) / 32;
    }

    let mut final_string: String = "".to_owned();
    for x in result {
        final_string.push_str(x);
    }

    final_string
}