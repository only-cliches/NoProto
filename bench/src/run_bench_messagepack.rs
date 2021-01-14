use crate::LOOPS;

use std::io::{BufReader, Cursor};
use std::collections::BTreeMap;


use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};

use std::collections::HashMap;
use rmp::{encode, decode};
use rmpv::{ValueRef::*, decode::read_value_ref};
use rmpv::encode::write_value_ref;

pub struct MessagePackBench();

impl MessagePackBench {

    pub fn size_bench() -> (usize, usize) {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("MessagePack: size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> std::string::String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 311);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)   
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
       
        let mut vector = Vec::new();

        for x in 0..3 {

            let bar = Map(vec![
                (String("time".into()), Integer((123456 + (x as i32)).into())),
                (String("ratio".into()), F32((3.14159 + (x as f32)).into())),
                (String("size".into()), Integer((10000 + (x as u16)).into()))
            ]);

            let foobar = Map(vec![
                (String("name".into()), String("Hello, World!".into())),
                (String("sibling".into()), bar),
                (String("rating".into()), F64((3.1415432432445543543 + (x as f64)).into())),
                (String("postfix".into()), String("!".into()))
            ]);

            vector.push(foobar);
        }

        let value = Map(vec![
            (String("fruit".into()), Integer(2u8.into())),
            (String("initialized".into()), Boolean(true)),
            (String("location".into()), String("http://arstechnica.com".into())),
            (String("list".into()), Array(vector))
        ]);

        let mut bytes: Vec<u8> = Vec::new();

        write_value_ref(&mut bytes, &value).unwrap();

        bytes
    }



    pub fn update_bench(base: u128) -> std::string::String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut container = read_value_ref(&mut &buffer[..]).unwrap().to_owned();

            match &mut container {
                rmpv::Value::Map(foobarcontainer) => {
                    if let rmpv::Value::Array(list) = Self::find_mut(foobarcontainer, "list") {
                        list.iter_mut().enumerate().for_each(|(x, value)| {
                            if x == 0 {
                                if let rmpv::Value::Map(foobar) = value {
                                    let value = Self::find_mut(foobar, "name");
                                    *value = rmpv::Value::String("bob".into());
                                   
                                } else { panic!() }
                            }
                        });
                    } else { panic!() }
                },
                _ => panic!()
            }

            let mut bytes: Vec<u8> = Vec::new();

            rmpv::encode::write_value(&mut bytes, &container).unwrap();

            assert_eq!(bytes.len(), 301);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> std::string::String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        let location = String("location".into());
        let url = String("http://arstechnica.com".into());

        for _x in 0..LOOPS {
            let container = read_value_ref(&mut &buffer[..]).unwrap();

            match &container {
                Map(foobarcontainer) => {
                    let location = foobarcontainer.iter().position(|(key, _value)| { key == &location }).unwrap();
                    assert_eq!(&foobarcontainer[location].1, &url);
                },
                _ => panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));    
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    pub fn find<'find>(container: &'find Vec<(rmpv::ValueRef, rmpv::ValueRef)>, key: &str) -> &'find rmpv::ValueRef<'find> {
        let k = String(key.into());
        let idx = container.iter().position(|(key, _value)| { key == &k }).unwrap();
        &container[idx].1
    }

    #[inline(always)]
    pub fn find_mut<'find>(container: &'find mut Vec<(rmpv::Value, rmpv::Value)>, key: &str) -> &'find mut rmpv::Value {
        let k = rmpv::Value::String(key.into());
        let idx = container.iter().position(|(key, _value)| { key == &k }).unwrap();
        &mut container[idx].1
    }

    pub fn decode_bench(base: u128) -> std::string::String  {
        
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        let hello_world = String("Hello, World!".into());
        let ars_technica = String("http://arstechnica.com".into());


        for _x in 0..LOOPS {
            let container = read_value_ref(&mut &buffer[..]).unwrap();

            match &container {
                Map(foobarcontainer) => {
                    assert_eq!(Self::find(foobarcontainer, "location"), &ars_technica);
                    assert_eq!(Self::find(foobarcontainer, "fruit"), &Integer(2u8.into()));
                    assert_eq!(Self::find(foobarcontainer, "initialized"), &Boolean(true));

                    let mut loops = 0;
                    if let Array(list) = Self::find(foobarcontainer, "list") {
                        list.iter().enumerate().for_each(|(x, value)| {
                            loops += 1;
 
                            if let Map(foobar) = value {
                                assert_eq!(Self::find(foobar, "name"), &hello_world);
                                assert_eq!(Self::find(foobar, "rating"), &F64((3.1415432432445543543 + (x as f64)).into()));
                                assert_eq!(Self::find(foobar, "postfix"), &String("!".into()));

                                if let Map(bar) = Self::find(foobar, "sibling") {
                                    assert_eq!(Self::find(bar, "time"), &Integer((123456 + (x as i32)).into()));
                                    assert_eq!(Self::find(bar, "ratio"), &F32((3.14159 + (x as f32)).into()));
                                    assert_eq!(Self::find(bar, "size"), &Integer((10000 + (x as u16)).into()));
                                } else { panic!() }
                            } else { panic!() }
                        });
                    } else {
                        panic!()
                    }
                    assert!(loops == 3);
                },
                _ => panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
