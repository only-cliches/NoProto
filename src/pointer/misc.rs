use crate::utils::to_base32;
use crate::json_flex::{JFMap, JFObject};
use crate::pointer::NP_ValueInto;
use crate::schema::{NP_SchemaKinds, NP_Schema, NP_TypeKeys};
use crate::pointer::NP_PtrKinds;
use crate::{memory::NP_Memory, pointer::NP_Value, error::NP_Error, utils::{Rand, to_hex}};
use core::fmt;

use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::string::ToString;

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
/// A value of "2039.756" could be stored as `NP_Dec {num: 2039756, exp: 3}`.  This would also be valid and have the same end value: `NP_Dec {num: 203975600, exp: 5}`.
/// 
/// The range of possible floating point values depends on the `exp` value.  The `num` property is an i64 variable so it can safely store 9.22e18 to -9.22e18.  
/// 
/// If `exp` is zero, all values stored are whole numbers.
/// 
/// For every increase in `exp` by 1, the maximum range of possible values decreases by a power of 10.  For example at `exp = 1` the range drops to 9.22e17 to -9.22e17. 
/// However, each increase in `exp` provides a decimal point of precision.  In another example, at `exp = 5` you have 5 decimal points of precision and a max range of 9.22e13 to -9.22e13.
/// 
/// Essentially, increaseing the `exp` factor decreases the range of possible values that can be stored in exchange for increased decimal precision.
/// 
/// `NP_Dec` values can safely be multiplied, added, devided, subtracted or compared with eachother.  However the `exp` value **must be identical** between two NP_Dec values for operations or comparisons to be performed.  You should make sure the `exp` value between two `NP_Dec` is the same using `shift_exp` to match the two `exp` values.  Default behavior exists to shift the two values before operations/comparisons but may not act in a way you expect.  It's better if you manually shift the `exp` value between two NP_Dec values before performaing operations or comparison.
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
    pub num: i64,
    pub exp: u8
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
        // let s = 10f64.powf(self.exp as f64);
        m / s
    }

    /// Shift the exponent of this NP_Dec to a new value.
    /// 
    /// If the new `exp` value is lower than the old `exp` value, there may be an overflow of the i64 value.
    /// 
    /// If the new `exp value is higher than the old one, information will likely be lost as decimal precision is being removed from the number.
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

    /// Given another NP_Dec value, match the `exp` value of this NP_Dec and the other one.  Returns a copy of the other NP_Dec.
    /// 
    /// This will figure out the lowest `exp` between the two NP_Dec (self and other), and modify either NP_Dec to match the new, lowest `exp`.
    /// 
    /// This possibly **mutates** self to match the `exp` of the other NP_Dec, possible/probably data loss here.
    /// 
    /// It's better if you use `shift_exp` to manually match two NP_Dec `exp` values, and not this method.
    /// 
    /// ```
    /// use no_proto::pointer::misc::NP_Dec;
    /// 
    /// let mut my_num = NP_Dec::new(2203, 3); // value is 2.203
    /// 
    /// let other_num = NP_Dec::new(50, 1); // value is 5.0
    /// 
    /// let matched_dec = my_num.match_exp(other_num);
    /// // `exp` values match now! They're both 1.
    /// assert_eq!(matched_dec.exp, my_num.exp);
    /// 
    /// // however, we've lost two decimal points of data in `my_num` in the process
    /// assert_eq!(my_num.to_float(), 2.2f64);
    /// ```
    /// 
    pub fn match_exp(&mut self, other: NP_Dec) -> NP_Dec {
        let mut other_copy = other.clone();

        if other_copy.exp == self.exp {
            return other_copy;
        }

        let new_exp = u8::min(self.exp, other.exp);

        if new_exp != self.exp {
            self.shift_exp(new_exp);
        } else {
            other_copy.shift_exp(new_exp);
        }

        return other_copy;
    }

    /// Export NP_Dec to it's parts.
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
/// If the two `exp` values are not identical, unexpected results may occur due to rounding
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
/// ```
impl core::cmp::PartialOrd for NP_Dec {

    fn lt(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num < other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num < new_other.num;
        }
    }

    fn le(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num <= other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num <= new_other.num;
        }
    }

    fn gt(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num > other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num > new_other.num;
        }
    }

    fn ge(&self, other: &NP_Dec) -> bool {
        if self.exp == other.exp {
            return self.num >= other.num;
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            return new_self.num >= new_other.num;
        }
    }

    fn partial_cmp(&self, other: &NP_Dec) -> Option<core::cmp::Ordering> { 
        
        
        let (a, b) = if self.exp == other.exp {
            (self.num, other.num)
        } else {

            let new_exp = u8::max(self.exp, other.exp);
            let new_self = if new_exp == self.exp { *self } else { self.clone().shift_exp(new_exp) };
            let new_other = if new_exp == other.exp { *other } else { other.clone().shift_exp(new_exp) };

            (new_self.num, new_other.num)
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

/// Converts a Dec64 into an Int64, rounds to nearest whole number
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

/// Converts an Int64 into a Dec64
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

/// Converts a Dec64 into a Float64
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

fn round_f64(n: f64) -> f64 {
    let value = if n < 0.0 { n - 0.5 } else { n + 0.5 };

    let bounds_value = value.max(core::i64::MIN as f64).min(core::i64::MAX as f64);

    (bounds_value as i64) as f64
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

/// Converts an Float64 into a Dec64
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

impl core::ops::DivAssign for NP_Dec { // a /= b
    fn div_assign(&mut self, other: NP_Dec) { 
        if self.exp != other.exp {
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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
            let other_copy = self.match_exp(other);
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

    fn is_type( type_str: &str) -> bool {
        "dec64" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Dec64 as i64, "dec64".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Dec64 as i64, "dec64".to_owned()) }

    fn schema_state(_type_string: &str, _json_schema: &JFObject) -> core::result::Result<i64, NP_Error> {

        match _json_schema["exp"].into_i64() {
            Some(x) => {
                if *x > 255 || *x < 0 {
                    return Err(NP_Error::new("Dec64 'exp' property must be between 0 and 255"));
                }
                return Ok(*x);
            },
            None => {
                return Err(NP_Error::new("Dec64 types must have 'exp' property!"))
            }
        }
    }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        let i64_value = value.num;

        let offset = core::i64::MAX as i128;

        if addr != 0 { // existing value, replace
            let bytes = (((i64_value as i128) + offset) as u64).to_be_bytes();

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }
            write_bytes[(addr + 8 as u32) as usize] = value.exp;
            return Ok(*kind);
        } else { // new value

            let mut bytes: [u8; 9] = [0; 9];
            let be_bytes = (((i64_value as i128) + offset) as u64).to_be_bytes();
            for x in 0..be_bytes.len() {
                bytes[x] = be_bytes[x];
            }
            bytes[8] = value.exp;
            addr = memory.malloc(bytes.to_vec())?;
            return Ok(memory.set_value_address(address, addr as u32, kind));
        }
    }
}

impl<'a> NP_ValueInto<'a> for NP_Dec {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &'a NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;

        let offset = core::i64::MAX as i128;

        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                let value = ((u64::from_be_bytes(*x) as i128) - offset) as i64;
                Some(Box::new(NP_Dec::new(value, schema.type_state as u8)))
            },
            None => None
        })
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JFMap::<JFObject>::new();

                        object.insert("num".to_owned(), JFObject::Integer(y.num));
                        object.insert("exp".to_owned(), JFObject::Integer(schema.type_state));
                        
                        JFObject::Dictionary(object)
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(9)
        }
    }


}


/// Represents a Geographic Coordinate (lat / lon)
/// 
/// When `geo4`, `geo8`, or `geo16` types are used the data is saved and retrieved with this struct.
#[derive(Debug)]
pub struct NP_Geo {
    pub lat: f64,
    pub lon: f64
}

impl Default for NP_Geo {
    fn default() -> Self { 
        NP_Geo { lat: 0.0, lon: 0.0 }
     }
}

impl NP_Value for NP_Geo {

    fn is_type( type_str: &str) -> bool {
        "geo4" == type_str || "geo8" == type_str || "geo16" == type_str 
    }

    fn schema_state(type_string: &str, _json_schema: &JFObject) -> core::result::Result<i64, NP_Error> {
        Ok(match type_string {
            "geo4" => 4,
            "geo8" => 8,
            "geo16" => 16,
            _ => 0
        })
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Geo as i64, "geo".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Geo as i64, "geo".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        let value_bytes_size = schema.type_state as usize;

        if value_bytes_size == 0 {
            unreachable!();
        }

        let write_bytes = memory.write_bytes();

        let half_value_bytes = value_bytes_size / 2;

        // convert input values into bytes
        let value_bytes = match schema.type_state {
            16 => {
                let dev = 1000000000f64;
                let mut v_bytes: [u8; 16] = [0; 16];
                let lat_bytes = ((value.lat * dev) as i64).to_be_bytes();
                let lon_bytes = ((value.lon * dev) as i64).to_be_bytes();

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
                let dev = 10000000f64;

                let mut v_bytes: [u8; 16] = [0; 16];
                let lat_bytes = ((value.lat * dev) as i32).to_be_bytes();
                let lon_bytes = ((value.lon * dev) as i32).to_be_bytes();

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
                let dev = 100f64;

                let mut v_bytes: [u8; 16] = [0; 16];
                let lat_bytes = ((value.lat * dev) as i16).to_be_bytes();
                let lon_bytes = ((value.lon * dev) as i16).to_be_bytes();

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

            return Ok(*kind);

        } else { // new value

            addr = match schema.type_state {
                16 => { memory.malloc([0; 16].to_vec())? },
                8 => { memory.malloc([0; 8].to_vec())? },
                4 => { memory.malloc([0; 4].to_vec())? },
                _ => { 0 }
            };

            // set values in buffer
            for x in 0..value_bytes.len() {
                if x < value_bytes_size {
                    write_bytes[(addr + x as u32) as usize] = value_bytes[x as usize];
                }
            }

            return Ok(memory.set_value_address(address, addr as u32, kind));
        }
        
    }

    

}

impl<'a> NP_ValueInto<'a> for NP_Geo {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {

        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        Ok(match schema.type_state {
            4 => {
                let bytes_lat: [u8; 2] = *buffer.get_2_bytes(addr).unwrap_or(&[0; 2]);
                let bytes_lon: [u8; 2] = *buffer.get_2_bytes(addr + 2).unwrap_or(&[0; 2]);

                let lat = i16::from_be_bytes(bytes_lat) as f64;
                let lon = i16::from_be_bytes(bytes_lon) as f64;

                let dev = 100f64;

                Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
            },
            8 => {
                let bytes_lat: [u8; 4] = *buffer.get_4_bytes(addr).unwrap_or(&[0; 4]);
                let bytes_lon: [u8; 4] = *buffer.get_4_bytes(addr + 4).unwrap_or(&[0; 4]);

                let lat = i32::from_be_bytes(bytes_lat) as f64;
                let lon = i32::from_be_bytes(bytes_lon) as f64;

                let dev = 10000000f64;

                Some(Box::new(NP_Geo { lat: lat / dev, lon: lon / dev}))
            },
            16 => {
         
                let bytes_lat: [u8; 8] = *buffer.get_8_bytes(addr).unwrap_or(&[0; 8]);
                let bytes_lon: [u8; 8] = *buffer.get_8_bytes(addr + 8).unwrap_or(&[0; 8]);

                Some(Box::new(NP_Geo { lat: f64::from_be_bytes(bytes_lat), lon: f64::from_be_bytes(bytes_lon)}))

            }
            _ => {
                unreachable!();
            }
        })
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        let mut object = JFMap::<JFObject>::new();

                        object.insert("lat".to_owned(), JFObject::Float(y.lat));
                        object.insert("lon".to_owned(), JFObject::Float(y.lon));
                        
                        JFObject::Dictionary(object)
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(schema.type_state as u32)
        }
    }



}

/// Represents a ULID type which has a 6 byte timestamp and 10 bytes of randomness
/// 
/// Useful for storing time stamp data that doesn't have collisions.
pub struct NP_ULID {
    pub time: u64,
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

    /// Generates 
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

    fn is_type( type_str: &str) -> bool {
        "ulid" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Ulid as i64, "ulid".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Ulid as i64, "ulid".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        let timebits: [u8; 8] = value.time.to_be_bytes();
        let idbits: [u8; 16] = value.id.to_be_bytes();

        if addr != 0 { // existing value, replace

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..16 {
                if x < 6 {
                    write_bytes[(addr + x as u32) as usize] = timebits[x as usize + 2];
                } else {
                    write_bytes[(addr + x as u32) as usize] = idbits[x as usize];
                }
            }

            return Ok(*kind);

        } else { // new value

            let mut bytes: [u8; 16] = [0; 16];

            for x in 0..bytes.len() {
                if x < 6 {
                    bytes[(addr + x as u32) as usize] = timebits[x as usize + 2];
                } else {
                    bytes[(addr + x as u32) as usize] = idbits[x as usize];
                }
            }

            addr = memory.malloc(bytes.to_vec())?;

            return Ok(memory.set_value_address(address, addr as u32, kind));
        }                    
        
    }
}

impl<'a> NP_ValueInto<'a> for NP_ULID {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let mut time_bytes: [u8; 8] = [0; 8];
        let mut id_bytes: [u8; 16] = [0; 16];

        let read_bytes: [u8; 16] = *buffer.get_16_bytes(addr).unwrap_or(&[0; 16]);

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

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        JFObject::String(y.to_string())
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(16)
        }
    }


}

/// Represents a V4 UUID, good for globally unique identifiers
/// 
/// `uuid` types are always represented with this struct.
pub struct NP_UUID {
    pub value: [u8; 16]
}

impl NP_UUID {

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

    fn is_type( type_str: &str) -> bool {
        "uuid" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Uuid as i64, "uuid".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Uuid as i64, "uuid".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        if addr != 0 { // existing value, replace
            let bytes = value.value;
            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(*kind);

        } else { // new value

            let bytes = value.value;
            addr = memory.malloc(bytes.to_vec())?;

            return Ok(memory.set_value_address(address, addr as u32, kind));
        }                    
        
    }


}

impl<'a> NP_ValueInto<'a> for NP_UUID {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;
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

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        JFObject::String(y.to_string())
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(16)
        }
    }


}

/// Represents the string value of a choice in a schema
pub struct NP_Option {
    pub value: Option<String>
}

impl NP_Option {
    pub fn new(value: String) -> NP_Option {
        NP_Option {
            value: Some(value)
        }
    }

    pub fn empty() -> NP_Option {
        NP_Option {
            value: None
        }
    }
    
    pub fn set(&mut self, value: Option<String>) {
        self.value = value;
    }
}

impl NP_Value for NP_Option {

    fn is_type( type_str: &str) -> bool {
        "option" == type_str || "enum" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Enum as i64, "option".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Enum as i64, "option".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();
        
        match &*schema.kind {
            NP_SchemaKinds::Enum { choices } => {

                let mut value_num: i32 = -1;

                {
                    let mut ct: u16 = 0;

                    for opt in choices {
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

                    let write_bytes = memory.write_bytes();

                    // overwrite existing values in buffer
                    for x in 0..bytes.len() {
                        write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
                    }
                    return Ok(*kind);

                } else { // new value

                    addr = memory.malloc(bytes.to_vec())?;

                    return Ok(memory.set_value_address(address, addr as u32, kind));
                }                    
                
                
            },
            _ => {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str("option");
                err.push_str(") to schema of type (");
                err.push_str(schema.type_data.1.as_str());
                err.push_str(")");
                Err(NP_Error::new(err))
            }
        }
    }


}

impl Default for NP_Option {
    fn default() -> Self { 
        NP_Option { value: None }
     }
}

impl<'a> NP_ValueInto<'a> for NP_Option {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;

        match &*schema.kind {
            NP_SchemaKinds::Enum { choices } => {

                Ok(match memory.get_1_byte(addr) {
                    Some(x) => {
                        let value_num = u8::from_be_bytes([x]) as usize;
        
                        if value_num > choices.len() {
                            None
                        } else {
                            Some(Box::new(NP_Option { value: Some(choices[value_num].clone()) }))
                        }
                    },
                    None => None
                })
            },
            _ => {
                let mut err = "TypeError: Attempted to cast type (".to_owned();
                err.push_str("option");
                err.push_str(") to schema of type (");
                err.push_str(schema.type_data.1.as_str());
                err.push_str(")");
                Err(NP_Error::new(err))
            }
        }
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        match y.value {
                            Some(str_value) => {
                                JFObject::String(str_value)
                            },
                            None => JFObject::Null
                        }
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>() as u32)
        }
    }


}


impl NP_Value for bool {

    fn is_type( type_str: &str) -> bool {
        "bool" == type_str || "boolean" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Boolean as i64, "bool".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Boolean as i64, "bool".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        if addr != 0 { // existing value, replace
            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            // overwrite existing values in buffer
            memory.write_bytes()[addr as usize] = bytes[0];

            return Ok(*kind);

        } else { // new value

            let bytes = if **value == true {
                [1] as [u8; 1]
            } else {
                [0] as [u8; 1]
            };

            addr = memory.malloc(bytes.to_vec())?;
            return Ok(memory.set_value_address(address, addr as u32, kind));
        }
        
    }


}

impl<'a> NP_ValueInto<'a> for bool {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;

        Ok(match memory.get_1_byte(addr) {
            Some(x) => {
                Some(Box::new(if x == 1 { true } else { false }))
            },
            None => None
        })
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        if *y == true {
                            JFObject::True
                        } else {
                            JFObject::False
                        }
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u8>() as u32)
        }
    }


}

/// Stores the current unix epoch in u64
pub struct NP_Date {
    pub value: u64
}

impl NP_Date {
    pub fn new(time: u64) -> Self {
        NP_Date { value: time }
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

    fn is_type( type_str: &str) -> bool {
        "date" == type_str
    }

    fn type_idx() -> (i64, String) { (NP_TypeKeys::Date as i64, "date".to_owned()) }
    fn self_type_idx(&self) -> (i64, String) { (NP_TypeKeys::Date as i64, "date".to_owned()) }

    fn buffer_set(address: u32, kind: &NP_PtrKinds, _schema: &NP_Schema, memory: &NP_Memory, value: Box<&Self>) -> core::result::Result<NP_PtrKinds, NP_Error> {

        let mut addr = kind.get_value();

        if addr != 0 { // existing value, replace
            let bytes = value.value.to_be_bytes();

            let write_bytes = memory.write_bytes();

            // overwrite existing values in buffer
            for x in 0..bytes.len() {
                write_bytes[(addr + x as u32) as usize] = bytes[x as usize];
            }

            return Ok(*kind);

        } else { // new value

            let bytes = value.value.to_be_bytes();
            addr = memory.malloc(bytes.to_vec())?;
            return Ok(memory.set_value_address(address, addr as u32, kind));
        }                    
        
    }


}

impl<'a> NP_ValueInto<'a> for NP_Date {
    fn buffer_into(_address: u32, kind: NP_PtrKinds, _schema: &'a NP_Schema, buffer: &NP_Memory) -> core::result::Result<Option<Box<Self>>, NP_Error> {
        let addr = kind.get_value() as usize;

        // empty value
        if addr == 0 {
            return Ok(None);
        }

        let memory = buffer;
        Ok(match memory.get_8_bytes(addr) {
            Some(x) => {
                Some(Box::new(NP_Date { value: u64::from_be_bytes(*x) }))
            },
            None => None
        })
    }

    fn buffer_to_json(address: u32, kind: &NP_PtrKinds, schema: &NP_Schema, buffer: &NP_Memory) -> JFObject {
        let this_string = Self::buffer_into(address, *kind, schema, buffer);

        match this_string {
            Ok(x) => {
                match x {
                    Some(y) => {
                        JFObject::Float(y.value as f64)
                    },
                    None => {
                        JFObject::Null
                    }
                }
            },
            Err(_e) => {
                JFObject::Null
            }
        }
    }

    fn buffer_get_size(_address: u32, kind: &'a NP_PtrKinds, _schema: &'a NP_Schema, _buffer: &'a NP_Memory) -> core::result::Result<u32, NP_Error> {
        let addr = kind.get_value() as usize;

        if addr == 0 {
            return Ok(0) 
        } else {
            Ok(core::mem::size_of::<u64>() as u32)
        }
    }
}