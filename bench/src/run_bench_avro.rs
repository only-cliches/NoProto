use crate::LOOPS;

use std::io::prelude::*;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use std::time::{SystemTime};
use serde::{Serialize, Deserialize};
use avro_rs::{
    types::Record, types::Value, Codec, Days, Decimal, Duration, Millis, Months, Reader, Schema,
    Writer, Error,
};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[repr(i32)]
enum Fruit {
    Apples, Pears, Bananas
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct Bar {
  time: i32,
  ratio: f32,
  size: u16
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct FooBar<'fb> {
  sibling: Bar,
  name: &'fb str,
  rating: f64,
  postfix: char
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
struct FooBarContainer<'con> {
  // list: Vec<FooBar<'con>>,
  list: Vec<String>,
  initialized: bool,
  fruit: i32, 
  location: &'con str
}




pub struct AvroBench();

impl AvroBench {

    fn get_schema() -> Schema {
        let foo_bar_container = r#"
        {
		  "name": "FooBarContainer",
          "type": "record",
          "fields": [
            {"name": "initialized", "type": "boolean"},
            {"name": "fruit", "type": "int"},
			{"name": "location", "type": "string"},
			{"name": "list" ,"type": "array", "items": {
				"name": "FooBar",
				"type": "record",
				"fields": [
				  {"name": "name", "type": "string"},
				  {"name": "rating", "type": "float"},
				  {"name": "postfix", "type": "string"},
				  {"name": "sibling", "type": "record", "fields": [
						{"name": "time", "type": "int"},
						{"name": "ratio", "type": "float"},
						{"name": "size", "type": "int"}
					]
				  }
				]
			  }}
          ]
        }"#;
    
        Schema::parse_str(foo_bar_container).unwrap()
    }

    pub fn size_bench() -> (usize, usize) {

        let schema = Self::get_schema();
        let encoded = Self::encode_single(&schema);

        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write(&encoded[..]).unwrap();
        let compressed = e.finish().unwrap();

        println!("Avro:        size: {}b, zlib: {}b", encoded.len(), compressed.len());
        return (encoded.len(), compressed.len())
    }

    pub fn encode_bench(base: u128) -> String {
        let schema = Self::get_schema();
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single(&schema);
            assert_eq!(buffer.len(), 702);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64)); 
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    fn encode_single(schema: &Schema) -> Vec<u8> {
 
        let mut foobar_c: Vec<(String, Value)> = Vec::new();
    
		foobar_c.push((String::from("initialized"), Value::Boolean(true)));
		foobar_c.push((String::from("fruit"), Value::Int(Fruit::Apples as i32)));
		foobar_c.push((String::from("location"), Value::String(String::from("http://arstechnica.com"))));

        let mut vector: Vec<Value> = Vec::new();

        for x in 0..3 {
          let mut bar: Vec<(String, Value)> = Vec::new();
		  bar.push((String::from("time"), Value::Int(123456 + (x as i32))));
		  bar.push((String::from("ratio"), Value::Float(3.14159 + (x as f32))));
		  bar.push((String::from("size"), Value::Int(10000 + (x as i32))));

		  let mut foobar: Vec<(String, Value)> = Vec::new();
		  
		  foobar.push((String::from("name"), Value::String(String::from("Hello, world!"))));
		  foobar.push((String::from("rating"), Value::Float(3.1415432432445543543 + (x as f32))));
		  foobar.push((String::from("postfix"), Value::String(String::from("!"))));
		  foobar.push((String::from("sibling"), Value::Record(bar)));

		  vector.push(Value::Record(foobar));
        }

        foobar_c.push((String::from("list"), Value::Array(vector)));

		let mut writer = Writer::new(&schema, Vec::new());
        writer.append(Value::Record(foobar_c)).unwrap();
        writer.into_inner().unwrap()
    }

    pub fn update_bench(base: u128) -> String  {
        
        // let buffer = Self::encode_single();

        // let start = SystemTime::now();

        // for _x in 0..LOOPS {
        //     let mut decoded: FooBarContainer = FooBarContainer::decode(&buffer[..]).unwrap();

        //     decoded.list[0].name = Some(String::from("bob"));

        //     let mut bytes = Vec::new();
        //     decoded.encode(&mut bytes).unwrap();
        //     assert_eq!(bytes.len(), 144);
        // }

        // let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        // println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        // format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
        String::from("")
    }

    pub fn decode_one_bench(base: u128) -> String {
		let start = SystemTime::now();
		
		let schema = Self::get_schema();

        let buffer = Self::encode_single(&schema);


        for _x in 0..LOOPS {
			let mut found = false;
			let reader = Reader::new(&buffer[..]).unwrap();
			for val in reader {
				if let Value::Record(data) = val.unwrap() {
					data.iter().for_each(|(key, data)| {
						if key == "location" {
							if let Value::String(x) = data {
								found = true;
								assert_eq!("http://arstechnica.com", x);
							}
						}
					});
				}
			}
			assert!(found == true);
        }
    
        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        // let buffer = Self::encode_single();

        // let start = SystemTime::now();

        // let hello_world = Some(String::from("Hello, world!"));
        // let ars_technica = Some(String::from("http://arstechnica.com"));

        // for _x in 0..LOOPS {
        //     let decoded: FooBarContainer = FooBarContainer::decode(&buffer[..]).unwrap();

        //     let mut loops = 0;

        //     decoded.list.iter().enumerate().for_each(|(x, foobar)| {
        //         loops += 1;
        //         match foobar.sibling.as_ref() {
        //             Some(old_bar) => {
        //                 assert_eq!(old_bar.time, 123456 + (x as i32));
        //                 assert_eq!(old_bar.ratio, 3.14159 + (x as f32));
        //                 assert_eq!(old_bar.size, 10000 + (x as u32));
        //             },
        //             None => panic!()
        //         }

        //         assert_eq!(foobar.name, hello_world);
        //         assert_eq!(foobar.rating, Some(3.1415432432445543543 + (x as f64)));
        //         assert_eq!(foobar.postfix, Some("!".as_bytes()[0] as u32));
        //     });

        //     assert!(loops == 3);

        //     assert_eq!(decoded.location, ars_technica);
        //     assert_eq!(decoded.fruit, Some(Enum::Apples as i32));
        //     assert_eq!(decoded.initialized, Some(true));
        // }

        // let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        // println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        // format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
        String::from("")
    }

}