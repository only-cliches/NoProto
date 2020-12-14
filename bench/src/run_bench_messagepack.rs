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

    pub fn size_bench() {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("MessagePack: size: {}b, zlib: {}b", encoded.len(), compressed.len());
    }

    pub fn encode_bench() {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 431);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:?}", time);        
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
       
        let mut vector = Vec::new();

        for x in 0..3 {
            let mut foo = BTreeMap::new();
            foo.insert(String::from("id"), Value::from(0xABADCAFEABADCAFE + (x as u64)));
            foo.insert(String::from("count"), Value::from(1000 + (x as i16)));
            foo.insert(String::from("prefix"), Value::from("@".as_bytes()[0] as i8));
            foo.insert(String::from("length"), Value::from(10000 + (x as u32)));

            let mut bar = BTreeMap::new();
            bar.insert(String::from("parent"), Value::from(foo));
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



    pub fn update_bench()  {
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

            assert_eq!(container.serialize().unwrap().len(), 421);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:?}", time);      

    }

    pub fn decode_one_bench()  {
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
        println!("MessagePack: {:?}", time);      

    }

    pub fn decode_bench()  {
        
        let buffer = Self::encode_single();

        let start = SystemTime::now();


        for _x in 0..LOOPS {
            let container = Value::deserialize(&mut BufReader::new(Cursor::new(buffer.clone()))).unwrap();

            match &container {
                Value::Map(foobarcontainer) => {
                    if let Value::String(location) = foobarcontainer.get("location").unwrap() {
                        assert_eq!(location, &String::from("http://arstechnica.com"));
                    } else { panic!() }
                    if let Value::UInt8(fruit) = foobarcontainer.get("fruit").unwrap() {
                        assert_eq!(fruit, &2u8);
                    } else { panic!() }
                    if let Value::Bool(init) = foobarcontainer.get("initialized").unwrap() {
                        assert_eq!(init, &true);
                    } else { panic!() }
                    if let Value::Array(list) = foobarcontainer.get("list").unwrap() {
                        list.iter().enumerate().for_each(|(x, value)| {
                            if let Value::Map(foobar) = value {
                                if let Value::String(name) = foobar.get("name").unwrap() {
                                    assert_eq!(name, &String::from("Hello, World!"));
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
                                    if let Value::Map(foo) = bar.get("parent").unwrap() {
                                        if let Value::UInt16(length) = foo.get("length").unwrap() {
                                            assert_eq!(length, &(10000 + x as u16));
                                        } else { panic!() }
                                        if let Value::UInt8(prefix) = foo.get("prefix").unwrap() {
                                            assert_eq!(prefix, &64);
                                        } else { panic!() }
                                        if let Value::Int8(count) = foo.get("count").unwrap() {
                                            assert_eq!(count, &(-24 + x as i8));
                                        } else { panic!() }
                                        if let Value::UInt64(id) = foo.get("id").unwrap() {
                                            assert_eq!(id, &(0xABADCAFEABADCAFE + (x as u64)));
                                        } else { panic!() }
                                    } else { panic!() }
                                } else { panic!() }
                            } else { panic!() }
                        });
                    } else { panic!() }
                },
                _ => panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("MessagePack: {:?}", time);      

    }
}
