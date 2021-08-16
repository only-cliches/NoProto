use crate::{LOOPS};


use std::io::{Write, prelude::*};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};

use rkyv::{Aligned, Archive, ArchiveBuffer, ArchiveWriter, Archived, Unarchive, Write as RkWrite, archived_ref, archived_value, archived_value_mut};


#[derive(Archive, PartialEq, Debug, Clone, Unarchive)]
struct Bar {
  time: i32,
  ratio: f32,
  size: u16
}

#[derive(Archive, PartialEq, Debug, Clone, Unarchive)]
struct FooBar {
  sibling: Bar,
  name: String,
  rating: f64,
  postfix: char
}

#[derive(Archive, PartialEq, Debug, Clone, Unarchive)]
struct FooBarContainer {
  list: Vec<FooBar>,
  initialized: bool,
  fruit: u8, 
  location: String
}

pub struct RkyvBench();

impl RkyvBench {

    pub fn size_bench() -> (usize, usize) {

        let (encoded, pos) = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Rkyv:        size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let (buffer, pos) = Self::encode_single();
            assert_eq!(buffer.len(), 180);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Rkyv:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64) 
    }

    #[inline(always)]
    fn encode_single() -> (Vec<u8>, usize) {

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
            fruit: 2,
            initialized: true,
            list: vector
        };

        let mut writer = ArchiveWriter::new(Vec::new());
        let pos = writer.archive(&foobar_c).expect("failed to archive test");
        (writer.into_inner(), pos)
    }

    pub fn update_bench(base: u128) -> String {
        let (buffer, pos) = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut decoded: FooBarContainer = unsafe { archived_value::<FooBarContainer>(&buffer[..], pos) }.unarchive();

            decoded.list[0].name = String::from("bob");

            let mut writer = ArchiveWriter::new(Vec::new());
            let pos = writer.archive(&decoded).expect("failed to archive test");

            assert_eq!(writer.into_inner().len(), 172);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Rkyv:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String {
        let (buffer, pos) = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let decoded = unsafe { archived_value::<FooBarContainer>(&buffer[..], pos) };
            assert_eq!(decoded.location, "http://arstechnica.com");
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Rkyv:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        let (buffer, pos) = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let decoded = unsafe { archived_value::<FooBarContainer>(&buffer[..], pos) };

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

            assert_eq!(decoded.location.as_str(), "http://arstechnica.com");
            assert_eq!(decoded.fruit, 2);
            assert_eq!(decoded.initialized, true);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Rkyv:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
