use core::str;
use alloc::string::String;
use alloc::borrow::ToOwned;

const KX: u32 = 123456789;
const KY: u32 = 362436069;
const KZ: u32 = 521288629;
const KW: u32 = 88675123;

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

fn fract(num: f64) -> f64 {
    if num == 0f64 {
        0f64
    } else {
        num % 1f64
    }
}

fn floor(num: f64) -> f64 {
    let f = fract(num);
    if f.is_nan() || f == 0f64 {
        0f64
    } else if num < 0f64 {
        num - f - 1f64
    } else {
        num - f
    }
}

pub fn to_hex(num: u64, length: i32) -> String {
    let mut result: String = "".to_owned();

    let hex_values = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f"];

    let mut i = length - 1;
    while i >= 0 {
        let raise = (16i32).pow(i as u32) as f64;
        let index = floor(num as f64 / raise) as i32;
        result.push_str(hex_values[(index % 16i32) as usize]);
        i -= 1 ;
    }

    result
}