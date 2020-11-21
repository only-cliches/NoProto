use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::memory::NP_Size;
use no_proto::collection::table::NP_Table;
use no_proto::pointer::NP_Ptr;
use no_proto::here;
use no_proto::path;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime, UNIX_EPOCH};


fn main() -> Result<(), NP_Error> {

    let user_factory = NP_Factory::new(r#"{
        "type": "string"
    }"#)?;

    let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));
    new_buffer.set(&[], String::from("hello"))?;
    println!("{:?}", new_buffer.get::<String>(&[])?);

    /*
    let user_factory = NP_Factory::new(r#"{
        "type": "table",
        "columns": [
            ["list",   {"type": "list", "of": {
                "type": "table",
                "columns": [
                    ["name", {"type": "string"}],
                    ["rating", {"type": "float"}],
                    ["postfix", {"type": "string"}],
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

    for x in 0..1_000_000 {
        let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.set(&["initialized"], true)?;
        new_buffer.set(&["location"], String::from("https://arstechnica.com"))?;
        new_buffer.set(&["fruit"], 2u8)?;

        for x in 0..3 {
            new_buffer.reset_cursor();
            new_buffer.move_cursor(&["list", x.to_string().as_str()])?;
            new_buffer.set(&["name"], String::from("Hello, world!"))?;
            new_buffer.set(&["rating"], 3.1415432432445543543 + (x as f32))?;
            new_buffer.set(&["postfix"], String::from("!") + x.to_string().as_str())?;

            new_buffer.move_cursor(&["sibling"])?;
            new_buffer.set(&["time"], 123456 + (x as u32))?;
            new_buffer.set(&["ratio"], 3.14159 + (x as f32))?;
            new_buffer.set(&["size"], 10000 + (x as u16))?;

            new_buffer.move_cursor(&["parent"])?;
            new_buffer.set(&["id"], 0xABADCAFEABADCAFE + (x as u64))?;
            new_buffer.set(&["count"], 10000 + (x as u16))?;
            new_buffer.set(&["prefix"], String::from("@") + x.to_string().as_str())?;
            new_buffer.set(&["length"], 1000000 + (x as u32))?;
        }

        // let bytes = new_buffer.close();
    }

    let time = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("{:?}", time);*/

    Ok(())
}