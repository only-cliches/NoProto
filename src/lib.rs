
use std::result;
use json::*;
use std::collections::HashMap;
use std::ops::{ Index, IndexMut, Deref };

pub mod pointer;
use pointer::*;



fn main() {
    let mut xx: NoProtoPointer = 42.into();

    xx.set("key", 42.into());

    // let x: Option<i64> = xx.into();
    let y: Option<i64> = (&xx["value"]).into();
    // let g: i64 = y.into().unwrap();
}