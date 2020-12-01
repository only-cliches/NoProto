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

    println!("====== ENCODE BENCHMARK ======");
    
    no_proto_encode().unwrap();
    flatbuffers_encode();
    protobuf_encode();

}


pub fn no_proto_encode() -> Result<(), NP_Error> {

    let user_factory = NP_Factory::new(r#"{
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
    }"#)?;

    let start = SystemTime::now();

    for _x in 0..LOOPS {

        let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));

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

        assert_eq!(new_buffer.close().len(), 417);

        // let bytes = new_buffer.close();
    }

    let time = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("NoProto:     {:?} size: {} zlib: {}", time, 417, 325);

    Ok(())
}

fn flatbuffers_encode() {

    let start = SystemTime::now();

    for _x in 0..LOOPS {

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
        assert_eq!(fbb.finished_data().to_vec().len(), 336);
    }

    let time = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("Flatbuffers: {:?} size: {} zlib: {}", time, 336, 214);
}

fn protobuf_encode() {

    let start = SystemTime::now();

    for x in 0..LOOPS {
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
        assert_eq!(bytes.len(), 220);
    }

    let time = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("PBuffers:    {:?} size: {} zlib: {}", time, 220, 163);
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