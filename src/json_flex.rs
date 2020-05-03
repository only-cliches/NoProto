//! JSON Parser, serializer and deserializer
//! 
//! This file is derived from the json_flex crate.
//! 
//! [github](https://github.com/nacika-ins/json_flex) | [crates.io](https://crates.io/crates/json_flex)
//! 
//! Changes:
//! - Library has been converted & stripped for no_std use
//! - All code paths that can panic have been replaced with proper error handling
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
use crate::error::NP_Error;

#[derive(Debug)]
pub struct JFMap<T> {
    pub data: Vec<(String, T)>
}

impl<T> JFMap<T> {

    pub fn new() -> Self {
        JFMap { data: Vec::new() }
    }

    pub fn insert(&mut self, key: String, value: T) -> usize {

        for x in 0..self.data.len() {
            if self.data[x].0 == key {
                self.data[x] = (key, value);
                return x;
            }
        }

        self.data.push((key, value));

        self.data.len()
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut T> {
        for x in 0..self.data.len() {
            if self.data[x].0 == *key {
                return Some(&mut self.data[x].1);
            }
        }
        None
    }

    pub fn get(&self, key: &str) -> Option<&T> {
        for x in 0..self.data.len() {
            if self.data[x].0 == *key {
                return Some(&self.data[x].1);
            }
        }
        None
    }

    pub fn has(&self, key: &str) -> bool {
        for x in 0..self.data.len() {
            if self.data[x].0 == *key {
                return true;
            }
        }
        false
    }
}

#[derive(Debug)]
pub enum JFObject {
    String(String),
    Integer(i64),
    Float(f64),
    Dictionary(JFMap<JFObject>),
    Array(Vec<JFObject>),
    Null,
    False,
    True,
}

impl JFObject {
    pub fn into_string(&self) -> Option<&String> {
        match self {
            &JFObject::String(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn into_i64(&self) -> Option<&i64> {
        match self {
            &JFObject::Integer(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn into_f64(&self) -> Option<&f64> {
        match self {
            &JFObject::Float(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn into_hashmap(&self) -> Option<&JFMap<JFObject>> {
        match self {
            &JFObject::Dictionary(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn into_vec(&self) -> Option<&Vec<JFObject>> {
        match self {
            &JFObject::Array(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn is_null(&self) -> bool {
        match self {
            &JFObject::Null => true,
            _ => false,
        }
    }
    pub fn is_true(&self) -> bool {
        match self {
            &JFObject::True => true,
            _ => false,
        }
    }
    pub fn is_false(&self) -> bool {
        match self {
            &JFObject::False => true,
            _ => false,
        }
    }
    pub fn is_array(&self) -> bool {
        match self {
            &JFObject::Array(_) => true,
            _ => false,
        }
    }
    pub fn is_dictionary(&self) -> bool {
        match self {
            &JFObject::Dictionary(_) => true,
            _ => false,
        }
    }
    pub fn is_string(&self) -> bool {
        match self {
            &JFObject::String(_) => true,
            _ => false,
        }
    }
    pub fn is_integer(&self) -> bool {
        match self {
            &JFObject::Integer(_) => true,
            _ => false,
        }
    }
    pub fn is_float(&self) -> bool {
        match self {
            &JFObject::Float(_) => true,
            _ => false,
        }
    }
    pub fn unwrap_string(&self) -> Option<&String> {
        match self {
            &JFObject::String(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn unwrap_i64(&self) -> Option<&i64> {
        match self {
            &JFObject::Integer(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn unwrap_f64(&self) -> Option<&f64> {
        match self {
            &JFObject::Float(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn unwrap_hashmap(&self) -> Option<&JFMap<JFObject>> {
        match self {
            &JFObject::Dictionary(ref v) => Some(v),
            _ => None,
        }
    }
    pub fn unwrap_vec(&self) -> Option<&Vec<JFObject>> {
        match self {
            &JFObject::Array(ref v) => Some(v),
            _ => None,
        }
    }

    pub fn to_json(&self) -> String {
        match self {
            &JFObject::String(ref v) => {
                let mut string: String = "\"".to_owned();
                string.push_str(v);
                string.push_str("\"");
                string
            },
            &JFObject::Integer(ref v) => v.to_string(),
            &JFObject::Float(ref v) => v.to_string(),
            &JFObject::Dictionary(ref v) => {
                let mut string: String = "{".to_owned();
                let mut is_first = true;
                for (k, v) in &v.data {
                    if is_first {
                        is_first = false;
                    } else {
                        string.push(',');
                    }
                    let mut substring = "\"".to_owned();
                    substring.push_str(k);
                    substring.push_str("\":");
                    string.push_str(substring.as_str());
                    string.push_str(&v.to_json());
                }
                string.push_str("}");
                string
            }
            &JFObject::Array(ref v) => {
                let mut string: String = "".to_owned();
                let mut is_first = true;
                for i in v {
                    if is_first {
                        is_first = false;
                    } else {
                        string.push(',');
                    }
                    string.push_str(&i.to_json());
                }
                let mut return_string = "[".to_owned();
                return_string.push_str(string.as_str());
                return_string.push_str("]");
                return_string
            }
            &JFObject::Null => "null".to_owned(),
            &JFObject::False => "false".to_owned(),
            &JFObject::True => "true".to_owned(),
        }
    }
}

impl Index<usize> for JFObject {
    type Output = JFObject;
    fn index<'a>(&'a self, id: usize) -> &'a Self::Output {
        match self.into_vec() {
            Some(x) => {
                match x.get(id) {
                    Some(y) => y,
                    None => &JFObject::Null
                }
            },
            None => &JFObject::Null
        }
    }
}

impl Index<String> for JFObject {
    type Output = JFObject;
    fn index<'a>(&'a self, id: String) -> &'a Self::Output {
        match self.into_hashmap() {
            Some(x) => {
                match x.get(id.as_str()) {
                    Some(y) => y,
                    None => &JFObject::Null
                }
            },
            None => &JFObject::Null
        }
    }
}

impl<'a> Index<&'a str> for JFObject {
    type Output = JFObject;
    fn index<'b>(&'b self, id: &str) -> &'b Self::Output {
        match self.into_hashmap() {
            Some(x) => {
                match x.get(&id.to_owned()) {
                    Some(y) => y,
                    None => &JFObject::Null
                }
            },
            None => &JFObject::Null
        }
    }
}


fn recursive(v: &mut JFObject,
             a_chain: Vec<i64>,
             d_chain: Vec<String>,
             mut a_nest: i64,
             mut d_nest: i64,
             last_chain: char,
             last_c: char,
             func: fn(&mut JFObject,
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

        JFObject::Array(ref mut vvz) => {
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

        JFObject::Dictionary(ref mut vv) => {
            let o_key = d_chain.get(d_nest as usize);
            match o_key {
                Some(ref key) => {
                    let vvv: Option<&mut JFObject> = vv.get_mut(*key);              

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

pub fn json_decode(text: String) -> Result<Box<JFObject>, NP_Error> {

    let mut ret = Box::new(JFObject::Null);

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

                            JFObject::Null => {
                                *ret = JFObject::Array(Vec::new());
                                true
                            }

                            _ => false,
                        };

                        if !is_root {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::Array(Vec::new()));
                                    }
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::Array(Vec::new()));
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
                        fn func(v: &mut JFObject,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {
                                    vv.push(JFObject::True);
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
                        fn func(v: &mut JFObject,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {
                                    vv.push(JFObject::False);
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
                        fn func(v: &mut JFObject,
                                _: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {
                                    vv.push(JFObject::Null);
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
                        fn func(v: &mut JFObject,
                                value: Option<String>,
                                _: Vec<i64>,
                                _: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {

                                    let mut new_num = NP_Error::unwrap(value)?;
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => vv.push( JFObject::Float(f64::from_str(&new_num.clone())?) ),
                                        None    => vv.push( JFObject::Integer(i64::from_str(&new_num.clone())?) ),
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    _: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::Null);
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

                        fn func(v: &mut JFObject,
                                _: Option<String>,
                                _: Vec<i64>,
                                d_chain: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {
                                    vv.push(JFObject::Dictionary(JFMap::new()));
                                }
                                JFObject::Dictionary(ref mut vv) => {
                                    let key = NP_Error::unwrap(d_chain.last())?.clone();
                                    vv.insert(key, JFObject::Dictionary(JFMap::new()));
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
                            JFObject::Null => {
                                *ret = JFObject::Dictionary(JFMap::new());
                                true
                            }
                            _ => false,
                        };

                        if !is_root {
                            let a_nest = 0i64;
                            let d_nest = 0i64;
                            let log: String = "".to_owned();
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::Dictionary(JFMap::new()));
                                    }
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::Dictionary(JFMap::new()));
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::True);
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::False);
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::Null);
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
                            fn func(v: &mut JFObject,
                                    value: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {

                                match *v {
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        let mut value = NP_Error::unwrap(value)?;
                                        NP_Error::unwrap(value.pop())?;
                                        value = value.trim().to_string();
                                        match value.find('.') {
                                            Some(_) => vv.insert(key, JFObject::Float(f64::from_str(&value.clone())?)) ,
                                            None    => vv.insert(key, JFObject::Integer(i64::from_str(&value.clone())?)),
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::True);
                                    }
                                    JFObject::Dictionary(ref mut vv) => {

                                        let key = NP_Error::unwrap(d_chain.last())?.clone();

                                        vv.insert(key, JFObject::True);

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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::False);
                                    }
                                    JFObject::Dictionary(ref mut vv) => {

                                        let key = NP_Error::unwrap(d_chain.last())?.clone();

                                        vv.insert(key, JFObject::False);

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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    d_chain: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::Null);
                                    }
                                    JFObject::Dictionary(ref mut vv) => {
                                        let key = NP_Error::unwrap(d_chain.last())?.clone();
                                        vv.insert(key, JFObject::Null);
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
                            fn func(v: &mut JFObject,
                                    _: Option<String>,
                                    _: Vec<i64>,
                                    _: Vec<String>,
                                    _: i64,
                                    _: i64,
                                    _: char) -> Result<(), NP_Error> {
                                match *v {
                                    JFObject::Array(ref mut vv) => {
                                        vv.push(JFObject::Null);
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
                        fn func(v: &mut JFObject,
                                value: Option<String>,
                                _: Vec<i64>,
                                d_chain: Vec<String>,
                                _: i64,
                                _: i64,
                                _: char) -> Result<(), NP_Error> {
                            match *v {
                                JFObject::Array(ref mut vv) => {
                                    let mut new_num = NP_Error::unwrap(value)?.clone();
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => {
                                            vv.push(JFObject::Float(f64::from_str(&new_num)?))
                                        }
                                        None => {
                                            vv.push(JFObject::Integer(i64::from_str(&new_num)?))
                                        }
                                    };

                                }
                                JFObject::Dictionary(ref mut vv) => {

                                    let key = NP_Error::unwrap(d_chain.last())?.clone();

                                    let mut new_num = NP_Error::unwrap(value)?.clone();
                                    NP_Error::unwrap(new_num.pop())?;
                                    new_num = new_num.trim().to_string();

                                    match new_num.find('.') {
                                        Some(_) => {
                                            vv.insert(key,
                                                      JFObject::Float(f64::from_str(&new_num)?))
                                        }
                                        None => {
                                            vv.insert(key,
                                                      JFObject::Integer(i64::from_str(&new_num)?))
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
                                fn func(v: &mut JFObject,
                                        value: Option<String>,
                                        _: Vec<i64>,
                                        d_chain: Vec<String>,
                                        _: i64,
                                        _: i64,
                                        _: char) -> Result<(), NP_Error> {

                                    match *v {
                                        JFObject::Dictionary(ref mut vv) => {
                                            let key = NP_Error::unwrap(d_chain.last())?.clone();
                                            let mut value = NP_Error::unwrap(value)?;
                                            NP_Error::unwrap(value.pop())?;
                                            vv.insert(key, JFObject::String(value.clone()));
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
                                    JFObject::Null => {
                                        *ret = JFObject::String(string.clone());
                                        true
                                    }
                                    _ => false,
                                };

                                if !is_root {
                                    let a_nest = 0i64;
                                    let d_nest = 0i64;
                                    let log: String = "".to_owned();
                                    fn func(v: &mut JFObject,
                                            value: Option<String>,
                                            _: Vec<i64>,
                                            _: Vec<String>,
                                            _: i64,
                                            _: i64,
                                            _: char) -> Result<(), NP_Error> {
                                        match *v {
                                            JFObject::Array(ref mut vv) => {
                                                vv.push(JFObject::String(NP_Error::unwrap(value)?
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
                                fn func(v: &mut JFObject,
                                        value: Option<String>,
                                        _: Vec<i64>,
                                        d_chain: Vec<String>,
                                        _: i64,
                                        _: i64,
                                        _: char) -> Result<(), NP_Error> {

                                    match *v {
                                        JFObject::Dictionary(ref mut vv) => {
                                            let key = NP_Error::unwrap(d_chain.last())?.clone();
                                            let mut value = NP_Error::unwrap(value)?;
                                            NP_Error::unwrap(value.pop())?;
                                            vv.insert(key, JFObject::String(value.clone()));
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
                                    JFObject::Null => {
                                        *ret = JFObject::String(string.clone());
                                        true
                                    }
                                    _ => false,
                                };

                                if !is_root {
                                    let a_nest = 0i64;
                                    let d_nest = 0i64;
                                    let log: String = "".to_owned();
                                    fn func(v: &mut JFObject,
                                            value: Option<String>,
                                            _: Vec<i64>,
                                            _: Vec<String>,
                                            _: i64,
                                            _: i64,
                                            _: char) -> Result<(), NP_Error> {
                                        match *v {
                                            JFObject::Array(ref mut vv) => {
                                                vv.push(JFObject::String(NP_Error::unwrap(value)?
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