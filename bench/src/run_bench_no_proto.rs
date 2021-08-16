use crate::LOOPS;
use no_proto::{error::NP_Error};
use no_proto::NP_Factory;
use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};

static SCHEMA: [u8; 135] = [21u8, 4, 4, 108, 105, 115, 116, 0, 83, 23, 21, 4, 4, 110, 97, 109, 101, 0, 6, 2, 0, 0, 0, 0, 0, 6, 114, 97, 116, 105, 110, 103, 0, 2, 12, 0, 7, 112, 111, 115, 116, 102, 105, 120, 0, 6, 2, 0, 0, 1, 0, 0, 7, 115, 105, 98, 108, 105, 110, 103, 0, 30, 21, 3, 4, 116, 105, 109, 101, 0, 2, 10, 0, 5, 114, 97, 116, 105, 111, 0, 2, 12, 0, 4, 115, 105, 122, 101, 0, 2, 9, 0, 11, 105, 110, 105, 116, 105, 97, 108, 105, 122, 101, 100, 0, 2, 15, 0, 8, 108, 111, 99, 97, 116, 105, 111, 110, 0, 6, 2, 0, 0, 0, 0, 0, 5, 102, 114, 117, 105, 116, 0, 2, 8, 0];

pub struct NoProtoBench();

impl NoProtoBench {

    pub fn setup_bench() -> u128 {
        let start = SystemTime::now();
    
        let factory = Self::get_factory().unwrap();
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");

        println!("NoProto:     setup: {:?}", time.as_micros() as f64 / 1000f64);
        time.as_micros()
    }

    pub fn size_bench() -> (usize, usize) {
        let factory = NoProtoBench::get_factory().unwrap();

        let encoded = Self::encode_single(&factory).unwrap();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("NoProto:     size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }


    pub fn encode_bench() -> Result<(u128, String), NP_Error> {
        let factory = NoProtoBench::get_factory()?;

        let start = SystemTime::now();
    
        for _x in 0..LOOPS {
            let new_buffer = NoProtoBench::encode_single(&factory)?;
            assert_eq!(new_buffer.len(), 308);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok((time.as_micros(), format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)))
    }

    pub fn update_bench() -> Result<(u128, String), NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut new_buff = factory.open_buffer(new_buffer.clone());

            new_buff.set(&["list", "0", "name"], "bob")?;

            // new_buff.compact(None)?;

            assert_eq!(new_buff.finish().bytes().len(), 308);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok((time.as_micros(), format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)))
    }

    #[inline(always)]
    fn get_factory() -> Result<NP_Factory, NP_Error> {
        
//         NP_Factory::new_bytes(&SCHEMA)
        NP_Factory::new(r#"
            struct({fields: {
                list: list({of: struct({fields: {
                    name: string(),
                    rating: float(),
                    postfix: string({size: 1}),
                    sibling: struct({fields: {
                        time: u32(),
                        ratio: float(),
                        size: u16()
                    }})
                }})}),
                initialized: bool(),
                location: string(),
                fruit: u8()
            }})
        "#)
        // NP_Factory::new_json(r#"{
        //     "type": "table",
        //     "columns": [
        //         ["list",   {"type": "list", "of": {
        //             "type": "table",
        //             "columns": [
        //                 ["name", {"type": "string"}],
        //                 ["rating", {"type": "float"}],
        //                 ["postfix", {"type": "string", "size": 1}],
        //                 ["sibling", {"type": "table", "columns": [
        //                     ["time", {"type": "u32"}],
        //                     ["ratio", {"type": "float"}],
        //                     ["size", {"type": "u16"}]
        //                 ]}]
        //             ]
        //         }}],
        //         ["initialized", {"type": "bool"}],
        //         ["location", {"type": "string"}],
        //         ["fruit", {"type": "u8"}]
        //     ]
        // }"#)
    }

    pub fn decode_one_bench() -> Result<(u128, String), NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let new_buff = factory.open_buffer_ref(&new_buffer);
            assert_eq!(new_buff.get(&["location"])?, Some("http://arstechnica.com"));
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("NoProto:     {:>9.0} ops/ms 1.00", LOOPS as f64 / time.as_millis() as f64);  

        Ok((time.as_micros(), format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)))
    }

    pub fn decode_bench() -> Result<(u128, String), NP_Error> {
        let factory = NoProtoBench::get_factory()?;
        let new_buffer = NoProtoBench::encode_single(&factory)?;
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let mut new_buff = factory.open_buffer_ref(&new_buffer);

            assert_eq!(new_buff.get(&["initialized"])?, Some(true));
            assert_eq!(new_buff.get(&["location"])?, Some("http://arstechnica.com"));
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

        Ok((time.as_micros(), format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)))
    }

    #[inline(always)]
    pub fn encode_single(factory: &NP_Factory) ->Result<Vec<u8>, NP_Error> {
        let mut new_buffer = factory.new_buffer(None);

        new_buffer.set(&["initialized"], true)?;
        new_buffer.set(&["location"], "http://arstechnica.com")?;
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
    
        Ok(new_buffer.finish().bytes())
    }
}