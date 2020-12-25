//! JSON Parser, serializer and deserializer
//! 
//! This file is derived from the json_flex crate.
//! 
//! [github](https://github.com/nacika-ins/json_flex) | [crates.io](https://crates.io/crates/json_flex)
//! 
//! Changes:
//! - Library has been converted & stripped for no_std use
//! - All `.unwrap()`s have been replaced with proper error handling
//! - Several additions that were needed for NoProto
//! - Some minor optimizations
//! 
//! The MIT License (MIT)
//! 
//! Copyright (c) 2015 nacika
//! Copyright (c) 2020 Scott Lott
//! 
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//! 
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//! 
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.


use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
use alloc::borrow::ToOwned;
use alloc::string::ToString;
use core::str::FromStr;
use core::ops::Index;
use crate::{error::NP_Error};

/// The JSON representation of a JS Map
#[derive(Debug, Clone)]
pub struct JSMAP {
    /// The vec of values in the map
    pub values: Vec<(String, NP_JSON)>
}

impl JSMAP {

    /// Generate a new empty map
    pub fn new() -> Self {
        JSMAP { values: Vec::new() }
    }

    /// Insert a value into the map
    pub fn insert(&mut self, key: String, value: NP_JSON) -> usize {

        for x in 0..self.values.len() {
            if self.values[x].0 == key {
                self.values[x] = (key, value);
                return x;
            }
        }

        self.values.push((key, value));

        self.values.len()
    }

    /// Get a mutable reference to a value in the map
    pub fn get_mut(&mut self, key: &str) -> Option<&mut NP_JSON> {
        for x in 0..self.values.len() {
            if self.values[x].0 == *key {
                return Some(&mut self.values[x].1);
            }
        }
        None
    }

    /// Get an immutable reference to a value in the map
    pub fn get(&self, key: &str) -> Option<&NP_JSON> {
        for x in 0..self.values.len() {
            if self.values[x].0 == *key {
                return Some(&self.values[x].1);
            }
        }
        None
    }

    /// Check if a value exists in the map
    pub fn has(&self, key: &str) -> bool {
        for x in 0..self.values.len() {
            if self.values[x].0 == *key {
                return true;
            }
        }
        false
    }
}

/// Represents an JSON value
#[derive(Debug, Clone)]
pub enum NP_JSON {
    /// String JSON type
    String(String), 
    /// Integer JSON type
    Integer(i64), 
    /// Float JSON type
    Float(f64), 
    /// Map JSON type
    Dictionary(JSMAP), 
    /// List JSON type
    Array(Vec<NP_JSON>), 
    /// NULL json type
    Null, 
    /// boolean false type
    False, 
    /// boolean true type
    True,
}



impl NP_JSON {


    /// copy this value and it's children
    pub fn clone(&self) -> NP_JSON {

        match self {
            NP_JSON::Dictionary(map) => {
                let mut new_map = JSMAP::new();

                for item in &map.values {
                    let cloned = {
                        (
                            item.0.clone(),
                            item.1.clone()
                        )
                    };
                    new_map.values.push(cloned);
                }

                NP_JSON::Dictionary(new_map)
            },
            NP_JSON::Array(list) => {
                let mut array = Vec::new();
                for item in list {
                    array.push(item.clone());
                }
                NP_JSON::Array(array)
            },
            NP_JSON::String(strng) => {
                NP_JSON::String(strng.clone())
            },
            NP_JSON::Integer(int) => {
                NP_JSON::Integer(*int)
            },
            NP_JSON::Float(num) => {
                NP_JSON::Float(*num)
            },
            NP_JSON::Null => {
                NP_JSON::Null
            },
            NP_JSON::False => {
                NP_JSON::False
            },
            NP_JSON::True => {
                NP_JSON::True
            },
        }
    }
    /// Get this value as a string
    pub fn into_string(&self) -> Option<&String> {
        match self {
            &NP_JSON::String(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get this value as an i64
    pub fn into_i64(&self) -> Option<&i64> {
        match self {
            &NP_JSON::Integer(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get this value as an f64
    pub fn into_f64(&self) -> Option<&f64> {
        match self {
            &NP_JSON::Float(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get this value as a hashmap
    pub fn into_hashmap(&self) -> Option<&JSMAP> {
        match self {
            &NP_JSON::Dictionary(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get this value as a list
    pub fn into_vec(&self) -> Option<&Vec<NP_JSON>> {
        match self {
            &NP_JSON::Array(ref v) => Some(v),
            _ => None,
        }
    }
    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        match self {
            &NP_JSON::Null => true,
            _ => false,
        }
    }
    /// Check if this value is boolean true
    pub fn is_true(&self) -> bool {
        match self {
            &NP_JSON::True => true,
            _ => false,
        }
    }
    /// Check if this value is boolean false
    pub fn is_false(&self) -> bool {
        match self {
            &NP_JSON::False => true,
            _ => false,
        }
    }
    /// Check if this value is array
    pub fn is_array(&self) -> bool {
        match self {
            &NP_JSON::Array(_) => true,
            _ => false,
        }
    }
    /// Check if this value is map
    pub fn is_dictionary(&self) -> bool {
        match self {
            &NP_JSON::Dictionary(_) => true,
            _ => false,
        }
    }
    /// Check if this value is string
    pub fn is_string(&self) -> bool {
        match self {
            &NP_JSON::String(_) => true,
            _ => false,
        }
    }
    /// Check if this value is an integer
    pub fn is_integer(&self) -> bool {
        match self {
            &NP_JSON::Integer(_) => true,
            _ => false,
        }
    }
    /// Check if this value is float
    pub fn is_float(&self) -> bool {
        match self {
            &NP_JSON::Float(_) => true,
            _ => false,
        }
    }
    /// Get a reference to the string in this value if it's a string
    pub fn unwrap_string(&self) -> Option<&String> {
        match self {
            &NP_JSON::String(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get a reference to the i64 in this value if it's a i64
    pub fn unwrap_i64(&self) -> Option<&i64> {
        match self {
            &NP_JSON::Integer(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get a reference to the f64 in this value if it's a f64
    pub fn unwrap_f64(&self) -> Option<&f64> {
        match self {
            &NP_JSON::Float(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get a reference to the hashmap in this value if it's a hashmap
    pub fn unwrap_hashmap(&self) -> Option<&JSMAP> {
        match self {
            &NP_JSON::Dictionary(ref v) => Some(v),
            _ => None,
        }
    }
    /// Get a reference to the list in this value if it's a list
    pub fn unwrap_vec(&self) -> Option<&Vec<NP_JSON>> {
        match self {
            &NP_JSON::Array(ref v) => Some(v),
            _ => None,
        }
    }
    /// Stringify this JSON object and it's children
    pub fn stringify(&self) -> String {
        match self {
            &NP_JSON::String(ref v) => {
                let mut string: String = "\"".to_owned();
                string.push_str(v.replace("\"", "\\\"").as_str());
                string.push_str("\"");
                string
            },
            &NP_JSON::Integer(ref v) => v.to_string(),
            &NP_JSON::Float(ref v) => v.to_string(),
            &NP_JSON::Dictionary(ref v) => {
                let mut string: String = "{".to_owned();
                let mut is_first = true;
                for (k, v) in &v.values {
                    if is_first {
                        is_first = false;
                    } else {
                        string.push(',');
                    }
                    let mut substring = "\"".to_owned();
                    substring.push_str(k.replace("\"", "\\\"").as_str());
                    substring.push_str("\":");
                    string.push_str(substring.as_str());
                    string.push_str(&v.stringify());
                }
                string.push_str("}");
                string
            }
            &NP_JSON::Array(ref v) => {
                let mut string: String = "".to_owned();
                let mut is_first = true;
                for i in v {
                    if is_first {
                        is_first = false;
                    } else {
                        string.push(',');
                    }
                    string.push_str(&i.stringify());
                }
                let mut return_string = "[".to_owned();
                return_string.push_str(string.as_str());
                return_string.push_str("]");
                return_string
            }
            &NP_JSON::Null => "null".to_owned(),
            &NP_JSON::False => "false".to_owned(),
            &NP_JSON::True => "true".to_owned(),
        }
    }
}

impl Index<usize> for NP_JSON {
    type Output = NP_JSON;
    fn index<'a>(&'a self, id: usize) -> &'a Self::Output {
        match self.into_vec() {
            Some(x) => {
                match x.get(id) {
                    Some(y) => y,
                    None => &NP_JSON::Null
                }
            },
            None => &NP_JSON::Null
        }
    }
}

impl Index<String> for NP_JSON {
    type Output = NP_JSON;
    fn index<'a>(&'a self, id: String) -> &'a Self::Output {
        panic!()
    }
}

impl<'a> Index<&'a str> for NP_JSON {
    type Output = NP_JSON;
    fn index<'b>(&'b self, id: &str) -> &'b Self::Output {
        match self.into_hashmap() {
            Some(x) => {
                match x.get(&id.to_owned()) {
                    Some(y) => y,
                    None => &NP_JSON::Null
                }
            },
            None => &NP_JSON::Null
        }
    }
}


fn recursive(v: &mut NP_JSON,
             a_chain: Vec<i64>,
             d_chain: Vec<String>,
             mut a_nest: i64,
             mut d_nest: i64,
             last_chain: char,
             last_c: char,
             func: fn(&mut NP_JSON,
                      Option<String>,
                      Vec<i64>,
                      Vec<String>,
                      i64,
                      i64,
                      char) -> Result<(), NP_Error>
                     ,
             value: Option<String>,
             log: String)
             -> Result<bool, NP_Error> {

    let is_find = match *v {

        NP_JSON::Array(ref mut vvz) => {
            let i = *NP_Error::unwrap(a_chain.get(a_nest as usize))?;
            let is_find: bool = {
                let vvv = vvz.get_mut(i as usize);
                let is_find: bool = match vvv {
                    Some(mut vvvv) => {
                        a_nest += 1;
                        recursive(&mut vvvv,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  value.clone(),
                                  log)?;
                        a_nest -= 1;
                        true
                    }
                    None => false,
                };
                is_find
            };
            if !is_find {
            }
            is_find
        }

        NP_JSON::Dictionary(ref mut vv) => {
            let o_key = d_chain.get(d_nest as usize);
            match o_key {
                Some(ref key) => {
                    let vvv: Option<&mut NP_JSON> = vv.get_mut(*key);              

                    let is_find: bool = match vvv {
                        Some(mut vvvv) => {
                            d_nest += 1;
                            recursive(&mut vvvv,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      value.clone(),
                                      log)?;
                            d_nest -= 1;
                            true
                        }
                        None => false,
                    };
                    is_find
                }
                None => false,
            }
        }
        _ => true,
    };

    if !is_find {
        func(v,
             value,
             a_chain.clone(),
             d_chain.clone(),
             a_nest,
             d_nest,
             last_c)?;
    }
    Ok(is_find)
}

/// Parse a JSON string into a JSON object in memory
pub fn json_decode<'json>(text: String) -> Result<Box<NP_JSON>, NP_Error> {

    let mut ret = Box::new(NP_JSON::Null);

    let mut pos: usize = 0;

    let mut chain: Vec<char> = Vec::new();
    let mut d_chain: Vec<String> = Vec::new();
    let mut a_chain: Vec<i64> = Vec::new();
    let mut last_chain: char = ' ';
    let mut last_active_char: char = ' ';
    let mut key: String;
    let mut string: String = "".to_owned();
    let mut num: String = "".to_owned();
    let mut last_c: char = ' ';
    let mut s_true: String = "".to_owned();
    let mut s_false: String = "".to_owned();
    let mut s_null: String = "".to_owned();

    let body: Vec<char> = text.chars().collect();
    let size = body.len();
    let mut done = false;
    while !done {

        let c: char = body[pos];

        match last_chain {
            's' => {
                string.push(c);
            }
            'w' => {
                string.push(c);
            }
            'n' => {
                num.push(c);
            }
            't' => {
                s_true.push(c);
            }
            'f' => {
                s_false.push(c);
            }
            '0' => {
                s_null.push(c);
            }
            _ => {}
        };

        match c {

            '[' => {

                match last_chain {

                    's' => {}
                    'w' => {}

                    _ => {

                        let a = 'a';
                        chain.push(a);
                        last_chain = a;
                        a_chain.push(0);

                        let is_root = match *ret {

                            NP_JSON::Null => {
                                *ret = NP_JSON::Array(Vec::new());
                                true
                            }

                            _ => false,
                        };

                        if !is_root {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::Array(Vec::new()));
                                    }
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::Array(Vec::new()));
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;
                        }
                    }
                };
                last_active_char = c.clone();
            }

            ']' => {
                match last_chain {

                    's' => {}
                    'w' => {}
                    't' => {

                        NP_Error::unwrap(s_true.pop())?;
                        s_true = s_true.trim().to_string();
                        if s_true != "true" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();
                        fn func(v: &mut NP_JSON,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {
                                    vv.push(NP_JSON::True);
                                }
                                _ => {}
                            };
                            Ok(())
                        }
                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  None,
                                  log)?;

                        NP_Error::unwrap(chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(a_chain.pop())?;
                        s_true = "".to_owned();
                    }

                    'f' => {

                        NP_Error::unwrap(s_false.pop())?;
                        s_false = s_false.trim().to_string();
                        if s_false != "false" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();
                        fn func(v: &mut NP_JSON,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {
                                    vv.push(NP_JSON::False);
                                }
                                _ => {}
                            };
                            Ok(())
                        }
                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  None,
                                  log)?;

                        NP_Error::unwrap(chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(a_chain.pop())?;

                        s_false = "".to_owned();
                    }

                    '0' => {

                        NP_Error::unwrap(s_null.pop())?;
                        s_null = s_null.trim().to_string();
                        if s_null != "null" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();
                        fn func(v: &mut NP_JSON,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {
                                    vv.push(NP_JSON::Null);
                                }
                                _ => {}
                            };
                            Ok(())
                        }
                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  None,
                                  log)?;


                        NP_Error::unwrap(chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(a_chain.pop())?;

                        s_null = "".to_owned();
                    }

                    'n' => {

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();
                        fn func(v: &mut NP_JSON,
                                value: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {

                                    let mut new_num = NP_Error::unwrap(value)?;
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => vv.push( NP_JSON::Float(f64::from_str(&new_num.clone())?) ),
                                        None    => vv.push( NP_JSON::Integer(i64::from_str(&new_num.clone())?) ),
                                    };
                                }
                                _ => {}
                            };
                            Ok(())
                        }
                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  Some(num),
                                  log)?;

                        num = "".to_owned();

                        NP_Error::unwrap(chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(a_chain.pop())?;

                    }

                    'a' => {


                        if last_active_char == ',' {

                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    _: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::Null);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;

                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(a_chain.pop())?;
                    }

                    _ => return Err(NP_Error::new("JSON Parse Error: Unknown chain from Array")),
                }

                last_active_char = c.clone();

            }

            '{' => {

                match last_chain {

                    's' => {}
                    'w' => {}

                    'v' => {

                        let a = 'd';
                        chain.push(a);
                        last_chain = a;

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();

                        fn func(v: &mut NP_JSON,
                                _: Option<String>,
                                _: Vec<i64>,
                                d_chain: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {
                                    vv.push(NP_JSON::Dictionary(JSMAP::new()));
                                }
                                NP_JSON::Dictionary(ref mut vv) => {
                                    let key = NP_Error::unwrap(d_chain.last())?.clone();
                                    vv.insert(key, NP_JSON::Dictionary(JSMAP::new()));
                                }
                                _ => {}
                            };
                            Ok(())
                        }

                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  None,
                                  log)?;
                    }

                    _ => {

                        let a = 'd';
                        chain.push(a);
                        last_chain = a;


                        let is_root = match *ret {
                            NP_JSON::Null => {
                                *ret = NP_JSON::Dictionary(JSMAP::new());
                                true
                            }
                            _ => false,
                        };

                        if !is_root {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::Dictionary(JSMAP::new()));
                                    }
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::Dictionary(JSMAP::new()));
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;
                        }
                    }
                }

                last_active_char = c.clone();

            }

            '}' => {
                match last_chain {

                    's' => {}
                    'w' => {}

                    't' => {

                        NP_Error::unwrap(s_true.pop())?;
                        s_true = s_true.trim().to_string();
                        if s_true != "true" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::True);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;
                        }

                        s_true = "".to_owned();
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }

                    'f' => {

                        NP_Error::unwrap(s_false.pop())?;
                        s_false = s_false.trim().to_string();
                        if s_false != "false" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::False);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;

                        }

                        s_false = "".to_owned();
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }

                    '0' => {

                        NP_Error::unwrap(s_null.pop())?;
                        s_null = s_null.trim().to_string();
                        if s_null != "null" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();


                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::Null);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;
                        }

                        s_null = "".to_owned();
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }

                    'n' => {

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    value: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        let mut value = NP_Error::unwrap(value)?;
                                        NP_Error::unwrap(value.pop())?;
                                        value = value.trim().to_string();
                                        match value.find('.') {
                                            Some(_) => vv.insert(key, NP_JSON::Float(f64::from_str(&value.clone())?)) ,
                                            None    => vv.insert(key, NP_JSON::Integer(i64::from_str(&value.clone())?)),
                                        };
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      Some(num.clone()),
                                      log)?;

                        }
                        num = "".to_owned();
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }

                    'v' => {
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }

                    _ => {
                        NP_Error::unwrap(d_chain.pop())?;
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                    }
                }
                last_active_char = c.clone();
            }

            ':' => {
                match last_chain {

                    's' => {}
                    'w' => {}

                    'd' => {

                        let v = 'v';
                        chain.push(v);
                        last_chain = v;

                        key = string.clone();
                        NP_Error::unwrap(key.pop())?;

                        d_chain.push(key.clone());

                        string = "".to_owned();
                    }

                    _ => {}
                }

                last_active_char = c.clone();

            }

            ',' => {
                match last_chain {

                    's' => {}
                    'w' => {}

                    't' => {

                        NP_Error::unwrap(s_true.pop())?;
                        s_true = s_true.trim().to_string();
                        if s_true != "true" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        if last_chain == 't' {

                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::True);
                                    }
                                    NP_JSON::Dictionary(ref mut vv) => {

                                        let key = NP_Error::unwrap(d_chain.last())?.clone();

                                        vv.insert(key, NP_JSON::True);

                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;

                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();
                            NP_Error::unwrap(d_chain.pop())?;
                        } else {
                            let a = NP_Error::unwrap(a_chain.pop())?;
                            a_chain.push(a + 1i64);
                        }

                        s_true = "".to_owned();
                    }

                    'f' => {

                        NP_Error::unwrap(s_false.pop())?;
                        s_false = s_false.trim().to_string();
                        if s_false != "false" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        if last_chain == 'f' {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::False);
                                    }
                                    NP_JSON::Dictionary(ref mut vv) => {

                                        let key = NP_Error::unwrap(d_chain.last())?.clone();

                                        vv.insert(key, NP_JSON::False);

                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;

                        }

                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();
                            NP_Error::unwrap(d_chain.pop())?;
                        } else {
                            let a = NP_Error::unwrap(a_chain.pop())?;
                            a_chain.push(a + 1i64);
                        }

                        s_false = "".to_owned();
                    }

                    '0' => {

                        NP_Error::unwrap(s_null.pop())?;
                        s_null = s_null.trim().to_string();
                        if s_null != "null" {
                            return Err(NP_Error::new("JSON Parse Error"));
                        }

                        if last_chain == '0' {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::Null);
                                    }
                                    NP_JSON::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, NP_JSON::Null);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            NP_Error::unwrap(chain.pop())?;
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;

                        }

                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();
                            NP_Error::unwrap(d_chain.pop())?;
                        } else {
                            let a = NP_Error::unwrap(a_chain.pop())?;
                            a_chain.push(a + 1i64);
                        }
                        s_null = "".to_owned();
                    }

                    'a' => {
                        let a = NP_Error::unwrap(a_chain.pop())?;
                        a_chain.push(a + 1i64);
                        if last_active_char == '[' || last_active_char == ',' {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut NP_JSON,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    _: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    NP_JSON::Array(ref mut vv) => {
                                        vv.push(NP_JSON::Null);
                                    }
                                    _ => {}
                                };
                                Ok(())
                            }
                            recursive(&mut ret,
                                      a_chain.clone(),
                                      d_chain.clone(),
                                      a_nest,
                                      d_nest,
                                      last_chain,
                                      last_c,
                                      func,
                                      None,
                                      log)?;
                        }
                    }

                    'n' => {

                        let a_nest = 0i64;
                        let d_nest = 0i64;
                        let log: String = "".to_owned();
                        fn func(v: &mut NP_JSON,
                                value: Option<String>,
                                _: Vec<i64>,
                                d_chain: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                NP_JSON::Array(ref mut vv) => {
                                    let mut new_num = NP_Error::unwrap(value)?.clone();
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => {
                                            vv.push(NP_JSON::Float(f64::from_str(&new_num)?))
                                        }
                                        None => {
                                            vv.push(NP_JSON::Integer(i64::from_str(&new_num)?))
                                        }
                                    };

                                }
                                NP_JSON::Dictionary(ref mut vv) => {

                                    let key = NP_Error::unwrap(d_chain.last())?.clone();

                                    let mut new_num = NP_Error::unwrap(value)?.clone();
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => {
                                            vv.insert(key,
                                                      NP_JSON::Float(f64::from_str(&new_num)?))
                                        }
                                        None => {
                                            vv.insert(key,
                                                      NP_JSON::Integer(i64::from_str(&new_num)?))
                                        }
                                    };


                                }
                                _ => {}
                            };
                            Ok(())
                        }
                        recursive(&mut ret,
                                  a_chain.clone(),
                                  d_chain.clone(),
                                  a_nest,
                                  d_nest,
                                  last_chain,
                                  last_c,
                                  func,
                                  Some(num),
                                  log)?;

                        num = "".to_owned();
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();

                        if last_chain == 'v' {
                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();
                            NP_Error::unwrap(d_chain.pop())?;
                        } else {
                            let a = NP_Error::unwrap(a_chain.pop())?;
                            a_chain.push(a + 1i64);
                        }

                    }

                    'v' => {
                        NP_Error::unwrap(chain.pop())?;
                        last_chain = chain.last().unwrap_or(&' ').to_owned();
                        NP_Error::unwrap(d_chain.pop())?;
                    }
                    _ => {}
                }

                last_active_char = c.clone();

            }

            '"' => {


                match last_chain {

                    'w' => {
                        if last_c != '\\' {

                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();

                            if last_chain == 'v' {

                                let a_nest = 0i64;
                                let d_nest = 0i64;
                                let log: String = "".to_owned();
                                fn func(v: &mut NP_JSON,
                                        value: Option<String>,
                                        _: Vec<i64>,
                                        d_chain: Vec<String>,
                                        _: i64,
                                        _: i64,
                                        _: char) -> Result<(), NP_Error> {

                                    match *v {
                                        NP_JSON::Dictionary(ref mut vv) => {
                                            let key = NP_Error::unwrap(d_chain.last())?.clone();
                                            let mut value = NP_Error::unwrap(value)?;
                                            NP_Error::unwrap(value.pop())?;
                                            vv.insert(key, NP_JSON::String(value.clone()));
                                        }
                                        _ => {}
                                    };
                                    Ok(())
                                }
                                recursive(&mut ret,
                                          a_chain.clone(),
                                          d_chain.clone(),
                                          a_nest,
                                          d_nest,
                                          last_chain,
                                          last_c,
                                          func,
                                          Some(string.clone()),
                                          log)?;
                                string = "".to_owned();
                            } else if last_chain != 'd' {
                                NP_Error::unwrap(string.pop())?;
                                let is_root = match *ret {
                                    NP_JSON::Null => {
                                        *ret = NP_JSON::String(string.clone());
                                        true
                                    }
                                    _ => false,
                                };

                                if !is_root {
                                    let a_nest = 0i64;
                                    let d_nest = 0i64;
                                    let log: String = "".to_owned();
                                    fn func(v: &mut NP_JSON,
                                            value: Option<String>,
                                            _: Vec<i64>,
                                            _: Vec<String>,
                                            _: i64,
                                            _: i64,
                                            _: char) -> Result<(), NP_Error> {
                                        match *v {
                                            NP_JSON::Array(ref mut vv) => {
                                                vv.push(NP_JSON::String(NP_Error::unwrap(value)?
                                                                              .clone()));
                                            }
                                            _ => {}
                                        };
                                        Ok(())
                                    }
                                    recursive(&mut ret,
                                              a_chain.clone(),
                                              d_chain.clone(),
                                              a_nest,
                                              d_nest,
                                              last_chain,
                                              last_c,
                                              func,
                                              Some(string),
                                              log)?;
                                }
                                string = "".to_owned();
                            }
                        }
                    }

                    _ => {
                        let w = 'w';
                        chain.push(w);
                        last_chain = w;
                        string = "".to_owned();
                    }
                }

                last_active_char = c.clone();

            }
            '\'' => {
                match last_chain {
                    's' => {
                        if last_c != '\\' {

                            NP_Error::unwrap(chain.pop())?;
                            last_chain = chain.last().unwrap_or(&' ').to_owned();

                            if last_chain == 'v' {
                                let a_nest = 0i64;
                                let d_nest = 0i64;
                                let log: String = "".to_owned();
                                fn func(v: &mut NP_JSON,
                                        value: Option<String>,
                                        _: Vec<i64>,
                                        d_chain: Vec<String>,
                                        _: i64,
                                        _: i64,
                                        _: char) -> Result<(), NP_Error> {

                                    match *v {
                                        NP_JSON::Dictionary(ref mut vv) => {
                                            let key = NP_Error::unwrap(d_chain.last())?.clone();
                                            let mut value = NP_Error::unwrap(value)?;
                                            NP_Error::unwrap(value.pop())?;
                                            vv.insert(key, NP_JSON::String(value.clone()));
                                        }
                                        _ => {}
                                    };
                                    Ok(())
                                }
                                recursive(&mut ret,
                                          a_chain.clone(),
                                          d_chain.clone(),
                                          a_nest,
                                          d_nest,
                                          last_chain,
                                          last_c,
                                          func,
                                          Some(string.clone()),
                                          log)?;
                                          NP_Error::unwrap(d_chain.pop())?;
                                string = "".to_owned();
                            } else {
                                NP_Error::unwrap(string.pop())?;
                                let is_root = match *ret {
                                    NP_JSON::Null => {
                                        *ret = NP_JSON::String(string.clone());
                                        true
                                    }
                                    _ => false,
                                };

                                if !is_root {
                                    let a_nest = 0i64;
                                    let d_nest = 0i64;
                                    let log: String = "".to_owned();
                                    fn func(v: &mut NP_JSON,
                                            value: Option<String>,
                                            _: Vec<i64>,
                                            _: Vec<String>,
                                            _: i64,
                                            _: i64,
                                            _: char) -> Result<(), NP_Error> {
                                        match *v {
                                            NP_JSON::Array(ref mut vv) => {
                                                vv.push(NP_JSON::String(NP_Error::unwrap(value)?
                                                                              .clone()));
                                            }
                                            _ => {}
                                        };
                                        Ok(())
                                    }
                                    recursive(&mut ret,
                                              a_chain.clone(),
                                              d_chain.clone(),
                                              a_nest,
                                              d_nest,
                                              last_chain,
                                              last_c,
                                              func,
                                              Some(string),
                                              log)?;
                                }
                                string = "".to_owned();
                            }
                        }
                    }
                    _ => {
                        string = "".to_owned();
                        let s = 's';
                        chain.push(s);
                        last_chain = s;
                    }
                }
                last_active_char = c.clone();
            }

            '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => {
                match last_chain {
                    'n' => {}
                    'w' => {}
                    's' => {}
                    _ => {
                        num = "".to_owned();
                        let n = 'n';
                        chain.push(n);
                        last_chain = n;
                        num.push(c);
                    }
                }
                last_active_char = c.clone();
            }

            '-' => {
                match last_chain {
                    'n' => {}
                    'w' => {}
                    's' => {}
                    _ => {
                        num = "".to_owned();
                        let n = 'n';
                        chain.push(n);
                        last_chain = n;
                        num.push(c);
                    }
                }
                last_active_char = c.clone();
            }

            't' => {

                match last_chain {
                    'n' => {}
                    'w' => {}
                    's' => {}

                    _ => {
                        let t = 't';
                        chain.push(t);
                        last_chain = t;
                        s_true = "".to_owned();
                        s_true.push(c);
                    }
                }
                last_active_char = c.clone();
            }

            'f' => {
                match last_chain {
                    'n' => {}
                    'w' => {}
                    's' => {}
                    _ => {
                        let f = 'f';
                        chain.push(f);
                        last_chain = f;
                        s_false = "".to_owned();
                        s_false.push(c);
                    }
                }
                last_active_char = c.clone();
            }

            'n' => {
                match last_chain {
                    'n' => {}
                    'w' => {}
                    's' => {}
                    _ => {
                        let null = '0';
                        chain.push(null);
                        last_chain = null;
                        s_null = "".to_owned();
                        s_null.push(c);
                    }
                }
                last_active_char = c.clone();
            }

            '\n' => {}
            _ => {}
        };

        pos += 1;
        if pos >= size {
            done = true;
        }

        last_c = c.clone();

    }


    Ok(ret)
}