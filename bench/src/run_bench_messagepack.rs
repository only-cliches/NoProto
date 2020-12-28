use crate::LOOPS;
use messagepack_rs::{deserializable::Deserializable, serializable::Serializable, value::Value};
use std::io::{BufReader, Cursor};
use std::collections::BTreeMap;


use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};


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

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 296);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)   
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
       
        let mut vector = Vec::new();

        for x in 0..3 {

            let mut bar = BTreeMap::new();
            bar.insert(String::from("time"), Value::from(123456 + (x as i32)));
            bar.insert(String::from("ratio"), Value::from(3.14159 + (x as f32)));
            bar.insert(String::from("size"), Value::from(10000 + (x as u16)));

            let mut foobar = BTreeMap::new();
            foobar.insert(String::from("name"), Value::from("Hello, World!"));
            foobar.insert(String::from("sibling"), Value::from(bar));
            foobar.insert(String::from("rating"), Value::from(3.1415432432445543543 + (x as f64)));
            foobar.insert(String::from("postfix"), Value::from("!".as_bytes()[0]));

            vector.push(Value::from(foobar));
        }

        let mut foobarcontainer = BTreeMap::new();
        foobarcontainer.insert(String::from("fruit"), Value::from(2u8));
        foobarcontainer.insert(String::from("initialized"), Value::from(true));
        foobarcontainer.insert(String::from("location"), Value::from("http://arstechnica.com"));
        foobarcontainer.insert(String::from("list"), Value::from(vector));

        Value::from(foobarcontainer).serialize().unwrap()
    }



    pub fn update_bench(base: u128) -> String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut container = Value::deserialize(&mut BufReader::new(Cursor::new(buffer.clone()))).unwrap();

            match &mut container {
                Value::Map(foobarcontainer) => {
                    if let Value::Array(list) = foobarcontainer.get_mut("list").unwrap() {
                        list.iter_mut().enumerate().for_each(|(x, value)| {
                            if x == 0 {
                                if let Value::Map(foobar) = value {
                                    foobar.insert(String::from("name"), Value::from("bob"));
                                   
                                } else { panic!() }
                            }
                        });
                    } else { panic!() }
                },
                _ => panic!()
            }

            assert_eq!(container.serialize().unwrap().len(), 286);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)

    }

    pub fn decode_one_bench(base: u128) -> String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();


        for _x in 0..LOOPS {
            let container = Value::deserialize(&mut BufReader::new(Cursor::new(buffer.clone()))).unwrap();

            match &container {
                Value::Map(foobarcontainer) => {
                    if let Value::String(location) = foobarcontainer.get("location").unwrap() {
                        assert_eq!(location, &String::from("http://arstechnica.com"));
                    } else { panic!() }
                },
                _ => panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));    
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String  {
        
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        let hello_world = String::from("Hello, world!");
        let ars_technica = String::from("http://arstechnica.com");


        for _x in 0..LOOPS {
            let container = Value::deserialize(&mut BufReader::new(Cursor::new(buffer.clone()))).unwrap();

            match &container {
                Value::Map(foobarcontainer) => {
                    if let Value::String(location) = foobarcontainer.get("location").unwrap() {
                        assert_eq!(location, &ars_technica);
                    } else { panic!() }
                    if let Value::UInt8(fruit) = foobarcontainer.get("fruit").unwrap() {
                        assert_eq!(fruit, &2u8);
                    } else { panic!() }
                    if let Value::Bool(init) = foobarcontainer.get("initialized").unwrap() {
                        assert_eq!(init, &true);
                    } else { panic!() }
                    let mut loops = 0;
                    if let Value::Array(list) = foobarcontainer.get("list").unwrap() {
                        list.iter().enumerate().for_each(|(x, value)| {
                            loops += 1;
                            if let Value::Map(foobar) = value {
                                if let Value::String(name) = foobar.get("name").unwrap() {
                                    assert_eq!(name, &hello_world);
                                } else { panic!() }
                                if let Value::Float64(rating) = foobar.get("rating").unwrap() {
                                    assert_eq!(rating, &(3.1415432432445543543 + (x as f64)));
                                } else { panic!() }
                                if let Value::UInt8(postfix) = foobar.get("postfix").unwrap() {
                                    assert_eq!(postfix, &"!".as_bytes()[0]);
                                } else { panic!() }
                                if let Value::Map(bar) = foobar.get("sibling").unwrap() {
                                    if let Value::UInt8(time) = bar.get("time").unwrap() {
                                        assert_eq!(time, &(64 + x as u8));
                                    } else { panic!(); }
                                    if let Value::Float32(ratio) = bar.get("ratio").unwrap() {
                                        assert_eq!(ratio, &(3.14159 + (x as f32)));
                                    } else { panic!() }
                                    if let Value::UInt16(size) = bar.get("size").unwrap() {
                                        assert_eq!(size, &(10000 + (x as u16)));
                                    } else { panic!() }
                                } else { panic!() }
                            } else { panic!() }
                        });
                    } else { panic!() }
                    assert!(loops == 3);
                },
                _ => panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
