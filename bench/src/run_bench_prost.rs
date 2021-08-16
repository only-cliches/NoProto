use crate::LOOPS;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};

use prost::*;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bar {
    #[prost(int32, required, tag="2")]
    pub time: i32,
    #[prost(float, required, tag="3")]
    pub ratio: f32,
    #[prost(uint32, required, tag="4")]
    pub size: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FooBar {
    #[prost(message, optional, tag="1")]
    pub sibling: ::core::option::Option<Bar>,
    #[prost(string, optional, tag="2")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(double, optional, tag="3")]
    pub rating: ::core::option::Option<f64>,
    #[prost(uint32, optional, tag="4")]
    pub postfix: ::core::option::Option<u32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FooBarContainer {
    /// 3 copies of the above
    #[prost(message, repeated, tag="1")]
    pub list: ::prost::alloc::vec::Vec<FooBar>,
    #[prost(bool, optional, tag="2")]
    pub initialized: ::core::option::Option<bool>,
    #[prost(enumeration="Enum", optional, tag="3")]
    pub fruit: ::core::option::Option<i32>,
    #[prost(string, optional, tag="4")]
    pub location: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Enum {
    Apples = 0,
    Pears = 1,
    Bananas = 2,
}


pub struct ProstBench();

impl ProstBench {


    pub fn size_bench() -> (usize, usize) {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Prost:       size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 154);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Prost:       {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
        let mut vector: Vec<FooBar> = Vec::new();

        for x in 0..3 {

            let bar = Bar {
                time: 123456 + (x as i32),
                ratio: 3.14159 + (x as f32),
                size: 10000 + (x as u32)
            };
            let foobar = FooBar {
                sibling: Some(bar),
                name: Some(String::from("Hello, world!")),
                rating: Some(3.1415432432445543543 + (x as f64)),
                postfix: Some("!".as_bytes()[0] as u32)
            };
            vector.push(foobar);
        }

        let foobar_c = FooBarContainer {
            location: Some(String::from("http://arstechnica.com")),
            fruit: Some(Enum::Apples as i32),
            initialized: Some(true),
            list: vector
        };

        let mut bytes = Vec::new();
        foobar_c.encode(&mut bytes).unwrap();
        bytes
    }

    pub fn update_bench(base: u128) -> String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut decoded: FooBarContainer = FooBarContainer::decode(&buffer[..]).unwrap();

            decoded.list[0].name = Some(String::from("bob"));

            let mut bytes = Vec::new();
            decoded.encode(&mut bytes).unwrap();
            assert_eq!(bytes.len(), 144);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Prost:       {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String {
        let start = SystemTime::now();

        let buffer = Self::encode_single();

        let value = Some(String::from("http://arstechnica.com"));

        for _x in 0..LOOPS {
            let decoded: FooBarContainer = FooBarContainer::decode(&buffer[..]).unwrap();
            assert_eq!(decoded.location, value);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Prost:       {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        let hello_world = Some(String::from("Hello, world!"));
        let ars_technica = Some(String::from("http://arstechnica.com"));

        for _x in 0..LOOPS {
            let decoded: FooBarContainer = FooBarContainer::decode(&buffer[..]).unwrap();

            let mut loops = 0;

            decoded.list.iter().enumerate().for_each(|(x, foobar)| {
                loops += 1;
                match foobar.sibling.as_ref() {
                    Some(old_bar) => {
                        assert_eq!(old_bar.time, 123456 + (x as i32));
                        assert_eq!(old_bar.ratio, 3.14159 + (x as f32));
                        assert_eq!(old_bar.size, 10000 + (x as u32));
                    },
                    None => panic!()
                }

                assert_eq!(foobar.name, hello_world);
                assert_eq!(foobar.rating, Some(3.1415432432445543543 + (x as f64)));
                assert_eq!(foobar.postfix, Some("!".as_bytes()[0] as u32));
            });

            assert!(loops == 3);

            assert_eq!(decoded.location, ars_technica);
            assert_eq!(decoded.fruit, Some(Enum::Apples as i32));
            assert_eq!(decoded.initialized, Some(true));
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Prost:       {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

}