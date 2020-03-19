use core::str;
use alloc::string::String;

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

    pub fn shuffle<T>(&mut self, a: &mut [T]) {
        if a.len()==0 {return;}
        let mut i = a.len()-1;
        while i>0 {
            let j = (self.rand() as usize)%(i+1);
            a.swap(i,j);
            i-=1;
        }
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