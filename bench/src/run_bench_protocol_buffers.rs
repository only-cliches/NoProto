use crate::LOOPS;
use crate::bench_pb::FooBarContainer;
use crate::bench_pb::FooBar;
use crate::bench_pb::Bar;
use crate::bench_pb::Foo;
use crate::bench_pb::Enum;
use crate::protobuf::Message;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct ProtocolBufferBench();

impl ProtocolBufferBench {


    pub fn size_bench() {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("PBuffers:    size: {}b, zlib: {}b", encoded.len(), compressed.len());
    }

    pub fn encode_bench() {
        let start = SystemTime::now();

        for x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 220);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("PBuffers:    {:?}", time);
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
        let mut foobarcontainer = FooBarContainer::new();
        let mut foobarlist: protobuf::RepeatedField<FooBar> = protobuf::RepeatedField::new();
        for y in 0..3 {
            let mut foobar = FooBar::new();
            foobar.set_name(String::from("Hello, World!"));
            foobar.set_rating(3.1415432432445543543 + y as f64);
            foobar.set_postfix("!".as_bytes()[0] as u32);
            let mut bar = Bar::new();
            bar.set_time(123456 + y as i32);
            bar.set_ratio(3.14159f32 + y as f32);
            bar.set_size(10000 + y as u32);
            let mut foo = Foo::new();
            foo.set_id(0xABADCAFEABADCAFE );
            foo.set_count(10000 );
            foo.set_prefix("@".as_bytes()[0] as i32);
            foo.set_length(1000000 );
            bar.set_parent(foo);
            foobar.set_sibling(bar);
            foobarlist.push(foobar);
        }

        foobarcontainer.set_location(String::from("http://arstechnica.com"));
        foobarcontainer.set_initialized(true);
        foobarcontainer.set_fruit(Enum::Apples);
        foobarcontainer.set_list(foobarlist);

        let mut bytes: Vec<u8> = Vec::new();
        let mut message = protobuf::CodedOutputStream::vec(&mut bytes);
        foobarcontainer.compute_size();
        foobarcontainer.write_to_with_cached_sizes(&mut message).unwrap();
        message.flush().unwrap();

        bytes
    }

    pub fn update_bench()  {
        let start = SystemTime::now();

        let buffer = Self::encode_single();

        for x in 0..LOOPS {
            let old_foo_bar: FooBarContainer = protobuf::parse_from_bytes(&buffer).unwrap();


            let mut foobarcontainer = FooBarContainer::new();
            let mut foobarlist: protobuf::RepeatedField<FooBar> = protobuf::RepeatedField::new();

            old_foo_bar.get_list().iter().enumerate().for_each(|(idx, old_foo_b)| {

                let mut foobar = FooBar::new();
                if idx == 0 { // our update
                    foobar.set_name(String::from("bob"));
                } else {
                    foobar.set_name(old_foo_b.get_name().to_string());
                }
                
                foobar.set_rating(old_foo_b.get_rating());
                foobar.set_postfix(old_foo_b.get_postfix());

                let old_bar = old_foo_b.get_sibling();

                let mut bar = Bar::new();
                bar.set_time(old_bar.get_time());
                bar.set_ratio(old_bar.get_ratio());
                bar.set_size(old_bar.get_size());

                let old_foo = old_bar.get_parent();

                let mut foo = Foo::new();
                foo.set_id(old_foo.get_id());
                foo.set_count(old_foo.get_count());
                foo.set_prefix(old_foo.get_prefix());
                foo.set_length(old_foo.get_length());
                bar.set_parent(foo);
                foobar.set_sibling(bar);
                foobarlist.push(foobar);
            });

            foobarcontainer.set_location(old_foo_bar.get_location().to_string());
            foobarcontainer.set_initialized(old_foo_bar.get_initialized());
            foobarcontainer.set_fruit(old_foo_bar.get_fruit());
            foobarcontainer.set_list(foobarlist);
            
            let mut bytes: Vec<u8> = Vec::new();
            let mut message = protobuf::CodedOutputStream::vec(&mut bytes);
            foobarcontainer.compute_size();
            foobarcontainer.write_to_with_cached_sizes(&mut message).unwrap();
            message.flush().unwrap();

            assert_eq!(bytes.len(), 210);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("PBuffers:    {:?}", time);
    }

    pub fn decode_one_bench() {
        let start = SystemTime::now();

        let buffer = Self::encode_single();

        for x in 0..LOOPS {
            let old_foo_bar: FooBarContainer = protobuf::parse_from_bytes(&buffer).unwrap();
            assert_eq!(old_foo_bar.get_location(), "http://arstechnica.com");
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("PBuffers:    {:?}", time);
    }

    pub fn decode_bench()  {
        let start = SystemTime::now();

        let buffer = Self::encode_single();

        for x in 0..LOOPS {
            let old_foo_bar: FooBarContainer = protobuf::parse_from_bytes(&buffer).unwrap();

            old_foo_bar.get_list().iter().enumerate().for_each(|(y, old_foo_b)| {

                assert_eq!(old_foo_b.get_name(), "Hello, World!");
                assert_eq!(old_foo_b.get_rating(), 3.1415432432445543543 + y as f64);
                assert_eq!(old_foo_b.get_postfix(), "!".as_bytes()[0] as u32);
                
                let old_bar = old_foo_b.get_sibling();
                assert_eq!(old_bar.get_time(), 123456 + y as i32);
                assert_eq!(old_bar.get_ratio(), 3.14159f32 + y as f32);
                assert_eq!(old_bar.get_size(), 10000 + y as u32);

                let old_foo = old_bar.get_parent();

                assert_eq!(old_foo.get_id(), 0xABADCAFEABADCAFE);
                assert_eq!(old_foo.get_count(), 10000 );
                assert_eq!(old_foo.get_prefix(), "@".as_bytes()[0] as i32);
                assert_eq!(old_foo.get_length(), 1000000 );

            });

            assert_eq!(old_foo_bar.get_location(), "http://arstechnica.com");
            assert_eq!(old_foo_bar.get_initialized(), true);
            assert_eq!(old_foo_bar.get_fruit(), Enum::Apples);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("PBuffers:    {:?}", time);
    }

}