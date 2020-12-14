use crate::LOOPS;
use crate::bench_fb::benchfb::get_root_as_foo_bar_container;
use crate::bench_fb::benchfb::FooBarContainerArgs as FooBarContainerArgsFB;
use crate::bench_fb::benchfb::FooBarContainer as FooBarContainerFB;
use crate::bench_fb::benchfb::FooBarArgs as FooBarArgsFB;
use crate::bench_fb::benchfb::FooBar as FooBarFB;
use crate::bench_fb::benchfb::Bar as BarFB;
use crate::bench_fb::benchfb::Foo as FooFB;
use crate::bench_fb::benchfb::Enum as EnumFB;




use flatbuffers::FlatBufferBuilder;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};


pub struct FlatBufferBench();

impl FlatBufferBench {

    pub fn size_bench() {

        let encoded = Self::encode_single();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Flatbuffers: size: {}b, zlib: {}b", encoded.len(), compressed.len());
    }

    pub fn encode_bench() {
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single();
            assert_eq!(buffer.len(), 336);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flatbuffers: {:?}", time);        
    }

    #[inline(always)]
    fn encode_single() -> Vec<u8> {
        let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
        let mut vector = Vec::new();

        for x in 0..3 {
            let foo = FooFB::new(0xABADCAFEABADCAFE + (x as u64), 1000 + (x as i16), "@".as_bytes()[0] as i8, 10000 + (x as u32));
            let bar = BarFB::new(&foo, 123456 + (x as i32), 3.14159 + (x as f32), 10000 + (x as u16));
            let name = fbb.create_string("Hello, World!");
            let foobar_args = FooBarArgsFB { name: Some(name), sibling: Some(&bar), rating:  3.1415432432445543543 + (x as f64), postfix:  "!".as_bytes()[0]};
            let foobar = FooBarFB::create(&mut fbb, &foobar_args);
            vector.push(foobar);
        }

        let location = fbb.create_string("http://arstechnica.com");
        let foobarvec = fbb.create_vector(&vector[..]);
        let foobarcontainer_args = FooBarContainerArgsFB { fruit: EnumFB::Apples, initialized: true, location: Some(location), list: Some(foobarvec) };
        let foobarcontainer = FooBarContainerFB::create(&mut fbb, &foobarcontainer_args);

        fbb.finish(foobarcontainer, None);

        fbb.finished_data().to_vec()
    }



    pub fn update_bench()  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = get_root_as_foo_bar_container(&buffer[..]);

            let mut fbb: FlatBufferBuilder = FlatBufferBuilder::new();
            let mut vector = Vec::new();

            container.list().unwrap().iter().enumerate().for_each(|(idx, foobar)| {

                let old_bar = foobar.sibling().unwrap();
                let old_foo = old_bar.parent();

                let foo = FooFB::new(old_foo.id(), old_foo.count(), old_foo.prefix(), old_foo.length());
                let bar = BarFB::new(&foo, old_bar.time(), old_bar.ratio(), old_bar.size_());
                let name = if idx == 0 { // our update
                    fbb.create_string("bob")
                } else {
                    fbb.create_string(foobar.name().unwrap())
                };
                let foobar_args = FooBarArgsFB { name: Some(name), sibling: Some(&bar), rating:  foobar.rating(), postfix: foobar.postfix()};
                let foobar = FooBarFB::create(&mut fbb, &foobar_args);
                vector.push(foobar);
            });
    
            let location = fbb.create_string(container.location().unwrap());
            let foobarvec = fbb.create_vector(&vector[..]);
            let foobarcontainer_args = FooBarContainerArgsFB { fruit: container.fruit(), initialized: container.initialized(), location: Some(location), list: Some(foobarvec) };
            let foobarcontainer = FooBarContainerFB::create(&mut fbb, &foobarcontainer_args);
    
            fbb.finish(foobarcontainer, None);
    
            let finished = fbb.finished_data().to_vec();

            assert_eq!(finished.len(), 320);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flatbuffers: {:?}", time);      

    }

    pub fn decode_one_bench()  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = get_root_as_foo_bar_container(&buffer[..]);
            assert_eq!(container.location(), Some("http://arstechnica.com"));
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flatbuffers: {:?}", time);      

    }

    pub fn decode_bench()  {
        let buffer = Self::encode_single();

        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let container = get_root_as_foo_bar_container(&buffer[..]);

            container.list().unwrap().iter().enumerate().for_each(|(x, foobar)| {

                let old_bar = foobar.sibling().unwrap();
                let old_foo = old_bar.parent();

                assert_eq!(old_foo.id(), 0xABADCAFEABADCAFE + (x as u64));
                assert_eq!(old_foo.count(), 1000 + (x as i16));
                assert_eq!(old_foo.prefix(), "@".as_bytes()[0] as i8);
                assert_eq!(old_foo.length(), 10000 + (x as u32));

                assert_eq!(old_bar.time(), 123456 + (x as i32));
                assert_eq!(old_bar.ratio(), 3.14159 + (x as f32));
                assert_eq!(old_bar.size_(), 10000 + (x as u16));

                assert_eq!(foobar.name(), Some("Hello, World!"));
                assert_eq!(foobar.rating(), 3.1415432432445543543 + (x as f64));
                assert_eq!(foobar.postfix(), "!".as_bytes()[0]);
            });

            assert_eq!(container.location(), Some("http://arstechnica.com"));
            assert_eq!(container.fruit(), EnumFB::Apples);
            assert_eq!(container.initialized(), true);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Flatbuffers: {:?}", time);      

    }
}
