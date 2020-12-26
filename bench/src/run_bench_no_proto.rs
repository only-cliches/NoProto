use crate::LOOPS;
use no_proto::{error::NP_Error};
use no_proto::NP_Factory;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
pub struct NoProtoBench();

impl NoProtoBench {

    pub fn size_bench() {
        let factory = NoProtoBench::get_factory().unwrap();
        let encoded = Self::encode_single(&factory).unwrap();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("NoProto:     size: {}b, zlib: {}b", encoded.len(), compressed.len());
    }


    pub fn encode_bench() -> Result<u128, NP_Error> {
        let factory = NoProtoBench::get_factory()?;

        let start = SystemTime::now();
    
        for _x in 0..LOOPS {
            let new_buffer = NoProtoBench::encode_single(&factory)?;
            assert_eq!(new_buffer.len(), 209);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok(time.as_micros())
    }

    pub fn update_bench() -> Result<u128, NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut new_buff = factory.open_buffer(new_buffer.clone());

            new_buff.set(&["list", "0", "name"], "bob")?;

            // new_buff.compact(None)?;

            assert_eq!(new_buff.close().len(), 209);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok(time.as_micros())
    }

    #[inline(always)]
    fn get_factory<'get>() -> Result<NP_Factory<'get>, NP_Error> {
        NP_Factory::new(r#"{
            "type": "table",
            "columns": [
                ["list",   {"type": "list", "of": {
                    "type": "table",
                    "columns": [
                        ["name", {"type": "string"}],
                        ["rating", {"type": "float"}],
                        ["postfix", {"type": "string", "size": 1}],
                        ["sibling", {"type": "table", "columns": [
                            ["time", {"type": "u32"}],
                            ["ratio", {"type": "float"}],
                            ["size", {"type": "u16"}]
                        ]}]
                    ]
                }}],
                ["initialized", {"type": "bool"}],
                ["location", {"type": "string"}],
                ["fruit", {"type": "u8"}]
            ]
        }"#)
    }

    pub fn decode_one_bench() -> Result<u128, NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let new_buff = factory.open_buffer_ro(&new_buffer);
            assert_eq!(new_buff.get(&["location"])?, Some("https://arstechnica.com"));
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok(time.as_micros())
    }

    pub fn decode_bench() -> Result<u128, NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut new_buff = factory.open_buffer_ro(&new_buffer);

            assert_eq!(new_buff.get(&["initialized"])?, Some(true));
            assert_eq!(new_buff.get(&["location"])?, Some("https://arstechnica.com"));
            assert_eq!(new_buff.get(&["fruit"])?, Some(2u8));

            let mut loops = 0;

            for (x1, x) in [("0", 0), ("1", 1), ("2", 2)].iter() {
                loops += 1;
                new_buff.cursor_to_root();
                new_buff.move_cursor(&["list", x1])?;
                assert_eq!(new_buff.get(&["name"])?, Some("Hello, world!"));
                assert_eq!(new_buff.get(&["rating"])?, Some(3.1415432432445543543 + (*x as f32)));
                assert_eq!(new_buff.get(&["postfix"])?, Some("!"));
        
                new_buff.move_cursor(&["sibling"])?;
                assert_eq!(new_buff.get(&["time"])?, Some(123456 + (*x as u32)));
                assert_eq!(new_buff.get(&["ratio"])?, Some(3.14159 + (*x as f32)));
                assert_eq!(new_buff.get(&["size"])?, Some(10000 + (*x as u16)));
            }

            assert!(loops == 3);
            
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok(time.as_micros())
    }

    #[inline(always)]
    pub fn encode_single(factory: &NP_Factory) ->Result<Vec<u8>, NP_Error> {
        let mut new_buffer = factory.empty_buffer(None);

        new_buffer.set(&["initialized"], true)?;
        new_buffer.set(&["location"], "https://arstechnica.com")?;
        new_buffer.set(&["fruit"], 2u8)?;
    
        for (x1, x) in [("0", 0), ("1", 1), ("2", 2)].iter() {
    
            new_buffer.cursor_to_root();
            new_buffer.move_cursor(&["list", x1])?;
            new_buffer.set(&["name"], "Hello, world!")?;
            new_buffer.set(&["rating"], 3.1415432432445543543 + (*x as f32))?;
            new_buffer.set(&["postfix"], "!")?;
    
            new_buffer.move_cursor(&["sibling"])?;
            new_buffer.set(&["time"], 123456 + (*x as u32))?;
            new_buffer.set(&["ratio"], 3.14159 + (*x as f32))?;
            new_buffer.set(&["size"], 10000 + (*x as u16))?;
            
        }
    
        Ok(new_buffer.close())
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