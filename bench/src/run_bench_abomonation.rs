use crate::LOOPS;


use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
use abomonation::{encode, decode};
use abomonation_derive::*;

#[derive(Abomonation, PartialEq, Eq, Debug, Clone)]
enum Fruit {
    Apples, Pears, Bananas
}

#[derive(Abomonation, PartialEq, Debug, Clone)]
struct Bar {
  time: i32,
  ratio: f32,
  size: u16
}

#[derive(Abomonation, PartialEq, Debug, Clone)]
struct FooBar {
  sibling: Bar,
  name: String,
  rating: f64,
  postfix: char
}

#[derive(Abomonation, PartialEq, Debug, Clone)]
struct FooBarContainer {
  list: Vec<FooBar>,
  initialized: bool,
  fruit: Fruit, 
  location: String
}

pub struct AbomBench();

impl AbomBench {

    pub fn size_bench() -> (usize, usize) {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Abomonation: size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 261);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Abomonation: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
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
                name: String::from("Hello, world!"),
                rating: 3.1415432432445543543 + (x as f64),
                postfix: '!'
            };
            vector.push(foobar);
        }

        let foobar_c = FooBarContainer {
            location: String::from("http://arstechnica.com"),
            fruit: Fruit::Apples,
            initialized: true,
            list: vector
        };

        let mut bytes = Vec::new();

        unsafe { encode(&foobar_c, &mut bytes).unwrap(); };

        bytes
    }

    pub fn update_bench(base: u128) -> String {
        let mut buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            if let Some((result, _remaining)) = unsafe { decode::<FooBarContainer>(&mut buffer) } {
                let mut result2 = result.clone();
                result2.list[0].name = String::from("bob");

                let mut bytes = Vec::new();

                unsafe { encode(&result2, &mut bytes).unwrap() };

                assert_eq!(bytes.len(), 251);
            } else {
                panic!()
            }           
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Abomonation: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String {
        let mut buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            if let Some((result, _remaining)) = unsafe { decode::<FooBarContainer>(&mut buffer) } {
                assert_eq!(result.location, "http://arstechnica.com");
            } else {
                panic!()
            }
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Abomonation: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        let mut buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let decoded: &FooBarContainer = if let Some((result, _remaining)) = unsafe { decode::<FooBarContainer>(&mut buffer) } { result } else { panic!() };

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
        println!("Abomonation: {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
