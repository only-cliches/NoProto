use crate::LOOPS;

use std::{io::prelude::*};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
use bson::*;


pub struct BSONBench();

impl BSONBench {

    pub fn size_bench() -> (usize, usize) {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("BSON:        size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 414);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("BSON:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
        let mut bson_object = doc!{
            "fruit": 2i32,
            "initialized": true,
            "location": "http://arstechnica.com",
            "list": []
        };

        for x in 0..3 {
            let list = bson_object.get_array_mut("list").unwrap();
            list.push(bson!({
                "name": "Hello, World!",
                "rating": 3.1415432432445543543 + (x as f64),
                "postfix": "!",
                "sibling": {
                    "time": 123456 + (x as i32),
                    "ratio": 3.14159f64,
                    "size": 10000 + (x as i32)
                }
            }));
        }


        let mut byte_array : Vec<u8> = vec![];
        bson_object.to_writer(&mut byte_array).unwrap();
        return byte_array
    }


    pub fn update_bench(base: u128) -> String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut container = Document::from_reader(&mut std::io::Cursor::new(buffer.clone())).unwrap();

            let list = container.get_array_mut("list").unwrap();
            let first_list = list[0].as_document_mut().unwrap();
            first_list.insert("name", "bob");

            let mut byte_array : Vec<u8> = vec![];
            container.to_writer(&mut byte_array).unwrap();

            assert_eq!(byte_array.len(), 404);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("BSON:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = Document::from_reader(&mut std::io::Cursor::new(buffer.clone())).unwrap();

            assert_eq!(container.get_str("location").unwrap(), "http://arstechnica.com");
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("BSON:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = Document::from_reader(&mut std::io::Cursor::new(buffer.clone())).unwrap();

            assert_eq!(container.get_str("location").unwrap(), "http://arstechnica.com");
            assert_eq!(container.get_i32("fruit").unwrap(), 2i32);
            assert_eq!(container.get_bool("initialized").unwrap(), true);

            let mut loops = 0;

            container.get_array("list").unwrap().iter().enumerate().for_each(|(x, bson)| {
                loops += 1;
                let foobar = bson.as_document().unwrap();
                assert_eq!(foobar.get_str("name").unwrap(), "Hello, World!");
                assert_eq!(foobar.get_f64("rating").unwrap(), 3.1415432432445543543 + (x as f64));
                assert_eq!(foobar.get_str("postfix").unwrap(), "!");
                let sibling = foobar.get_document("sibling").unwrap();
                assert_eq!(sibling.get_i32("time").unwrap(), 123456 + (x as i32));
                assert_eq!(sibling.get_f64("ratio").unwrap(), 3.14159f64);
                assert_eq!(sibling.get_i32("size").unwrap(), 10000 + (x as i32));
            });

            assert!(loops == 3);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("BSON:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));    
        format!("{:>5.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
