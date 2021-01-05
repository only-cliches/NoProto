use crate::LOOPS;

use avro_rs::{
    types::Record, types::Value, Codec, Days, Decimal, Duration, Error, Millis, Months, Reader,
    Schema, Writer,
};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use serde::{Deserialize, Serialize};
use std::io::prelude::*;
use std::time::SystemTime;


#[repr(i32)]
enum Fruit {
    Apples, Pears, Bananas
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

        println!(
            "Avro:        size: {}b, zlib: {}b",
            encoded.len(),
            compressed.len()
        );
        return (encoded.len(), compressed.len());
    }

    pub fn encode_bench(base: u128) -> String {
        let schema = Self::get_schema();
        let start = SystemTime::now();

        for _x in 0..LOOPS {
            let buffer = Self::encode_single(&schema);
            assert_eq!(buffer.len(), 702);
        }

        let time = SystemTime::now()
            .duration_since(start)
            .expect("Time went backwards");
        println!(
            "Avro:        {:>9.0} ops/ms {:.2}",
            LOOPS as f64 / time.as_millis() as f64,
            (base as f64 / time.as_micros() as f64)
        );
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    #[inline(always)]
    fn encode_single(schema: &Schema) -> Vec<u8> {
        let mut foobar_c: Vec<(String, Value)> = Vec::new();

        foobar_c.push((String::from("initialized"), Value::Boolean(true)));
        foobar_c.push((String::from("fruit"), Value::Int(Fruit::Apples as i32)));
        foobar_c.push((
            String::from("location"),
            Value::String(String::from("http://arstechnica.com")),
        ));

        let mut vector: Vec<Value> = Vec::new();

        for x in 0..3 {
            let mut bar: Vec<(String, Value)> = Vec::new();
            bar.push((String::from("time"), Value::Int(123456 + (x as i32))));
            bar.push((String::from("ratio"), Value::Float(3.14159 + (x as f32))));
            bar.push((String::from("size"), Value::Int(10000 + (x as i32))));

            let mut foobar: Vec<(String, Value)> = Vec::new();

            foobar.push((
                String::from("name"),
                Value::String(String::from("Hello, world!")),
            ));
            foobar.push((
                String::from("rating"),
                Value::Float(3.1415432432445543543 + (x as f32)),
            ));
            foobar.push((String::from("postfix"), Value::String(String::from("!"))));
            foobar.push((String::from("sibling"), Value::Record(bar)));

            vector.push(Value::Record(foobar));
        }

        foobar_c.push((String::from("list"), Value::Array(vector)));

        let mut writer = Writer::new(&schema, Vec::new());
        writer.append(Value::Record(foobar_c)).unwrap();
        writer.into_inner().unwrap()
    }

    pub fn update_bench(base: u128) -> String {

        let schema = Self::get_schema();

        let buffer = Self::encode_single(&schema);

        let start = SystemTime::now();

        for _x in 0..LOOPS {

            let reader = Reader::new(&buffer[..]).unwrap();
            let mut foobar_c: Vec<(String, Value)> = Vec::new();

            

            for val in reader {
                if let Value::Record(data) = val.unwrap() {
                    data.iter().for_each(|(key, data)| {
						match key.as_str() {
                            "list" => {

                                let mut vector: Vec<Value> = Vec::new();

                                if let Value::Array(list) = data {
                                    list.iter().enumerate().for_each(|(i, foo_bar)| {
                                        
                                        if i == 0 {
                                            if let Value::Record(foo_bar) = foo_bar {
                                                let mut new_foobar: Vec<(String, Value)> = Vec::new();

                                                foo_bar.iter().for_each(|(key, value)| {
                                                    match key.as_str() {
                                                        "name" => {
                                                            new_foobar.push((String::from("name"), Value::String(String::from("bob"))));
                                                        },
                                                        _ => {
                                                            new_foobar.push((key.clone(), value.clone()));
                                                        }
                                                    }
                                                });
                                                vector.push(Value::Record(new_foobar));
                                            } else {
                                                panic!()
                                            }                                            
                                        } else {
                                            vector.push(foo_bar.clone());
                                        }
                                    });
                                } else {
                                    panic!()
                                }

                                foobar_c.push((String::from("list"), Value::Array(vector)));
                            },
                            _ => {
                                foobar_c.push((key.clone(), data.clone()));
                            }
						};
                    });
                }
            }

            let mut writer = Writer::new(&schema, Vec::new());
            writer.append(Value::Record(foobar_c)).unwrap();
            let finished = writer.into_inner().unwrap();
            assert_eq!(finished.len(), 692);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_one_bench(base: u128) -> String {
        

        let schema = Self::get_schema();

        let start = SystemTime::now();

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

        let time = SystemTime::now()
            .duration_since(start)
            .expect("Time went backwards");
        println!(
            "Avro:        {:>9.0} ops/ms {:.2}",
            LOOPS as f64 / time.as_millis() as f64,
            (base as f64 / time.as_micros() as f64)
        );
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }

    pub fn decode_bench(base: u128) -> String {
        let schema = Self::get_schema();

        let buffer = Self::encode_single(&schema);

        let start = SystemTime::now();

        for _x in 0..LOOPS {

            let mut loops = 0;

            let reader = Reader::new(&buffer[..]).unwrap();
            for val in reader {
                if let Value::Record(data) = val.unwrap() {
                    data.iter().for_each(|(key, data)| {
						match key.as_str() {
							"location" => {
                                if let Value::String(x) = data {
                                    assert_eq!("http://arstechnica.com", x);
                                } else {
                                    panic!()
                                }
							},
							"initialized" => {
                                if let Value::Boolean(x) = data {
                                    assert_eq!(true, *x);
                                } else {
                                    panic!()
                                }
							},
							"fruit" => {
                                if let Value::Int(x) = data {
                                    assert_eq!(Fruit::Apples as i32, *x);
                                } else {
                                    panic!()
                                }
							},
							"list" => {
                                if let Value::Array(list) = data {
                                    list.iter().for_each(|foo_bar| {
                                        if let Value::Record(foo_bar) = foo_bar {
                                            let mut key_count = 0;
                                            foo_bar.iter().for_each(|(key, value)| {
                                                key_count += 1;
                                                match key.as_str() {
                                                    "name" => {
                                                        if let Value::String(x) = value {
                                                            assert_eq!("Hello, world!", x);
                                                        } else {
                                                            panic!()
                                                        }
                                                    },
                                                    "rating" => {
                                                        if let Value::Float(x) = value {
                                                            assert_eq!(3.1415432432445543543 + (loops as f32), *x);
                                                        } else {
                                                            panic!()
                                                        }
                                                    },
                                                    "postfix" => {
                                                        if let Value::String(x) = value {
                                                            assert_eq!("!", x);
                                                        } else {
                                                            panic!()
                                                        }
                                                    },
                                                    "sibling" => {
                                                        if let Value::Record(sibling) = value {
                                                            let mut foo_key_count = 0;
                                                            sibling.iter().for_each(|(skey, svalue)| {
                                                                foo_key_count += 1;
                                                                match skey.as_str() {
                                                                    "time" => {
                                                                        assert_eq!(Value::Int(123456 + (loops as i32)), *svalue);
                                                                    },
                                                                    "ratio" => {
                                                                        assert_eq!(Value::Float(3.14159 + (loops as f32)), *svalue);
                                                                    },
                                                                    "size" => {
                                                                        assert_eq!(Value::Int(10000 + (loops as i32)), *svalue);
                                                                    },
                                                                    _ => panic!()
                                                                }
                                                            });
                                                            assert_eq!(foo_key_count, 3);
                                                        } else {
                                                            panic!()
                                                        }
                                                    },
                                                    _ => panic!()
                                                }
                                            });
                                            assert_eq!(key_count, 4);
                                        } else {
                                            panic!()
                                        }

                                        loops += 1;
                                    });
                                } else {
                                    panic!()
                                }
							},
							_ => panic!()
						};
                    });
                }
            }

            assert!(loops == 3);
        }

        let time = SystemTime::now().duration_since(start).expect("Time went backwards");
        println!("Avro:        {:>9.0} ops/ms {:.2}", LOOPS as f64 / time.as_millis() as f64, (base as f64 / time.as_micros() as f64));
        format!("{:>6.0}", LOOPS as f64 / time.as_millis() as f64)
    }
}
