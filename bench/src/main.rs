use no_proto::error::NP_Error;
use no_proto::NP_Factory;
use no_proto::memory::NP_Size;
use no_proto::collection::table::NP_Table;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime, UNIX_EPOCH};

/*
1,000,000 iterations
0.4.2 - 105s
0.5.0 - 15s

*/

struct TEST<'test> {
    list: Vec<TEST_LIST<'test>>,
    initialized: bool,
    location: &'test str,
    fruit: u8
}

struct TEST_LIST<'test> {
    name: &'test str,
    rating: f32,
    postfix: &'test str,
    sibling: TEST_LIST_SIBLING<'test>
}

struct TEST_LIST_SIBLING<'test> {
    time: u32,
    ratio: f32,
    size: u16,
    parent: TEST_LIST_SIBLING_PARENT<'test>
}

struct TEST_LIST_SIBLING_PARENT<'test> {
    id: u64,
    count: u16,
    prefix: &'test str,
    length: u32
}

fn main() -> Result<(), NP_Error> {

    
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

    

    let mut example_data = TEST {
        initialized: true,
        location: "https://arstechnica.com",
        fruit: 2,
        list: Vec::new()
    };

    for x in 0..3 {
        example_data.list.push(TEST_LIST {
            name: "Hello, world!",
            rating: 3.1415432432445543543 + (x as f32),
            postfix: "!",
            sibling: TEST_LIST_SIBLING {
                time: 123456 + (x as u32),
                ratio: 3.14159 + (x as f32),
                size: 10000 + (x as u16),
                parent: TEST_LIST_SIBLING_PARENT {
                    id: 0xABADCAFEABADCAFE + (x as u64),
                    count: 10000 + (x as u16),
                    prefix: "@",
                    length: 1000000 + (x as u32)
                }
            }
        })
    }

    let start = SystemTime::now();

    for x in 0..1_000 {
/*
        // 0.4.2 API
        let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.set("initialized", true)?;
        new_buffer.set("location", String::from("https://arstechnica.com"))?;
        assert_eq!(new_buffer.set("fruit", 2u8)?, true);

     
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

        // 0.5.0 and greater API
        let mut new_buffer = user_factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.set(&["initialized"], example_data.initialized)?;
        new_buffer.set(&["location"], example_data.location)?;
        new_buffer.set(&["fruit"], example_data.fruit)?;

        for (i, list) in example_data.list.iter().enumerate() {

            new_buffer.reset_cursor();
            new_buffer.move_cursor(&["list", i.to_string().as_str()])?;
            new_buffer.set(&["name"], list.name)?;
            new_buffer.set(&["rating"], list.rating)?;
            new_buffer.set(&["postfix"], list.postfix)?;

            new_buffer.move_cursor(&["sibling"])?;
            new_buffer.set(&["time"], list.sibling.time)?;
            new_buffer.set(&["ratio"], list.sibling.ratio)?;
            new_buffer.set(&["size"], list.sibling.size)?;

            new_buffer.move_cursor(&["parent"])?;
            new_buffer.set(&["id"], list.sibling.parent.id)?;
            new_buffer.set(&["count"], list.sibling.parent.count)?;
            new_buffer.set(&["prefix"], list.sibling.parent.prefix)?;
            new_buffer.set(&["length"], list.sibling.parent.length)?;
        }

        // let bytes = new_buffer.close();
    }

    let time = SystemTime::now().duration_since(start).expect("Time went backwards");
    println!("{:?}", time);

    Ok(())
}