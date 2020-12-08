//! Represents a fixed point decimal number.
//! 
//! Allows floating point values to be stored without rounding errors, useful for storing financial data.
//! 
//! Do NOT perform calculations with `.to_float()` method, you'll make using this kind of moot.
//! 
//! NP_Dec values contain two parts:
//!     1. The actual number value (`num`)
//!     2. The position of the decimal point from the right (`exp`)
//! 
//! A value of "2039.756" could be stored as `NP_Dec {num: 2039756, exp: 3}`.  It could also be stored as: `NP_Dec {num: 203975600, exp: 5}`.
//! 
//! The range of possible floating point values depends on the `exp` value.  The `num` property is an i64 variable so it can safely store 9.22e18 to -9.22e18.  
//! 
//! If `exp` is zero, all values stored are whole numbers.
//! 
//! For every increase in `exp` by 1, the maximum range of possible values decreases by a power of 10.  For example at `exp = 1` the range drops to 9.22e17 to -9.22e17. 
//! However, each increase in `exp` provides a decimal point of precision.  In another example, at `exp = 5` you have 5 decimal points of precision and a max range of 9.22e13 to -9.22e13.
//! 
//! Essentially, increaseing the `exp` factor decreases the maximum range of possible values that can be stored in exchange for increased decimal precision.
//! 
//! `NP_Dec` values can safely be multiplied, added, devided, subtracted or compared with eachother.  It's a good idea to manually shift the `exp` values of two `NP_Dec` to match before performing any operation between them, otherwise the operation might not do what you expect.
//! 
//! When `NP_Dec` values are pulled out of a buffer, the `num` property is pulled from the buffer contents and the `exp` property comes from the schema.
//! 
//! ```
//! use no_proto::pointer::dec::NP_Dec;
//! 
//! // Creating a new NP_Dec for 20.49
//! let mut dec = NP_Dec::new(2049, 2);
//! 
//! // add 2
//! dec += NP_Dec::new(200, 2);
//! 
//! // add 0.03
//! dec += NP_Dec::new(3, 2);
//! 
//! // convert float then use it to minus 5
//! let mut f: NP_Dec = 5.0_f64.into();
//! f.shift_exp(2); // set new NP_Dec to `exp` of 2.
//! dec -= f; // subtract
//! 
//! assert_eq!(dec.to_float(), 17.52_f64);
//! 
//! ```
//! 
//! ```
//! use no_proto::error::NP_Error;
//! use no_proto::NP_Factory;
//! use no_proto::pointer::dec::NP_Dec;
//! 
//! let factory: NP_Factory = NP_Factory::new(r#"{
//!    "type": "dec",
//!    "exp": 2
//! }"#)?;
//!
//! let mut new_buffer = factory.empty_buffer(None);
//! new_buffer.set(&[], NP_Dec::new(50283, 2))?;
//! 
//! assert_eq!(502.83f64, new_buffer.get::<NP_Dec>(&[])?.unwrap().to_float());
//!
//! # Ok::<(), NP_Error>(()) 
//! ```
//!

use alloc::prelude::v1::Box;
use crate::utils::to_signed;
use crate::schema::{NP_Parsed_Schema};
use alloc::vec::Vec;
use crate::utils::to_unsigned;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_TypeKeys};
use crate::{pointer::NP_Value, error::NP_Error};
use core::{fmt::{Debug}, hint::unreachable_unchecked};

use alloc::borrow::ToOwned;
use super::{NP_Cursor, NP_Cursor_Addr};
use crate::NP_Memory;
use alloc::string::ToString;

/// Holds fixed decimal data.
/// 
/// Check out documentation [here](../dec/index.html).
/// 
#[derive(Clone, Copy, Debug)]
pub struct NP_Dec {
    /// The number being stored, does not include decimal point data
    pub num: i64,
    /// The exponent of this number
    pub exp: u8
}

impl super::NP_Scalar for NP_Dec {}

impl NP_Dec {
    /// Convert an NP_Dec into a native floating point value.
    /// 
    /// DO NOT use this to perform calculations, only to export/display the value.
    /// 
    /// ```
    /// use no_proto::pointer::dec::NP_Dec;
    ///     
    /// let my_num = NP_Dec::new(2203, 3); // value is 2.203
    /// 
    /// assert_eq!(my_num.to_float(), 2.203f64);
    /// ```
    /// 
    pub fn to_float(&self) -> f64 {
        let m = self.num as f64;
        let mut step = self.exp;
        let mut s = 1f64;
        while step > 0 {
            s *= 10f64;
            step -= 1;
        }
        m / s
    }

    /// Shift the exponent of this NP_Dec to a new value.
    /// 
    /// If the new `exp` value is higher than the old `exp` value, there may be an overflow of the i64 value.
    /// 
    /// If the new `exp` value is lower than the old one, information will likely be lost as decimal precision is being removed from the number.
    /// 
    /// ```
    /// use no_proto::pointer::dec::NP_Dec;
    /// 
    /// let mut my_num = NP_Dec::new(2203, 3); // value is 2.203
    /// 
    /// my_num.shift_exp(1); // set `exp` to 1 instead of 3.  This will force our value to 2.2
    /// 
    /// assert_eq!(my_num.to_float(), 2.2_f64); // notice we've lost the "03" at the end because of reducing the `exp` value. 
    /// 
    /// ```
    pub fn shift_exp(&mut self, new_exp: u8) -> NP_Dec {
        let diff = self.exp as i64 - new_exp as i64;

        let mut step = i64::abs(diff);

        if self.exp == new_exp { return *self }

        if diff < 0 { // moving decimal to right
            while step > 0 {
                self.num *= 10;
                step -=1;
            }
        } else { // moving decimal to left
            while step > 0 {
                self.num /= 10;
                step -=1;
            }
        }

        self.exp = new_exp;
        
        *self
    }

    /// Generate a new NP_Dec value
    /// 
    /// First argument is the `num` value, second is the `exp` or exponent.
    /// 
    /// ```
    /// use no_proto::pointer::dec::NP_Dec;
    /// 
    /// let x = NP_Dec::new(2, 0); // stores "2.00"
    /// assert_eq!(x.to_float(), 2f64);
    /// 
    /// let x = NP_Dec::new(2, 1); // stores "0.20"
    /// assert_eq!(x.to_float(), 0.2f64);
    /// 
    /// let x = NP_Dec::new(2, 2); // stores "0.02"
    /// assert_eq!(x.to_float(), 0.02f64);
    /// 
    /// let x = NP_Dec::new(5928, 1); // stores "592.8"
    /// assert_eq!(x.to_float(), 592.8f64);
    /// 
    /// let x = NP_Dec::new(59280, 2); // also stores "592.8"
    /// assert_eq!(x.to_float(), 592.8f64);
    /// 
    /// let x = NP_Dec::new(592800, 3); // also stores "592.8"
    /// assert_eq!(x.to_float(), 592.8f64);
    /// 
    /// ```
    pub fn new(num: i64, exp: u8) -> Self {
        NP_Dec { num, exp }
    }

    /// Given another NP_Dec value, match the `exp` value of this NP_Dec to the other one.  Returns a copy of the other NP_Dec.
    /// 
    /// This creates a copy of the other NP_Dec then shifts it's `exp` value to whatever self is, then returns that copy.
    /// 
    /// ```
    /// use no_proto::pointer::dec::NP_Dec;
    /// 
    /// let mut my_num = NP_Dec::new(2203, 3); // value is 2.203
    /// 
    /// let other_num = NP_Dec::new(50, 1); // value is 5.0
    /// 
    /// let matched_dec = my_num.match_exp(&other_num);
    /// // `exp` values match now! They're both 3.
    /// assert_eq!(matched_dec.exp, my_num.exp);
    /// ```
    /// 
    pub fn match_exp(&self, other: &NP_Dec) -> NP_Dec {
        let mut other_copy = other.clone();

        if other_copy.exp == self.exp {
            return other_copy
        }

        other_copy.shift_exp(self.exp);

        other_copy
    }

    /// Export NP_Dec to it's component parts.
    /// 
    /// ```
    /// use no_proto::pointer::dec::NP_Dec;
    /// 
    /// let my_num = NP_Dec::new(2203, 3); // value is 2.203
    /// 
    /// assert_eq!(my_num.export(), (2203i64, 3u8));
    /// ```
    pub fn export(&self) -> (i64, u8) {
        (self.num, self.exp)
    }
}

/// Check if two NP_Dec are equal or not equal
/// 
/// If the two `exp` values are not identical, unexpected results may occur due to rounding.
/// 
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let result = NP_Dec::new(202, 1) == NP_Dec::new(202, 1);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(202, 1) != NP_Dec::new(200, 1);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(202, 1) == NP_Dec::new(2020, 2);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(203, 1) != NP_Dec::new(2020, 2);
/// assert_eq!(result, true);
/// 
/// ```
impl core::cmp::PartialEq for NP_Dec {
    fn ne(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num != other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num != new_other.num;
        }
    }
    fn eq(&self, other: &NP_Dec) -> bool { 
        if self.exp == other.exp {
            return self.num == other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num == new_other.num;
        }
    }
}

/// Compare two NP_Dec
/// 
/// If the two `exp` values are not identical, unexpected results may occur due to rounding.
/// 
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let result = NP_Dec::new(203, 1) > NP_Dec::new(202, 1);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(202, 1) < NP_Dec::new(203, 1);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(20201, 2) > NP_Dec::new(202, 0);
/// assert_eq!(result, true);
/// 
/// let result = NP_Dec::new(20201, 2) == NP_Dec::new(2020100, 4);
/// assert_eq!(result, true);
/// ```
impl core::cmp::PartialOrd for NP_Dec {

    fn lt(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num < other.num;
        } else {
            let new_other = self.match_exp(other);
            return self.num < new_other.num;
        }
    }

    fn le(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num <= other.num;
        } else {
            let new_other = self.match_exp(other);
            return self.num <= new_other.num;
        }
    }

    fn gt(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num > other.num;
        } else {
            let new_other = self.match_exp(other);
            return self.num > new_other.num;
        }
    }

    fn ge(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num >= other.num;
        } else {
            let new_other = self.match_exp(other);
            return self.num >= new_other.num;
        }
    }

    fn partial_cmp(&self, other: &NP_Dec) -> Option<core::cmp::Ordering> { 

        let (a, b) = if self.exp == other.exp {
            (self.num, other.num)
        } else {
            let new_other = self.match_exp(other);
            (self.num, new_other.num)
        };

        if a > b {
            return Some(core::cmp::Ordering::Greater);
        } else if a < b {
            return Some(core::cmp::Ordering::Less);
        } else if a == b {
            return Some(core::cmp::Ordering::Equal);
        }

        return None;
    }
}


/// Converts an NP_Dec into an Int32, rounds to nearest whole number
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = NP_Dec::new(10123, 2);
/// let y: i32 = x.into();
/// 
/// assert_eq!(y, 101i32);
/// ```
impl Into<i32> for NP_Dec {
    fn into(self) -> i32 { 
        let mut change_value = self.num;
        let mut loop_val = self.exp;
        while loop_val > 0 {
            change_value /= 10;
            loop_val -= 1;
        }
        change_value as i32
    }
}

/// Converts an Int32 into a NP_Dec
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = 101i32;
/// let y: NP_Dec = x.into();
/// 
/// assert_eq!(y.num as i32, x);
/// ```
impl Into<NP_Dec> for i32 {
    fn into(self) -> NP_Dec { 
        NP_Dec::new(self as i64, 0)
    }
}


/// Converts an NP_Dec into an Int64, rounds to nearest whole number
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = NP_Dec::new(10123, 2);
/// let y: i64 = x.into();
/// 
/// assert_eq!(y, 101i64);
/// ```
impl Into<i64> for NP_Dec {
    fn into(self) -> i64 { 
        let mut change_value = self.num;
        let mut loop_val = self.exp;
        while loop_val > 0 {
            change_value /= 10;
            loop_val -= 1;
        }
        change_value
    }
}

/// Converts an Int64 into a NP_Dec
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = 101i64;
/// let y: NP_Dec = x.into();
/// 
/// assert_eq!(y.num, x);
/// ```
impl Into<NP_Dec> for i64 {
    fn into(self) -> NP_Dec { 
        NP_Dec::new(self, 0)
    }
}



fn round_f64(n: f64) -> f64 {
    let value = if n < 0.0 { n - 0.5 } else { n + 0.5 };

    let bounds_value = value.max(core::i64::MIN as f64).min(core::i64::MAX as f64);

    (bounds_value as i64) as f64
}

fn round_f32(n: f32) -> f32 {
    let value = if n < 0.0 { n - 0.5 } else { n + 0.5 };

    let bounds_value = value.max(core::i64::MIN as f32).min(core::i64::MAX as f32);

    (bounds_value as i64) as f32
}

fn round(n: f64, precision: u32) -> f64 {
    round_f64(n * 10_u32.pow(precision) as f64) / 10_i32.pow(precision) as f64
}

fn precision(x: f64) -> Option<u32> {
    for digits in 0..core::f64::DIGITS {
        if round(x, digits) == x {
            return Some(digits);
        }
    }
    None
}

fn round32(n: f32, precision: u32) -> f32 {
    round_f32(n * 10_u32.pow(precision) as f32) / 10_i32.pow(precision) as f32
}

fn precision32(x: f32) -> Option<u32> {
    for digits in 0..core::f64::DIGITS {
        if round32(x, digits) == x {
            return Some(digits);
        }
    }
    None
}

/// Converts a NP_Dec into a Float64
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = NP_Dec::new(10023, 2);
/// let y: f64 = x.into();
/// 
/// assert_eq!(y, x.to_float());
/// ```
impl Into<f64> for NP_Dec {
    fn into(self) -> f64 { 
        self.to_float()
    }
}

/// Converts a Float64 into a NP_Dec
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = 100.238f64;
/// let y: NP_Dec = x.into();
/// 
/// assert_eq!(y.to_float(), x);
/// ```
impl Into<NP_Dec> for f64 {
    fn into(self) -> NP_Dec { 
        match precision(self) {
            Some(x) => {
                let max_decimal_places = u32::min(x, 18);
                let mut new_self = self.clone();
                let mut loop_exp = max_decimal_places;
                while loop_exp > 0 {
                    new_self *= 10f64;
                    loop_exp -= 1;
                }
                let value = round_f64(new_self) as i64;
                return NP_Dec::new(value, max_decimal_places as u8);
            },
            None => { // this should be impossible, but just incase
                let value = round_f64(self) as i64;
                return NP_Dec::new(value, 0);
            }
        }
    }
}

/// Converts a NP_Dec into a Float32
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = NP_Dec::new(10023, 2);
/// let y: f32 = x.into();
/// 
/// assert_eq!(y, x.to_float() as f32);
/// ```
impl Into<f32> for NP_Dec {
    fn into(self) -> f32 { 
        self.to_float() as f32
    }
}

/// Converts a Float32 into a NP_Dec
/// ```
/// use no_proto::pointer::dec::NP_Dec;
/// 
/// let x = 100.238f32;
/// let y: NP_Dec = x.into();
/// 
/// assert_eq!(y.to_float() as f32, x);
/// ```
impl Into<NP_Dec> for f32 {
    fn into(self) -> NP_Dec { 
        match precision32(self) {
            Some(x) => {
                let max_decimal_places = u32::min(x, 18);
                let mut new_self = self.clone();
                let mut loop_exp = max_decimal_places;
                while loop_exp > 0 {
                    new_self *= 10f32;
                    loop_exp -= 1;
                }
                let value = round_f32(new_self) as i64;
                return NP_Dec::new(value, max_decimal_places as u8);
            },
            None => { // this should be impossible, but just incase
                let value = round_f32(self) as i64;
                return NP_Dec::new(value, 0);
            }
        }
    }
}

impl core::ops::DivAssign for NP_Dec { // a /= b
    fn div_assign(&mut self, other: NP_Dec) { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num / other_copy.num;
        } else {
            self.num = self.num / other.num;
        }
    }
}

impl core::ops::Div for NP_Dec { // a / b
    type Output = NP_Dec;
    fn div(mut self, other: NP_Dec) -> <Self as core::ops::Sub<NP_Dec>>::Output { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num / other_copy.num;
        } else {
            self.num = self.num / other.num;
        }
        return self;
    }
}

impl core::ops::SubAssign for NP_Dec { // a -= b
    fn sub_assign(&mut self, other: NP_Dec) { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num - other_copy.num;
        } else {
            self.num = self.num - other.num;
        }
    }
}

impl core::ops::Sub for NP_Dec { // a - b
    type Output = NP_Dec;
    fn sub(mut self, other: NP_Dec) -> <Self as core::ops::Sub<NP_Dec>>::Output { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num - other_copy.num;
        } else {
            self.num = self.num - other.num;
        }
        return self;
    }
}

impl core::ops::AddAssign for NP_Dec { // a += b
    fn add_assign(&mut self, other: NP_Dec) { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num + other_copy.num;
        } else {
            self.num = self.num + other.num;
        }
    }
}

impl core::ops::Add for NP_Dec { // a + b
    type Output = NP_Dec;
    fn add(mut self, other: NP_Dec) -> <Self as core::ops::Add<NP_Dec>>::Output { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num + other_copy.num;
        } else {
            self.num = self.num + other.num;
        }
        return self;
    }
}

impl core::ops::MulAssign for NP_Dec { // a *= b
    fn mul_assign(&mut self, other: NP_Dec) { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num * other_copy.num;
        } else {
            self.num = self.num * other.num;
        }
    }
}

impl core::ops::Mul for NP_Dec { // a * b
    type Output = NP_Dec;
    fn mul(mut self, other: NP_Dec) -> <Self as core::ops::Mul<NP_Dec>>::Output { 

        if self.exp != other.exp {
            let other_copy = self.match_exp(&other);
            self.num = self.num * other_copy.num;
        } else {
            self.num = self.num * other.num;
        }
        return self;
    }
}

impl Default for NP_Dec {
    fn default() -> Self { 
        NP_Dec::new(0,0)
     }
}


impl<'value> NP_Value<'value> for NP_Dec {

    fn type_idx() -> (&'value str, NP_TypeKeys) { ("decimal", NP_TypeKeys::Decimal) }
    fn self_type_idx(&self) -> (&'value str, NP_TypeKeys) { ("decimal", NP_TypeKeys::Decimal) }

    fn schema_to_json(schema: &Vec<NP_Parsed_Schema>, address: usize)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().0.to_string()));

        match &schema[address] {
            NP_Parsed_Schema::Decimal { i: _, sortable: _, default, exp} => {
                schema_json.insert("exp".to_owned(), NP_JSON::Integer(exp.clone() as i64));
    
                if let Some(d) = default {
                    let value = NP_Dec::new(d.num.clone(), exp.clone());
                    schema_json.insert("default".to_owned(), NP_JSON::Float(value.into()));
                }
        
                Ok(NP_JSON::Dictionary(schema_json))
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn schema_default(schema: &NP_Parsed_Schema) -> Option<Self> {

        match schema {
            NP_Parsed_Schema::Decimal { i: _, sortable: _, default, exp: _} => {
                if let Some(d) = default {
                    Some(d.clone())
                } else {
                    None
                }
            },
            _ => { unsafe { unreachable_unchecked() } }
        }
    }

    fn set_value<'set>(mut cursor: NP_Cursor_Addr, memory: &'set NP_Memory, value: Self) -> Result<NP_Cursor_Addr, NP_Error> where Self: 'set + Sized {


        let c = memory.get_parsed(&cursor);

        let mut value_address = c.value.get_addr_value() as usize;

        let exp = match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::Decimal { i: _, sortable: _, default: _, exp} => {
                exp
            },
            _ => { unsafe { unreachable_unchecked() } }
        };


        let mut cloned_value = (value).clone();
        cloned_value.shift_exp(exp);

        let i64_value = cloned_value.num;

        if value_address != 0 { // existing value, replace
            let mut bytes = i64_value.to_be_bytes();

            // convert to unsigned
            bytes[0] = to_unsigned(bytes[0]);

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[value_address + x] = bytes[x];
            }

        } else { // new value

            let mut be_bytes = i64_value.to_be_bytes();

            // convert to unsigned
            be_bytes[0] = to_unsigned(be_bytes[0]);

            value_address = memory.malloc_borrow(&be_bytes)?;
            c.value.set_addr_value(value_address as u16);

        }

        Ok(cursor)
    }

    fn into_value(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> Result<Option<Self>, NP_Error> {

        let c = memory.get_parsed(&cursor);

        let value_addr = c.value.get_addr_value() as usize;

        // empty value
        if value_addr == 0 {
            return Ok(None);
        }

        let exp = match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::Decimal { i: _, sortable: _, default: _, exp} => {
                exp
            },
            _ => { unsafe { unreachable_unchecked() } }
        };


        Ok(match memory.get_8_bytes(value_addr) {
            Some(x) => {
                let mut be_bytes = x.clone();
                be_bytes[0] = to_signed(be_bytes[0]);
                Some(NP_Dec::new(i64::from_be_bytes(be_bytes), exp))
            },
            None => None
        })
    }

    fn to_json(cursor: NP_Cursor_Addr, memory: &'value NP_Memory) -> NP_JSON {

        let c = memory.get_parsed(&cursor);
        let exp = match memory.schema[c.schema_addr] {
            NP_Parsed_Schema::Decimal { exp, .. } => {
                exp
            },
            _ => { unsafe { unreachable_unchecked() } }
        };


        match Self::into_value(cursor.clone(), memory) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JSMAP::new();

                        object.insert("num".to_owned(), NP_JSON::Integer(y.num));
                        object.insert("exp".to_owned(), NP_JSON::Integer(exp as i64));
                        
                        NP_JSON::Dictionary(object)
                    },
                    None => {
                        match memory.schema[c.schema_addr] {
                            NP_Parsed_Schema::Decimal { i: _, sortable: _, default, exp} => {
                                if let Some(d) = default {
                                    let mut object = JSMAP::new();

                                    object.insert("num".to_owned(), NP_JSON::Integer(d.num.clone()));
                                    object.insert("exp".to_owned(), NP_JSON::Integer(exp as i64));
                                    
                                    NP_JSON::Dictionary(object)
                                } else {
                                    NP_JSON::Null
                                }
                            },
                            _ => { unsafe { unreachable_unchecked() } }
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(cursor: NP_Cursor_Addr, memory: &NP_Memory<'value>) -> Result<usize, NP_Error> {
        
        let c = memory.get_parsed(&cursor);

        if c.value.get_addr_value() == 0 {
            Ok(0) 
        } else {
            Ok(core::mem::size_of::<i64>())
        }
    }

    fn from_json_to_schema(mut schema: Vec<NP_Parsed_Schema>, json_schema: &Box<NP_JSON>) -> Result<(bool, Vec<u8>, Vec<NP_Parsed_Schema>), NP_Error> {

        let mut schema_data: Vec<u8> = Vec::new();
        schema_data.push(NP_TypeKeys::Decimal as u8);

        let exp: u8;

        match json_schema["exp"] {
            NP_JSON::Integer(x) => {
                if x > 255 || x < 0 {
                    return Err(NP_Error::new("Decimal 'exp' property must be between 0 and 255!"))
                }
                exp = x as u8;
                schema_data.push(x as u8);
            },
            _ => {
                return Err(NP_Error::new("Decimal type requires 'exp' property!"))
            }
        }

        let mult = 10i64.pow(exp as u32);

        let default = match json_schema["default"] {
            NP_JSON::Float(x) => {
                schema_data.push(1);
                let value = x * (mult as f64);
                schema_data.extend((value as i64).to_be_bytes().to_vec());
                Some(NP_Dec::new(value as i64, exp))
            },
            NP_JSON::Integer(x) => {
                schema_data.push(1);
                let value = x * (mult as i64);
                schema_data.extend((value as i64).to_be_bytes().to_vec());
                Some(NP_Dec::new(value as i64, exp))
            },
            _ => {
                schema_data.push(0);
                // schema_data.extend(0i64.to_be_bytes().to_vec())
                None
            }
        };

        schema.push(NP_Parsed_Schema::Decimal {
            i: NP_TypeKeys::Decimal,
            default,
            sortable: true,
            exp: exp
        });

        return Ok((true, schema_data, schema))
   
    }

    fn from_bytes_to_schema(mut schema: Vec<NP_Parsed_Schema>, address: usize, bytes: &Vec<u8>) -> (bool, Vec<NP_Parsed_Schema>) {
        let exp = bytes[address + 1];

        let default = if bytes[address + 2] == 0 {
            None
        } else {
            let mut slice = 0i64.to_be_bytes();
            slice.copy_from_slice(&bytes[(address + 3)..address + 11]);
            let value = i64::from_be_bytes(slice);
            Some(NP_Dec::new(value, exp))
        };

        schema.push(NP_Parsed_Schema::Decimal {
            i: NP_TypeKeys::Decimal,
            exp: exp,
            default,
            sortable: true
        });

        (true, schema)
    }
}

#[test]
fn schema_parsing_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"decimal\",\"exp\":3,\"default\":203.293}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());

    let schema = "{\"type\":\"decimal\",\"exp\":3}";
    let factory = crate::NP_Factory::new(schema)?;
    assert_eq!(schema, factory.schema.to_json()?.stringify());
    
    Ok(())
}

#[test]
fn default_value_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"decimal\",\"exp\":3,\"default\":203.293}";
    let factory = crate::NP_Factory::new(schema)?;
    let buffer = factory.empty_buffer(None);
    assert_eq!(buffer.get::<NP_Dec>(&[])?.unwrap(), NP_Dec::new(203293, 3));

    Ok(())
}


#[test]
fn set_clear_value_and_compaction_works() -> Result<(), NP_Error> {
    let schema = "{\"type\":\"decimal\",\"exp\": 3}";
    let factory = crate::NP_Factory::new(schema)?;
    let mut buffer = factory.empty_buffer(None);
    buffer.set(&[], NP_Dec::new(203293, 3))?;
    assert_eq!(buffer.get::<NP_Dec>(&[])?.unwrap(), NP_Dec::new(203293, 3));
    buffer.del(&[])?;
    assert_eq!(buffer.get::<NP_Dec>(&[])?, None);

    buffer.compact(None)?;
    assert_eq!(buffer.calc_bytes()?.current_buffer, 2usize);

    Ok(())
}