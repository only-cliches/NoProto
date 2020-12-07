use crate::LOOPS;
use no_proto::{error::NP_Error, memory::NP_Size};
use no_proto::NP_Factory;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime, UNIX_EPOCH};
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


    pub fn encode_bench() -> Result<(), NP_Error> {
        let factory = NoProtoBench::get_factory()?;

        let start = SystemTime::now();
    
        for _x in 0..LOOPS {
            let new_buffer = NoProtoBench::encode_single(&factory)?;
            assert_eq!(new_buffer.len(), 408);
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
            let mut new_buff = factory.open_buffer(new_buffer.clone())?;

            new_buff.set(&["list", "0", "name"], "bob")?;

            assert_eq!(new_buff.close().len(), 408);
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
                        ["postfix", {"type": "string", "size": 1}],
                        ["sibling", {"type": "table", "columns": [
                            ["time", {"type": "u32"}],
                            ["ratio", {"type": "float"}],
                            ["size", {"type": "u16"}],
                            ["parent", {"type": "table", "columns": [
                                ["id", {"type": "u64"}],
                                ["count", {"type": "u16"}],
                                ["prefix", {"type": "string", "size": 1}],
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

    pub fn decode_bench() -> Result<(), NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for x in 0..LOOPS {
            let mut new_buff = factory.open_buffer(new_buffer.clone())?;

            assert_eq!(new_buff.get(&["initialized"])?, Some(true));
            assert_eq!(new_buff.get(&["location"])?, Some("https://arstechnica.com"));
            assert_eq!(new_buff.get(&["fruit"])?, Some(2u8));

            for x in 0..3 {
    
                new_buff.cursor_to_root();
                new_buff.move_cursor(&["list", x.to_string().as_str()])?;
                assert_eq!(new_buff.get(&["name"])?, Some("Hello, world!"));
                assert_eq!(new_buff.get(&["rating"])?, Some(3.1415432432445543543 + (x as f32)));
                assert_eq!(new_buff.get(&["postfix"])?, Some("!"));
        
                new_buff.move_cursor(&["sibling"])?;
                assert_eq!(new_buff.get(&["time"])?, Some(123456 + (x as u32)));
                assert_eq!(new_buff.get(&["ratio"])?, Some(3.14159 + (x as f32)));
                assert_eq!(new_buff.get(&["size"])?, Some(10000 + (x as u16)));
        
                new_buff.move_cursor(&["parent"])?;
                assert_eq!(new_buff.get(&["id"])?, Some(0xABADCAFEABADCAFE + (x as u64)));
                assert_eq!(new_buff.get(&["count"])?, Some(10000 + (x as u16)));
                assert_eq!(new_buff.get(&["prefix"])?, Some("@"));
                assert_eq!(new_buff.get(&["length"])?, Some(1000000 + (x as u32)));
                
            }
            
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:?}", time);

        Ok(())
    }

    #[inline(always)]
    pub fn encode_single(factory: &NP_Factory) ->Result<Vec<u8>, NP_Error> {
        let mut new_buffer = factory.empty_buffer(None, Some(NP_Size::U16));

        new_buffer.insert(&["initialized"], true)?;
        new_buffer.insert(&["location"], "https://arstechnica.com")?;
        new_buffer.insert(&["fruit"], 2u8)?;
    
        for x in 0..3 {
    
            new_buffer.cursor_to_root();
            new_buffer.move_cursor(&["list"], x.to_string().as_str())?;
            new_buffer.insert(&["name"], "Hello, world!")?;
            new_buffer.insert(&["rating"], 3.1415432432445543543 + (x as f32))?;
            new_buffer.insert(&["postfix"], "!")?;
    
            new_buffer.move_cursor(&["sibling"])?;
            new_buffer.insert(&["time"], 123456 + (x as u32))?;
            new_buffer.insert(&["ratio"], 3.14159 + (x as f32))?;
            new_buffer.insert(&["size"], 10000 + (x as u16))?;
    
            new_buffer.move_cursor(&["parent"])?;
            new_buffer.insert(&["id"], 0xABADCAFEABADCAFE + (x as u64))?;
            new_buffer.insert(&["count"], 10000 + (x as u16))?;
            new_buffer.insert(&["prefix"], "@")?;
            new_buffer.insert(&["length"], 1000000 + (x as u32))?;
            
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