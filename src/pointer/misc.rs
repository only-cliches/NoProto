use crate::schema::NP_Schema_Ptr;
use alloc::vec::Vec;
use crate::utils::to_signed;
use crate::utils::to_unsigned;
use crate::utils::to_base32;
use crate::json_flex::{JSMAP, NP_JSON};
use crate::schema::{NP_Schema, NP_TypeKeys};
use crate::pointer::NP_PtrKinds;
use crate::{pointer::NP_Value, error::NP_Error, utils::{Rand, to_hex}};
use core::fmt;
use core::convert::TryInto;

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::{string::ToString};
use super::NP_Lite_Ptr;

/// Represents a fixed point decimal number.
/// 
/// Allows floating point values to be stored without rounding errors, useful for storing financial data.
/// 
/// Do NOT perform calculations with `.to_float()` method, you'll make using this kind of moot.
/// 
/// NP_Dec values contain two parts:
///     1. The actual number value (`num`)
///     2. The position of the decimal point from the right (`exp`)
/// 
/// A value of "2039.756" could be stored as `NP_Dec {num: 2039756, exp: 3}`.  It could also be stored as: `NP_Dec {num: 203975600, exp: 5}`.
/// 
/// The range of possible floating point values depends on the `exp` value.  The `num` property is an i64 variable so it can safely store 9.22e18 to -9.22e18.  
/// 
/// If `exp` is zero, all values stored are whole numbers.
/// 
/// For every increase in `exp` by 1, the maximum range of possible values decreases by a power of 10.  For example at `exp = 1` the range drops to 9.22e17 to -9.22e17. 
/// However, each increase in `exp` provides a decimal point of precision.  In another example, at `exp = 5` you have 5 decimal points of precision and a max range of 9.22e13 to -9.22e13.
/// 
/// Essentially, increaseing the `exp` factor decreases the maximum range of possible values that can be stored in exchange for increased decimal precision.
/// 
/// `NP_Dec` values can safely be multiplied, added, devided, subtracted or compared with eachother.  It's a good idea to manually shift the `exp` values of two `NP_Dec` to match before performing any operation between them, otherwise the operation might not do what you expect.
/// 
/// When `NP_Dec` values are pulled out of a buffer, the `num` property is pulled from the buffer contents and the `exp` property comes from the schema.
/// 
/// ```
/// use no_proto::pointer::misc::NP_Dec;
/// 
/// // Creating a new NP_Dec for 20.49
/// let mut dec = NP_Dec::new(2049, 2);
/// 
/// // add 2
/// dec += NP_Dec::new(200, 2);
/// 
/// // add 0.03
/// dec += NP_Dec::new(3, 2);
/// 
/// // convert float then use it to minus 5
/// let mut f: NP_Dec = 5.0_f64.into();
/// f.shift_exp(2); // set new NP_Dec to `exp` of 2.
/// dec -= f; // subtract
/// 
/// assert_eq!(dec.to_float(), 17.52_f64);
/// 
/// ```
#[derive(Clone, Copy, Debug)]
pub struct NP_Dec {
    /// The number being stored, does not include decimal point data
    pub num: i64,
    /// The exponent of this number
    pub exp: u8
}

/// Schema state for NP_Dec
#[derive(Clone, Copy, Debug)]
pub struct NP_Dec_Schema_State {
    /// The expontent of this decimal
    pub exp: u8,
    /// The default value for this decimal
    pub default: Option<i64>
}

impl NP_Dec {
    /// Convert an NP_Dec into a native floating point value.
    /// 
    /// DO NOT use this to perform calculations, only to export/display the value.
    /// 
    /// ```
    /// use no_proto::pointer::misc::NP_Dec;
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

    /// Get the schema data for this type
    pub fn get_schema_state(schema_ptr: &NP_Schema_Ptr) -> NP_Dec_Schema_State {

        let exp = schema_ptr.schema.bytes[schema_ptr.address + 1];

        let default = if schema_ptr.schema.bytes[schema_ptr.address + 2] == 0 {
            None
        } else {
            let mut bytes = 0i64.to_be_bytes();
            bytes.copy_from_slice(&schema_ptr.schema.bytes[(schema_ptr.address + 3)..schema_ptr.address + 11]);
            let value = i64::from_be_bytes(bytes);
            Some(value)
        };

        return NP_Dec_Schema_State { exp: exp, default: default }
    }

    /// Shift the exponent of this NP_Dec to a new value.
    /// 
    /// If the new `exp` value is higher than the old `exp` value, there may be an overflow of the i64 value.
    /// 
    /// If the new `exp` value is lower than the old one, information will likely be lost as decimal precision is being removed from the number.
    /// 
    /// ```
    /// use no_proto::pointer::misc::NP_Dec;
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
    /// use no_proto::pointer::misc::NP_Dec;
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
    /// use no_proto::pointer::misc::NP_Dec;
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
    /// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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
/// use no_proto::pointer::misc::NP_Dec;
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


impl NP_Value for NP_Dec {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Decimal as u8, "decimal".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Decimal as u8, "decimal".to_owned()) }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = NP_Dec::get_schema_state(&schema_ptr);

        schema_json.insert("exp".to_owned(), NP_JSON::Integer(schema_state.exp.into()));
    
        if let Some(default) = schema_state.default {
            let value = NP_Dec::new(default, schema_state.exp);
            schema_json.insert("default".to_owned(), NP_JSON::Float(value.into()));
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        let schema_data = NP_Dec::get_schema_state(&schema);

        match schema_data.default {
            Some(x) => {
                Some(Box::new(NP_Dec::new(x, schema_data.exp)))
            },
            None => None
        }
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let schema_data = NP_Dec::get_schema_state(&ptr.schema);

        let mut addr = ptr.kind.get_value_addr();

        let mut cloned_value = (*value).clone();
        cloned_value.shift_exp(schema_data.exp);

        let i64_value = cloned_value.num;

        if addr != 0 { // existing value, replace
            let mut bytes = i64_value.to_be_bytes();

            // convert to unsigned
            bytes[0] = to_unsigned(bytes[0]);

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);
        } else { // new value

            let mut be_bytes = i64_value.to_be_bytes();

            // convert to unsigned
            be_bytes[0] = to_unsigned(be_bytes[0]);

            addr = ptr.memory.malloc(be_bytes.to_vec())?;
            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let schema_data = NP_Dec::get_schema_state(&ptr.schema);

        let memory = ptr.memory;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                let mut be_bytes = x.clone();
                be_bytes[0] = to_signed(be_bytes[0]);
                Some(Box::new(NP_Dec::new(i64::from_be_bytes(be_bytes), schema_data.exp)))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_value = Self::into_value(ptr.clone());

        let schema_data = NP_Dec::get_schema_state(&ptr.schema);

        match this_value {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JSMAP::new();

                        object.insert("num".to_owned(), NP_JSON::Integer(y.num));
                        object.insert("exp".to_owned(), NP_JSON::Integer(schema_data.exp as i64));
                        
                        NP_JSON::Dictionary(object)
                    },
                    None => {
                        match schema_data.default {
                            Some(x) => {
                                let mut object = JSMAP::new();

                                object.insert("num".to_owned(), NP_JSON::Integer(x));
                                object.insert("exp".to_owned(), NP_JSON::Integer(schema_data.exp as i64));
                                
                                NP_JSON::Dictionary(object)
                            },
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<i64>() as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "decimal" == type_str || "dec" == type_str {
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

            match json_schema["default"] {
                NP_JSON::Float(x) => {
                    schema_data.push(1);
                    let value = x * (mult as f64);
                    schema_data.extend((value as i64).to_be_bytes().to_vec())
                },
                NP_JSON::Integer(x) => {
                    schema_data.push(1);
                    let value = x * (mult as i64);
                    schema_data.extend((value as i64).to_be_bytes().to_vec())
                },
                _ => {
                    schema_data.push(0);
                    // schema_data.extend(0i64.to_be_bytes().to_vec())
                }
            }


            return Ok(Some(schema_data))
        }

        Ok(None)
    }
}

/// Allows you to efficiently retrieve just the bytes of the geographic coordinate
#[derive(Debug)]
pub struct NP_Geo_Bytes {
    /// Size of this coordinate: 4, 8 or 16
    pub size: u8,
    /// latitude bytes
    pub lat: Vec<u8>,
    /// longitude bytes
    pub lng: Vec<u8>
}

impl NP_Geo_Bytes {
    /// Get the actual geographic coordinate for these bytes
    pub fn into_geo(self) -> Option<NP_Geo> {
        match self.size {
            16 => {
         
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 8]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 8]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i64::from_be_bytes(bytes_lat) as f64;
                let lon = i64::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(16);

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 16})
            },
            8 => {
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 4]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 4]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(8);

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 8})
            },
            4 => {
                let mut bytes_lat = self.lat.as_slice().try_into().unwrap_or([0; 2]);
                let mut bytes_lon = self.lng.as_slice().try_into().unwrap_or([0; 2]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i16::from_be_bytes(bytes_lat) as f64;
                let lon = i16::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(4);

                Some(NP_Geo { lat: lat / dev, lng: lon / dev, size: 4})
            },
            _ => {
                None
            }
        }
    }
}

impl Default for NP_Geo_Bytes {
    fn default() -> Self { 
        NP_Geo_Bytes { lat: Vec::new(), lng: Vec::new(), size: 0 }
     }
}

impl NP_Value for NP_Geo_Bytes {
    fn schema_default(_schema: &NP_Schema_Ptr) -> Option<Box<Self>> {
        None
    }
    fn type_idx() -> (u8, String) { NP_Geo::type_idx() }
    fn self_type_idx(&self) -> (u8, String) { NP_Geo::type_idx() }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> { NP_Geo::schema_to_json(schema_ptr)}

    fn set_value(_ptr: NP_Lite_Ptr, _value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {
        Err(NP_Error::new("Can't set value with NP_Geo_Bytes, use NP_Geo instead!"))
    }
    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        NP_Geo::to_json(ptr)
    }
    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        NP_Geo::get_size(ptr)
    }
    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {

        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let state = NP_Geo::get_schema_state(&ptr.schema);

        Ok(Some(Box::new(match state.size {
            16 => {
                let bytes_lat: [u8; 8] = *ptr.memory.get_8_bytes(addr).unwrap_or(&[0; 8]);
                let bytes_lon: [u8; 8] = *ptr.memory.get_8_bytes(addr + 8).unwrap_or(&[0; 8]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 16 }
            },
            8 => {
                let bytes_lat: [u8; 4] = *ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
                let bytes_lon: [u8; 4] = *ptr.memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 8 }
            },
            4 => {
                let bytes_lat: [u8; 2] = *ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2]);
                let bytes_lon: [u8; 2] = *ptr.memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2]);

                NP_Geo_Bytes { lat: bytes_lat.to_vec(), lng: bytes_lon.to_vec(), size: 4 }
            },
            _ => {
                unreachable!();
            }
        })))
    }
}


/// Represents a Geographic Coordinate (lat / lon)
/// 
/// When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
#[derive(Debug)]
pub struct NP_Geo {
    /// The size of this geographic coordinate.  4, 8 or 16
    pub size: u8,
    /// The latitude of this coordinate
    pub lat: f64,
    /// The longitude of this coordinate
    pub lng: f64
}

/// Schema state for NP_Geo
#[derive(Debug)]
pub struct NP_Geo_Schema_State {
    /// 0 for dynamic size, anything greater than 0 is for fixed size
    pub size: u8,
    /// The default bytes
    pub default: Option<NP_Geo>
}

impl NP_Geo {

    /// Create a new NP_Geo value, make sure the size matches the schema
    pub fn new(size: u8, lat: f64, lng: f64) -> Self {
        NP_Geo { size, lat, lng}
    }

    /// Get the schema state struct from the schema bytes
    /// 
    pub fn get_schema_state(schema_ptr: &NP_Schema_Ptr) -> NP_Geo_Schema_State {
        let size = schema_ptr.schema.bytes[schema_ptr.address + 1];

        if schema_ptr.schema.bytes[schema_ptr.address + 2] == 0 {
            return NP_Geo_Schema_State {
                size: size,
                default: None
            }
        }

        match size {
            4 => {
                let lat = &schema_ptr.schema.bytes[(schema_ptr.address + 3)..(schema_ptr.address + 5)];
                let lng = &schema_ptr.schema.bytes[(schema_ptr.address + 6)..(schema_ptr.address + 8)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Geo_Schema_State {
                    size: size,
                    default: Some(default_value.into_geo().unwrap())
                }
            },
            8 => {
                let lat = &schema_ptr.schema.bytes[(schema_ptr.address + 3)..(schema_ptr.address + 7)];
                let lng = &schema_ptr.schema.bytes[(schema_ptr.address + 7)..(schema_ptr.address + 11)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Geo_Schema_State {
                    size: size,
                    default: Some(default_value.into_geo().unwrap())
                }
            },
            16 => {
                let lat = &schema_ptr.schema.bytes[(schema_ptr.address + 3)..(schema_ptr.address + 11)];
                let lng = &schema_ptr.schema.bytes[(schema_ptr.address + 12)..(schema_ptr.address + 20)];
                let default_value = NP_Geo_Bytes { size: size, lat: lat.to_vec(), lng: lng.to_vec()};
                NP_Geo_Schema_State {
                    size: size,
                    default: Some(default_value.into_geo().unwrap())
                }
            },
            _ => {
                unreachable!();
            }
        }
    }

    /// Get the deviser value depending on the resolution of the type in the schema
    pub fn get_deviser(size: i64) -> f64 {
        match size {
            16 => 1000000000f64,
            8 =>  10000000f64,
            4 =>  100f64,
            _ => 0.0
        }
     }

     /// Get the bytes that represent this geographic coordinate
     pub fn get_bytes(&self) -> Option<NP_Geo_Bytes> {
        if self.size == 0 {
            return None
        }

        let dev = NP_Geo::get_deviser(self.size as i64);

        match self.size {
            16 => {

                let mut lat_bytes = ((self.lat * dev) as i64).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i64).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            8 => {

                let mut lat_bytes = ((self.lat * dev) as i32).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i32).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            4 => {

                let mut lat_bytes = ((self.lat * dev) as i16).to_be_bytes();
                let mut lon_bytes = ((self.lng * dev) as i16).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                Some(NP_Geo_Bytes { lat: lat_bytes.to_vec(), lng: lon_bytes.to_vec(), size: self.size })
            },
            _ => {
                None
            }
        }
     }
}

impl Default for NP_Geo {
    fn default() -> Self { 
        NP_Geo { lat: 0.0, lng: 0.0, size: 0 }
     }
}

fn geo_default_value(size: u8, json: &NP_JSON) -> Result<Option<NP_Geo_Bytes>, NP_Error> {
    match &json["default"] {
        NP_JSON::Dictionary(x) => {
            let mut lat = 0f64;
            match x.get("lat") {
                Some(x) => {
                    match x {
                        NP_JSON::Integer(y) => {
                            lat = *y as f64;
                        },
                        NP_JSON::Float(y) => {
                            lat = *y as f64;
                        },
                        _ => {}
                    }
                },
                None => {
                    return Err(NP_Error::new("Default values for NP_Geo should have lat key!"))
                }
            };
            let mut lng = 0f64;
            match x.get("lng") {
                Some(x) => {
                    match x {
                        NP_JSON::Integer(y) => {
                            lng = *y as f64;
                        },
                        NP_JSON::Float(y) => {
                            lng = *y as f64;
                        },
                        _ => {}
                    }
                },
                None => {
                    return Err(NP_Error::new("Default values for NP_Geo should have lng key!"))
                }
            };

            match NP_Geo::new(size, lat, lng).get_bytes() {
                Some(b) => return Ok(Some(b)),
                None => return Ok(None)
            }
        },
        _ => return Ok(None)
    }
}

impl NP_Value for NP_Geo {

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {
        let schema_state = Self::get_schema_state(schema);

        match schema_state.default {
            Some(x) => {
                Some(Box::new(x))
            },
            None => None
        }
    }

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Geo as u8, "geo".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Geo as u8, "geo".to_owned()) }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
       
        let schema_state = Self::get_schema_state(&schema_ptr);

        let mut type_str = Self::type_idx().1;
        type_str.push_str(schema_state.size.to_string().as_str());
        schema_json.insert("type".to_owned(), NP_JSON::String(type_str));
    
        if let Some(default) = schema_state.default {
            let mut default_map = JSMAP::new();
            default_map.insert("lat".to_owned(), NP_JSON::Float(default.lat));
            default_map.insert("lng".to_owned(), NP_JSON::Float(default.lng));
            schema_json.insert("default".to_owned(), NP_JSON::Dictionary(default_map));
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let schema_state = Self::get_schema_state(&ptr.schema);

        let value_bytes_size = schema_state.size as usize;

        if value_bytes_size == 0 {
            unreachable!();
        }

        let write_bytes = ptr.memory.write_bytes();

        let half_value_bytes = value_bytes_size / 2;

        // convert input values into bytes
        let value_bytes = match schema_state.size {
            16 => {
                let dev = NP_Geo::get_deviser(16);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i64).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i64).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                for x in 0..value_bytes_size {
                    if x < half_value_bytes {
                        v_bytes[x] = lat_bytes[x];
                    } else {
                        v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                    }
                }
                v_bytes
            },
            8 => {
                let dev = NP_Geo::get_deviser(8);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i32).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i32).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                for x in 0..value_bytes_size {
                    if x < half_value_bytes {
                        v_bytes[x] = lat_bytes[x];
                    } else {
                        v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                    }
                }
                v_bytes
            },
            4 => {
                let dev = NP_Geo::get_deviser(4);

                let mut v_bytes: [u8; 16] = [0; 16];
                let mut lat_bytes = ((value.lat * dev) as i16).to_be_bytes();
                let mut lon_bytes = ((value.lng * dev) as i16).to_be_bytes();

                // convert to unsigned bytes
                lat_bytes[0] = to_unsigned(lat_bytes[0]);
                lon_bytes[0] = to_unsigned(lon_bytes[0]);

                for x in 0..value_bytes_size {
                    if x < half_value_bytes {
                        v_bytes[x] = lat_bytes[x];
                    } else {
                        v_bytes[x] = lon_bytes[x - half_value_bytes]; 
                    }
                }
                v_bytes
            },
            _ => {
                [0; 16]
            }
        };

        if addr != 0 { // existing value, replace

            // overwrite existing values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                }
            }

            return Ok(ptr.kind);

        } else { // new value

            addr = match schema_state.size {
                16 => { ptr.memory.malloc([0; 16].to_vec())? },
                8 => { ptr.memory.malloc([0; 8].to_vec())? },
                4 => { ptr.memory.malloc([0; 4].to_vec())? },
                _ => { 0 }
            };

            // set values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                }
            }

            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {

        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let schema_state = Self::get_schema_state(&ptr.schema);

        Ok(Some(Box::new(match schema_state.size {
            16 => {
         
                let mut bytes_lat: [u8; 8] = *ptr.memory.get_8_bytes(addr).unwrap_or(&[0; 8]);
                let mut bytes_lon: [u8; 8] = *ptr.memory.get_8_bytes(addr + 8).unwrap_or(&[0; 8]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i64::from_be_bytes(bytes_lat) as f64;
                let lon = i64::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(16);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 16}
            },
            8 => {
                let mut bytes_lat: [u8; 4] = *ptr.memory.get_4_bytes(addr).unwrap_or(&[0; 4]);
                let mut bytes_lon: [u8; 4] = *ptr.memory.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(8);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 8}
            },
            4 => {
                let mut bytes_lat: [u8; 2] = *ptr.memory.get_2_bytes(addr).unwrap_or(&[0; 2]);
                let mut bytes_lon: [u8; 2] = *ptr.memory.get_2_bytes(addr + 2).unwrap_or(&[0; 2]);

                // convert to signed bytes
                bytes_lat[0] = to_signed(bytes_lat[0]); 
                bytes_lon[0] = to_signed(bytes_lon[0]); 

                let lat = i16::from_be_bytes(bytes_lat) as f64;
                let lon = i16::from_be_bytes(bytes_lon) as f64;

                let dev = NP_Geo::get_deviser(4);

                NP_Geo { lat: lat / dev, lng: lon / dev, size: 4}
            },
            _ => {
                unreachable!();
            }
        })))
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_value = Self::into_value(ptr.clone());

        match this_value {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JSMAP::new();

                        object.insert("lat".to_owned(), NP_JSON::Float(y.lat));
                        object.insert("lng".to_owned(), NP_JSON::Float(y.lng));
                        
                        NP_JSON::Dictionary(object)
                    },
                    None => {
                        let schema_state = Self::get_schema_state(&ptr.schema);

                        match schema_state.default {
                            Some(k) => {
                                let mut object = JSMAP::new();

                                object.insert("lat".to_owned(), NP_JSON::Float(k.lat));
                                object.insert("lng".to_owned(), NP_JSON::Float(k.lng));
                                
                                NP_JSON::Dictionary(object)
                            },
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            let schema_state = Self::get_schema_state(&ptr.schema);
            Ok(schema_state.size as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        match type_str.as_str() {
            "geo4" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(4);
                match geo_default_value(4, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat);
                        schema_data.extend(x.lng);
                    },
                    None => {
                        schema_data.push(0);
                    }
                }
                Ok(Some(schema_data))
            },
            "geo8" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(8);
                match geo_default_value(8, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat);
                        schema_data.extend(x.lng);
                    },
                    None => {
                        schema_data.push(0);
                    }
                }
                Ok(Some(schema_data))
            },
            "geo16" => {
                let mut schema_data: Vec<u8> = Vec::new();
                schema_data.push(NP_TypeKeys::Geo as u8);
                schema_data.push(16);
                match geo_default_value(16, json_schema)? {
                    Some(x) => {
                        schema_data.push(1);
                        schema_data.extend(x.lat);
                        schema_data.extend(x.lng);
                    },
                    None => {
                        schema_data.push(0);
                    }
                }
                Ok(Some(schema_data))
            },
            _ => {
                Ok(None)
            }
        }
    }
}

/// Represents a ULID type which has a 6 byte timestamp and 10 bytes of randomness
/// 
/// Useful for storing time stamp data that doesn't have collisions.
pub struct NP_ULID {
    /// The unix timestamp in milliseconds for this ULID
    pub time: u64,
    /// The random bytes for this ULID
    pub id: u128
}

impl NP_ULID {

    /// Creates a new ULID from the timestamp and provided seed.
    /// 
    /// The random seed is used to generate the ID, the same seed will always lead to the same random bytes so try to use something actually random for the seed.
    /// 
    /// The time should be passed in as the unix epoch in milliseconds.
    pub fn generate(now_ms: u64, random_seed: u32) -> NP_ULID {
        let mut rng = Rand::new(random_seed);

        let mut id: [u8; 16] = [0; 16];

        for x in 6..id.len() {
            id[x] = rng.gen_range(0, 255) as u8;
        }

        NP_ULID {
            time: now_ms,
            id: u128::from_be_bytes(id)
        }
    }

    /// Generates a ULID with the given time and a provided random number generator.
    /// This is the preferrable way to generate a ULID, if you can provide a better RNG function than the psudorandom one built into this library, you should.
    /// 
    pub fn generate_with_rand<F>(now_ms: u64, random_fn: F) -> NP_ULID where F: Fn(u8, u8) -> u8 {

        let mut id: [u8; 16] = [0; 16];

        for x in 6..id.len() {
            id[x] = random_fn(0, 255);
        }

        NP_ULID {
            time: now_ms,
            id: u128::from_be_bytes(id)
        }
    }

    /// Generates a stringified version of this ULID with base32.
    /// 
    pub fn to_string(&self) -> String {
        let mut result: String = "".to_owned();

        result.push_str(to_base32(self.time as u128, 10).as_str());
        result.push_str(to_base32(self.id, 16).as_str());

        result
    }
}


impl Default for NP_ULID {
    fn default() -> Self { 
        NP_ULID { id: 0, time: 0 }
     }
}

impl fmt::Debug for NP_ULID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl NP_Value for NP_ULID {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Ulid as u8, "ulid".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Ulid as u8, "ulid".to_owned()) }

    fn schema_to_json(_schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let time_bytes: [u8; 8] = value.time.to_be_bytes();
        let id_bytes: [u8; 16] = value.id.to_be_bytes();

        if addr != 0 { // existing value, replace

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..16 {
                if x < 6 {
                    write_bytes[(addr + x as u32) as usize] = time_bytes[x as usize + 2];
                } else {
                    write_bytes[(addr + x as u32) as usize] = id_bytes[x as usize];
                }
            }

            return Ok(ptr.kind);

        } else { // new value

            let mut bytes: [u8; 16] = [0; 16];

            for x in 0..bytes.len() {
                if x < 6 {
                    bytes[(addr + x as u32) as usize] = time_bytes[x as usize + 2];
                } else {
                    bytes[(addr + x as u32) as usize] = id_bytes[x as usize];
                }
            }

            addr = ptr.memory.malloc(bytes.to_vec())?;

            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }                    
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let mut time_bytes: [u8; 8] = [0; 8];
        let mut id_bytes: [u8; 16] = [0; 16];

        let read_bytes: [u8; 16] = *ptr.memory.get_16_bytes(addr).unwrap_or(&[0; 16]);

        for x in 0..read_bytes.len() {
            if x < 6 {
                time_bytes[x + 2] = read_bytes[x];
            } else {
                id_bytes[x] = read_bytes[x];
            }
        }

        Ok(Some(Box::new(NP_ULID {
            time: u64::from_be_bytes(time_bytes),
            id: u128::from_be_bytes(id_bytes)
        })))
         
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(y.to_string())
                    },
                    None => {
                        NP_JSON::Null
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(16)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "ulid" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Ulid as u8);
            return Ok(Some(schema_data));
        }
        
        Ok(None)
    }

}

/// Represents a V4 UUID, good for globally unique identifiers
/// 
/// `uuid` types are always represented with this struct.
pub struct NP_UUID {
    /// The random bytes for this UUID
    pub value: [u8; 16]
}

impl NP_UUID {

    /// Generate a new UUID with a given random seed.  You should attempt to provide a seed with as much randomness as possible.
    /// 
    pub fn generate(random_seed: u32) -> NP_UUID {


        let mut uuid = NP_UUID {
            value: [0; 16]
        };

        let mut rng = Rand::new(random_seed);

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + rng.gen_range(0, 15) as u8;
            } else {
                uuid.value[x] = rng.gen_range(0, 255) as u8;
            }
        }

        uuid
    }

    /// Generates a UUID with a provided random number generator.
    /// This is the preferrable way to generate a ULID, if you can provide a better RNG function than the psudorandom one built into this library, you should.
    /// 
    pub fn generate_with_rand<F>(random_fn: F) -> NP_UUID where F: Fn(u8, u8) -> u8 {
        let mut uuid = NP_UUID {
            value: [0; 16]
        };

        for x in 0..uuid.value.len() {
            if x == 6 {
                uuid.value[x] = 64 + random_fn(0, 15) as u8;
            } else {
                uuid.value[x] = random_fn(0, 255) as u8;
            }
        }

        uuid
    }

    /// Generates a stringified version of the UUID.
    /// 
    pub fn to_string(&self) -> String {

        let mut result: String = "".to_owned();

        for x in 0..self.value.len() {
            if x == 4 || x == 6 || x == 8 || x == 10 {
                result.push_str("-");
            }
            let value = self.value[x] as u64;
            result.push_str(to_hex(value, 2).as_str());
        }

        result
    }
}

impl fmt::Debug for NP_UUID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Default for NP_UUID {
    fn default() -> Self { 
        NP_UUID { value: [0; 16] }
     }
}

impl NP_Value for NP_UUID {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Uuid as u8, "uuid".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Uuid as u8, "uuid".to_owned()) }

    fn schema_to_json(_schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = value.value;
            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);

        } else { // new value

            let bytes = value.value;
            addr = ptr.memory.malloc(bytes.to_vec())?;

            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }                    
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = ptr.memory;
        Ok(match memory.get_16_bytes(addr) {
            Some(x) => {
                // copy since we're handing owned value outside the library
                let mut bytes: [u8; 16] = [0; 16];
                bytes.copy_from_slice(x);
                Some(Box::new(NP_UUID { value: bytes}))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::String(y.to_string())
                    },
                    None => {
                        NP_JSON::Null
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(16)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "uuid" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Uuid as u8);
            return Ok(Some(schema_data));
        }
        
        Ok(None)
    }

}

/// Represents the string value of a choice in a schema
#[derive(Clone, Debug)]
pub struct NP_Option {
    /// The value of this option type
    pub value: Option<String>
}

impl NP_Option {
    /// Create a new option type with the given string
    pub fn new<S: AsRef<str>>(value: S) -> NP_Option {
        NP_Option {
            value: Some(value.as_ref().to_string())
        }
    }

    /// Create a new empty option type
    pub fn empty() -> NP_Option {
        NP_Option {
            value: None
        }
    }
    
    /// Set the value of this option type
    pub fn set(&mut self, value: Option<String>) {
        self.value = value;
    }

    /// Get schema state
    pub fn get_schema_state(schema_ptr: &NP_Schema_Ptr) -> NP_Option_Schema_State {
        let mut default_index: Option<u8> = None;

        if schema_ptr.schema.bytes[schema_ptr.address + 1] > 0 {
            default_index = Some(schema_ptr.schema.bytes[schema_ptr.address + 1] - 1);
        }

        let choices_len = schema_ptr.schema.bytes[schema_ptr.address + 2];

        let mut choices: Vec<String> = Vec::new();
        let mut offset: usize = schema_ptr.address + 3;
        for _x in 0..choices_len {
            let choice_size = schema_ptr.schema.bytes[offset] as usize;
            let choice_bytes = &schema_ptr.schema.bytes[(offset + 1)..(offset + 1 + choice_size)];
            choices.push(String::from_utf8_lossy(choice_bytes).into());
            offset += 1 + choice_size;
        }

        NP_Option_Schema_State {
            default: default_index,
            choices: choices
        }
    }
}

impl Default for NP_Option {
    fn default() -> Self { 
        NP_Option { value: None }
     }
}

/// The schema state for NP_Option type
#[derive(Clone, Debug)]
pub struct NP_Option_Schema_State {
    /// Default option index
    pub default: Option<u8>,
    /// All choices for this option type
    pub choices: Vec<String>
}


impl NP_Value for NP_Option {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Enum as u8, "option".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Enum as u8, "option".to_owned()) }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = Self::get_schema_state(&schema_ptr);

        let options: Vec<NP_JSON> = schema_state.choices.into_iter().map(|value| {
            NP_JSON::String(value)
        }).collect();
    
        if let Some(default) = schema_state.default {
            schema_json.insert("default".to_owned(), options[default as usize].clone());
        }

        schema_json.insert("choices".to_owned(), NP_JSON::Array(options));

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        let schema_state = Self::get_schema_state(&schema);

        if let Some(idx) = schema_state.default {
            Some(Box::new(NP_Option { value: Some(schema_state.choices[idx as usize].clone()) }))
        } else {
            None
        }
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        let schema_state = Self::get_schema_state(&ptr.schema);

        let mut value_num: i32 = -1;

        {
            let mut ct: u16 = 0;

            for opt in schema_state.choices {
                if value.value == Some(opt.to_string()) {
                    value_num = ct as i32;
                }
                ct += 1;
            };

            if value_num == -1 {
                return Err(NP_Error::new("Option not found, cannot set uknown option!"));
            }
        }

        let bytes = (value_num as u8).to_be_bytes();

        if addr != 0 { // existing value, replace

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            return Ok(ptr.kind);

        } else { // new value

            addr = ptr.memory.malloc(bytes.to_vec())?;

            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }                    
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = ptr.memory;

        let schema_state = NP_Option::get_schema_state(&ptr.schema);

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                let value_num = u8::from_be_bytes([x]) as usize;

                if value_num > schema_state.choices.len() {
                    None
                } else {
                    Some(Box::new(NP_Option { value: Some(schema_state.choices[value_num].clone()) }))
                }
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        match y.value {
                            Some(str_value) => {
                                NP_JSON::String(str_value)
                            },
                            None => {
                                let schema_state = NP_Option::get_schema_state(&ptr.schema);
                                if let Some(x) = schema_state.default {
                                    NP_JSON::String(schema_state.choices[x as usize].clone())
                                } else {
                                    NP_JSON::Null
                                }
                            }
                        }
                    },
                    None => {
                        let schema_state = NP_Option::get_schema_state(&ptr.schema);
                        if let Some(x) = schema_state.default {
                            NP_JSON::String(schema_state.choices[x as usize].clone())
                        } else {
                            NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>() as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "option" == type_str || "enum" == type_str {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Enum as u8);

            let mut choices: Vec<String> = Vec::new();

            let mut default_stir: Option<String> = None;

            match &json_schema["default"] {
                NP_JSON::String(def) => {
                    default_stir = Some(def.clone());
                },
                _ => {}
            }

            let mut default_index: Option<u8> = None;

            match &json_schema["choices"] {
                NP_JSON::Array(x) => {
                    for opt in x {
                        match opt {
                            NP_JSON::String(stir) => {
                                if stir.len() > 255 {
                                    return Err(NP_Error::new("'option' choices cannot be longer than 255 characters each!"))
                                }

                                if let Some(def) = &default_stir {
                                    if def == stir {
                                        default_index = Some(choices.len() as u8);
                                    }
                                }
                                choices.push(stir.clone());
                            },
                            _ => {}
                        }
                    }
                },
                _ => {
                    return Err(NP_Error::new("'option' type requires a 'choices' key with an array of strings!"))
                }
            }

            if choices.len() > 254 {
                return Err(NP_Error::new("'option' type cannot have more than 254 choices!"))
            }

            // default value
            match default_index {
                Some(x) => schema_data.push(x + 1),
                None => schema_data.push(0)
            }

            // choices
            schema_data.push(choices.len() as u8);
            for choice in choices {
                schema_data.push(choice.len() as u8);
                schema_data.extend(choice.as_bytes().to_vec())
            }

            return Ok(Some(schema_data));
        }
        
        Ok(None)
    }

}

fn bool_get_schema_state(schema_ptr: &NP_Schema_Ptr) -> Option<bool> {

    match schema_ptr.schema.bytes[schema_ptr.address + 1] {
        0 => None,
        1 => Some(true),
        2 => Some(false),
        _ => unreachable!()
    }
}

impl NP_Value for bool {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Boolean as u8, "bool".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Boolean as u8, "bool".to_owned()) }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = bool_get_schema_state(&schema_ptr);

        if let Some(default) = schema_state {
            schema_json.insert("default".to_owned(), match default {
                true => NP_JSON::True,
                false => NP_JSON::False
            });
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        let state = bool_get_schema_state(&schema);

        match state {
            Some(x) => {
                Some(Box::new(x))
            },
            None => None
        }
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            // overwrite existing values in buffer
            ptr.memory.write_bytes()[addr as usize] = bytes[0];

            return Ok(ptr.kind);

        } else { // new value

            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            addr = ptr.memory.malloc(bytes.to_vec())?;
            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = ptr.memory;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(if x == 1 { true } else { false }))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {
        let this_string = Self::into_value(ptr.clone());

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        if *y == true {
                            NP_JSON::True
                        } else {
                            NP_JSON::False
                        }
                    },
                    None => {
                        let state = bool_get_schema_state(&ptr.schema);
                        match state {
                            Some(x) => {
                                if x == true {
                                    NP_JSON::True
                                } else {
                                    NP_JSON::False
                                }
                            },
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>() as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if type_str == "bool" || type_str == "boolean" {
            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Boolean as u8);

            match json_schema["default"] {
                NP_JSON::False => {
                    schema_data.push(2);
                },
                NP_JSON::True => {
                    schema_data.push(1);
                },
                _ => {
                    schema_data.push(0);
                }
            };

            return Ok(Some(schema_data));
        }

        Ok(None)
    }
}

/// Stores the current unix epoch in u64
#[derive(Clone, Copy)]
pub struct NP_Date {
    /// The value of the date
    pub value: u64
}

impl NP_Date {
    /// Create a new date type with the given time
    pub fn new(time: u64) -> Self {
        NP_Date { value: time }
    }

    /// Get schema state for NP_Date
    pub fn get_schema_state(schema_ptr: &NP_Schema_Ptr) -> Option<NP_Date> {

        let has_default = schema_ptr.schema.bytes[schema_ptr.address + 1];

        if has_default == 0 {
            None
        } else {
            let bytes_slice = &schema_ptr.schema.bytes[(schema_ptr.address + 2)..(schema_ptr.address + 10)];

            let mut u64_bytes = 0u64.to_be_bytes();
            u64_bytes.copy_from_slice(bytes_slice);
            Some(NP_Date { value: u64::from_be_bytes(u64_bytes)})
        }
    }
}

impl PartialEq for NP_Date {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Default for NP_Date {
    fn default() -> Self { 
        NP_Date { value: 0 }
     }
}

impl fmt::Debug for NP_Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl NP_Value for NP_Date {

    fn type_idx() -> (u8, String) { (NP_TypeKeys::Date as u8, "date".to_owned()) }
    fn self_type_idx(&self) -> (u8, String) { (NP_TypeKeys::Date as u8, "date".to_owned()) }

    fn schema_to_json(schema_ptr: NP_Schema_Ptr)-> Result<NP_JSON, NP_Error> {
        let mut schema_json = JSMAP::new();
        schema_json.insert("type".to_owned(), NP_JSON::String(Self::type_idx().1));

        let schema_state = Self::get_schema_state(&schema_ptr);
    
        if let Some(default) = schema_state {
            schema_json.insert("default".to_owned(), NP_JSON::Integer(default.value as i64));
        }

        Ok(NP_JSON::Dictionary(schema_json))
    }

    fn schema_default(schema: &NP_Schema_Ptr) -> Option<Box<Self>> {

        match NP_Date::get_schema_state(&schema) {
            Some(x) => {
                Some(Box::new(x))
            },
            None => None
        }
    }

    fn set_value(ptr: NP_Lite_Ptr, value: Box<&Self>) -> Result<NP_PtrKinds, NP_Error> {

        let mut addr = ptr.kind.get_value_addr();

        if addr != 0 { // existing value, replace
            let bytes = value.value.to_be_bytes();

            let write_bytes = ptr.memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(ptr.kind);

        } else { // new value

            let bytes = value.value.to_be_bytes();
            addr = ptr.memory.malloc(bytes.to_vec())?;
            return Ok(ptr.memory.set_value_address(ptr.location, addr as u32, &ptr.kind));
        }                    
        
    }

    fn into_value(ptr: NP_Lite_Ptr) -> Result<Option<Box<Self>>, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = ptr.memory;
        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Date { value: u64::from_be_bytes(*x) }))
            },
            None => None
        })
    }

    fn to_json(ptr: NP_Lite_Ptr) -> NP_JSON {

        match Self::into_value(ptr.clone()) {
            Ok(x) => {
                match x {
                    Some(y) => {
                        NP_JSON::Integer(y.value as i64)
                    },
                    None => {
                        match NP_Date::get_schema_state(&ptr.schema) {
                            Some(x) => NP_JSON::Integer(x.value as i64),
                            None => NP_JSON::Null
                        }
                    }
                }
            },
            Err(_e) => {
                NP_JSON::Null
            }
        }
    }

    fn get_size(ptr: NP_Lite_Ptr) -> Result<u32, NP_Error> {
        let addr = ptr.kind.get_value_addr() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u64>() as u32)
        }
    }

    fn from_json_to_schema(json_schema: &NP_JSON) -> Result<Option<Vec<u8>>, NP_Error> {

        let type_str = NP_Schema::get_type(json_schema)?;

        if "date" == type_str {

            let mut schema_data: Vec<u8> = Vec::new();
            schema_data.push(NP_TypeKeys::Date as u8);

            match json_schema["default"] {
                NP_JSON::Integer(x) => {
                    schema_data.push(1);
                    schema_data.extend((x as u64).to_be_bytes().to_vec())
                },
                _ => {
                    schema_data.push(0);
                }
            }

            return Ok(Some(schema_data));
        }

        Ok(None)
    }
}