use crate::LOOPS;


use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
use serde::{Serialize, Deserialize};
use flexbuffers::{self, Reader};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
enum Fruit {
    Apples, Pears, Bananas
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Bar {
  time: i32,
  ratio: f32,
  size: u16
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct FooBar<'fb> {
  sibling: Bar,
  name: &'fb str,
  rating: f64,
  postfix: char
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct FooBarContainer<'con> {
  list: Vec<FooBar<'con>>,
  initialized: bool,
  fruit: Fruit, 
  location: &'con str
}

pub struct FlexBench();

impl FlexBench {

    pub fn size_bench() -> (usize, usize) {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Flexbuffers: size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 490);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flexbuffers: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64) 
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {

        let mut vector: Vec<FooBar> = Vec::new();

        for x in 0..3 {

            let bar = Bar {
                time: 123456 + (x as i32),
                ratio: 3.14159 + (x as f32),
                size: 10000 + (x as u16)
            };
            let foobar = FooBar {
                sibling: bar,
                name: "Hello, world!",
                rating: 3.1415432432445543543 + (x as f64),
                postfix: '!'
            };
            vector.push(foobar);
        }

        let foobar_c = FooBarContainer {
            location: "http://arstechnica.com",
            fruit: Fruit::Apples,
            initialized: true,
            list: vector
        };

        let mut s = flexbuffers::FlexbufferSerializer::new();

        foobar_c.serialize(&mut s).unwrap();

        s.view().to_vec()
    }

    pub fn update_bench(base: u128) -> String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let reader = flexbuffers::Reader::get_root(&buffer[..]).unwrap();

            let mut decoded: FooBarContainer = FooBarContainer::deserialize(reader).unwrap().clone();

            decoded.list[0].name = "bob";

            let mut s = flexbuffers::FlexbufferSerializer::new();

            decoded.serialize(&mut s).unwrap();

            assert_eq!(s.view().to_vec().len(), 482);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flexbuffers: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let root = Reader::get_root(&buffer[..]).unwrap();
            let decoded = root.as_map();
            assert_eq!(decoded.idx("location").as_str(), "http://arstechnica.com");
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flexbuffers: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let reader = flexbuffers::Reader::get_root(&buffer[..]).unwrap();

            let decoded: FooBarContainer = FooBarContainer::deserialize(reader).unwrap();

            let mut loops = 0;

            decoded.list.iter().enumerate().for_each(|(x, foobar)| {
                loops += 1;
                let old_bar = &foobar.sibling;

                assert_eq!(old_bar.time, 123456 + (x as i32));
                assert_eq!(old_bar.ratio, 3.14159 + (x as f32));
                assert_eq!(old_bar.size, 10000 + (x as u16));

                assert_eq!(foobar.name, "Hello, world!");
                assert_eq!(foobar.rating, 3.1415432432445543543 + (x as f64));
                assert_eq!(foobar.postfix, '!');
            });

            assert!(loops == 3);

            assert_eq!(decoded.location, "http://arstechnica.com");
            assert_eq!(decoded.fruit, Fruit::Apples);
            assert_eq!(decoded.initialized, true);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flexbuffers: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
