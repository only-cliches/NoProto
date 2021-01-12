use crate::LOOPS;

use std::{io::prelude::*};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
use rawbson::{
    DocRef,
    DocBuf,
    elem,
};
use bson::*;

pub struct RawBSONBench();

impl RawBSONBench {

    pub fn size_bench() -> (usize, usize) {

        let (encoded, doc) = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Raw BSON:    size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let (buffer, doc) = Self::encode_single();
            assert_eq!(buffer.len(), 414);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Raw BSON:    {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    fn encode_single() -> (Vec<u8>, Document) {
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
        return (byte_array, bson_object)
    }


    pub fn update_bench(base: u128) -> String  {
        let (buffer, doc) = Self::encode_single();

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
        println!("Raw BSON:    {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String  {
        let (buffer, doc) = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = DocRef::new(&buffer[..]).unwrap();

            assert_eq!(container.get_str("location").unwrap().unwrap(), "http://arstechnica.com");
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Raw BSON:    {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String  {
        let (buffer, doc_ ) = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = DocRef::new(&buffer[..]).unwrap();

let new_doc = doc!{"list": ["allocations", "are", "slow"]};

// iterating through list of items
for (i, item ) in new_doc.get_array("list").unwrap().iter().enumerate() {
    // item is &Bson type
    match i {
        0 => assert_eq!(item.as_str().unwrap(), "allocations"),
        1 => assert_eq!(item.as_str().unwrap(), "are"),
        2 => assert_eq!(item.as_str().unwrap(), "slow"),
        _ => {}
    }
}

let mut byte_array : Vec<u8> = vec![];
new_doc.to_writer(&mut byte_array).unwrap();

let ref_dec = DocRef::new(&byte_array[..]).unwrap();

for (i, item ) in ref_dec.get_array("list").unwrap().iter().enumerate() {
    // item is &ArrayRef?  This code doesn't work....
}

// instead you seem to have to do something like this...
let list = ref_dec.get_array("list").unwrap().unwrap();

for i in 0..3 { // how would I get the array length if I didn't know it?
    match i {
        0 => assert_eq!(list.get_str(i).unwrap().unwrap(), "allocations"),
        1 => assert_eq!(list.get_str(i).unwrap().unwrap(), "are"),
        2 => assert_eq!(list.get_str(i).unwrap().unwrap(), "slow"),
        _ => {}
    }
}


            assert_eq!(container.get_str("location").unwrap().unwrap(), "http://arstechnica.com");
            assert_eq!(container.get_i32("fruit").unwrap().unwrap(), 2i32);
            assert_eq!(container.get_bool("initialized").unwrap().unwrap(), true);

            let mut loops = 0;

            let list = container.get_array("list").unwrap().unwrap();

            for x in 0..3 {
                loops += 1;
                let foobar = list.get_document(x).unwrap().unwrap();
                assert_eq!(foobar.get_str("name").unwrap().unwrap(), "Hello, World!");
                assert_eq!(foobar.get_f64("rating").unwrap().unwrap(), 3.1415432432445543543 + (x as f64));
                assert_eq!(foobar.get_str("postfix").unwrap().unwrap(), "!");
                let sibling = foobar.get_document("sibling").unwrap().unwrap();
                assert_eq!(sibling.get_i32("time").unwrap().unwrap(), 123456 + (x as i32));
                assert_eq!(sibling.get_f64("ratio").unwrap().unwrap(), 3.14159f64);
                assert_eq!(sibling.get_i32("size").unwrap().unwrap(), 10000 + (x as i32));               
            }

            assert!(loops == 3);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Raw BSON:    {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));    
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
