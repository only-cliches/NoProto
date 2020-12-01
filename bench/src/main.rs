use crate::bench_generated::benchfb::get_root_as_foo_bar_container;
use crate::flatbuffers::Follow;
use crate::protobuf::Message;
use crate::bench_generated::benchfb::FooBarContainerArgs as FooBarContainerArgsFB;
use crate::bench_generated::benchfb::FooBarContainer as FooBarContainerFB;
use crate::bench_generated::benchfb::FooBarArgs as FooBarArgsFB;
use crate::bench_generated::benchfb::FooBar as FooBarFB;
use crate::bench_generated::benchfb::Bar as BarFB;
use crate::bench_generated::benchfb::Foo as FooFB;
use crate::bench_generated::benchfb::Enum as EnumFB;


use crate::bench_pb::FooBarContainer;
use crate::bench_pb::FooBar;
use crate::bench_pb::Bar;
use crate::bench_pb::Foo;
use crate::bench_pb::Enum;

use flatbuffers::FlatBufferBuilder;
use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::memory::NP_Size;
use no_proto::collection::table::NP_Table;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime, UNIX_EPOCH};

const LOOPS: usize = 1_000_000;

mod bench_generated;
mod bench_pb;
extern crate protobuf;
extern crate flatbuffers;
#[macro_use] 
extern crate json;

/*
1,000,000 iterations
0.4.2 - 144s
0.5.0 - 6s

*/

fn main() {

    println!("====== SIZE BENCHMARK ======");

    NoProtoBench::size_bench();
    FlatBufferBench::size_bench();
    ProtocolBufferBench::size_bench();

    println!("\n====== ENCODE BENCHMARK ======");
    
    NoProtoBench::encode_bench().unwrap();
    FlatBufferBench::encode_bench();
    ProtocolBufferBench::encode_bench();

    println!("\n====== UPDATE BENCHMARK ======");

    NoProtoBench::update_bench().unwrap();
    FlatBufferBench::update_bench();
    ProtocolBufferBench::update_bench();
}


struct NoProtoBench();

impl NoProtoBench {

    pub fn size_bench() {
        let factory = NoProtoBench::get_factory().unwrap();
        let encoded = Self::encode_single(&factory).unwrap();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("NoProto:     size: {}b, zlib: {}b", encoded.len(), compressed.len());
    }


    pub fn encode_bench() -> Result<(), NP_Error> {
        let factory = NoProtoBench::get_factory()?;

        let start = SystemTime::now();
    
        for _x in 0..LOOPS {
            let new_buffer = NoProtoBench::encode_single(&factory)?;
            assert_eq!(new_buffer.len(), 417);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:?}", time);
    
        Ok(())
    }

    pub fn update_bench() -> Result<(), NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for x in 0..LOOPS {
            let mut new_buff = factory.open_buffer(new_buffer.clone());

            new_buff.set(&["list", "0", "name"], "bob")?;

            assert_eq!(new_buff.close().len(), 417);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:?}", time);

        Ok(())
    }

    #[inline(always)]
    fn get_factory() -> Result<NP_Factory, NP_Error> {
        NP_Factory::new(r#"{
            "type": "table",
            "columns": [
                ["list",   {"type": "list", "of": {
                    "type": "table",
                    "columns": [
                        ["name", {"type": "string"}],
                        ["rating", {"type": "float"}],
                        ["postfix", {"type": "string", "size": 2}],
                        ["sibling", {"type": "table", "columns": [
                            ["time", {"type": "u32"}],
                            ["ratio", {"type": "float"}],
                            ["size", {"type": "u16"}],
                            ["parent", {"type": "table", "columns": [
                                ["id", {"type": "u64"}],
                                ["count", {"type": "u16"}],
                                ["prefix", {"type": "string", "size": 2}],
                                ["length", {"type": "u32"}]
                            ]}]
                        ]}]
                    ]
                }}],
                ["initialized", {"type": "bool"}],
                ["location", {"type": "string"}],
                ["fruit", {"type": "u8"}]
            ]
        }"#)
    }

    #[inline(always)]
    pub fn encode_single(factory: &NP_Factory) ->Result<Vec<u8>, NP_Error> {
        let mut new_buffer = factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.fast_insert("initialized", true)?;
        new_buffer.fast_insert("location", "https://arstechnica.com")?;
        new_buffer.fast_insert("fruit", 2)?;
    
        for x in 0..3 {
    
            new_buffer.cursor_to_root();
            new_buffer.move_cursor(&["list", x.to_string().as_str()])?;
            new_buffer.fast_insert("name", "Hello, world!")?;
            new_buffer.fast_insert("rating", 3.1415432432445543543 + (x as f32))?;
            new_buffer.fast_insert("postfix", "!")?;
    
            new_buffer.move_cursor(&["sibling"])?;
            new_buffer.fast_insert("time", 123456 + (x as u32))?;
            new_buffer.fast_insert("ratio", 3.14159 + (x as f32))?;
            new_buffer.fast_insert("size", 10000 + (x as u16))?;
    
            new_buffer.move_cursor(&["parent"])?;
            new_buffer.fast_insert("id", 0xABADCAFEABADCAFE + (x as u64))?;
            new_buffer.fast_insert("count", 10000 + (x as u16))?;
            new_buffer.fast_insert("prefix", "@")?;
            new_buffer.fast_insert("length", 1000000 + (x as u32))?;
            
        }
    
        Ok(new_buffer.close())
    }
}

struct FlatBufferBench();

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

        for x in 0..LOOPS {
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
}

struct ProtocolBufferBench();

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

}

/*
        // 0.4.2 API
        let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.set("initialized", true)?;
        new_buffer.set("location", String::from("https://arstechnica.com"))?;
        new_buffer.set("fruit", 2u8)?;

        for x in 0..3 {

            new_buffer.set(format!("list.{}.name", x).as_str(), String::from("Hello, world!"))?;
            new_buffer.set(format!("list.{}.rating", x).as_str(), 3.1415432432445543543 + (x as f32))?;
            new_buffer.set(format!("list.{}.postfix", x).as_str(), String::from("!"))?;

            new_buffer.set(format!("list.{}.sibling.time", x).as_str(), 123456 + (x as u32))?;
            new_buffer.set(format!("list.{}.sibling.ratio", x).as_str(), 3.14159 + (x as f32))?;
            new_buffer.set(format!("list.{}.sibling.size", x).as_str(), 10000 + (x as u16))?;

            new_buffer.set(format!("list.{}.sibling.parent.id", x).as_str(), 0xABADCAFEABADCAFE + (x as u64))?;
            new_buffer.set(format!("list.{}.sibling.parent.count", x).as_str(), 10000 + (x as u16))?;
            new_buffer.set(format!("list.{}.sibling.parent.prefix", x).as_str(), String::from("@"))?;
            new_buffer.set(format!("list.{}.sibling.parent.length", x).as_str(), 1000000 + (x as u32))?;
        }
*/