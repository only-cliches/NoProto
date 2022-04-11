use crate::schema::AST_STR;
use alloc::string::String;
use alloc::vec::Vec;
use crate::map::NP_OrderedMap;
use crate::json_flex::{NP_JSON, JSMAP};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NP_Schema_Args {
    NULL,
    TRUE,
    FALSE,
    STRING (AST_STR),
    NUMBER (AST_STR),
    MAP (NP_OrderedMap<NP_Schema_Args>),
    LIST (Vec<NP_Schema_Args>)
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NP_Args<'a> {
    NULL,
    TRUE,
    FALSE,
    STRING (&'a str),
    NUMBER (&'a str),
    MAP (NP_OrderedMap<NP_Args<'a>>),
    LIST (Vec<NP_Args<'a>>)
}

impl<'a> NP_Args<'a> {

    pub fn from_schema_args(schema: &NP_Schema_Args, source_string: &'a str) -> Self {
        match schema {
            NP_Schema_Args::NULL => NP_Args::NULL,
            NP_Schema_Args::TRUE => NP_Args::TRUE,
            NP_Schema_Args::FALSE => NP_Args::FALSE,
            NP_Schema_Args::STRING(ast_str) => NP_Args::STRING(ast_str.read(source_string)),
            NP_Schema_Args::NUMBER(ast_str) => NP_Args::NUMBER(ast_str.read(source_string)),
            NP_Schema_Args::MAP(in_map) => {
                let mut map = NP_OrderedMap::new();

                for (key, value) in in_map.iter() {
                    map.set(key, Self::from_schema_args(value, source_string));
                }

                NP_Args::MAP(map)
            },
            NP_Schema_Args::LIST(in_list) => {
                NP_Args::LIST(in_list.iter().map(|v| Self::from_schema_args(v, source_string)).collect())
            }
        }
    }

    pub fn to_json(&self) -> NP_JSON {
        match self {
            NP_Args::NULL => NP_JSON::Null,
            NP_Args::TRUE => NP_JSON::True,
            NP_Args::FALSE => NP_JSON::False,
            NP_Args::STRING(str_data) => NP_JSON::String( String::from(*str_data) ),
            NP_Args::NUMBER(str_data) => {
                if let Ok(result) = str_data.parse::<i64>() {
                    NP_JSON::Integer(result)
                } else {
                    if let Ok(result) = str_data.parse::<f64>() {
                        NP_JSON::Float(result)
                    } else {
                        NP_JSON::Null
                    }
                }
            },
            NP_Args::MAP( map_data ) => {
                let mut json_map = JSMAP::new();

                for (key, value) in map_data.iter() {
                    json_map.insert(String::from(key), value.to_json());
                }

                NP_JSON::Dictionary(json_map)
            }
            NP_Args::LIST( list_data ) => {
                NP_JSON::Array(list_data.iter().map(|v| v.to_json()).collect())
            }
        }
    }
}

impl Default for NP_Schema_Args {
    fn default() -> Self {
        NP_Schema_Args::NULL
    }
}

#[allow(dead_code)]
impl NP_Schema_Args {

    pub fn query<'q>(&'q self, path: &str, str_source: &'q str) -> Option<NP_Args<'q>> {

        let mut dot_locations: [usize; 32] = Default::default();
        let mut num_dots: usize = 1;

        for (idx, char) in path.chars().enumerate() {
            if char == '.' {
                dot_locations[num_dots] = idx;
                num_dots += 1;
            }
        }

        let mut query_object = self;

        let mut step: usize = 0;

        while step <= num_dots {

            if step >= num_dots || path.trim().len() == 0 {
                return Some(NP_Args::from_schema_args(query_object, str_source));
            } else {

                let use_path = if step == 0 { // first
                    if num_dots == 1 { // no dots in path
                        path
                    } else { // we have dots!
                        &path[0..(dot_locations[step + 1])]
                    }
                } else if step == num_dots - 1 { // last
                    &path[(dot_locations[step] + 1)..path.len()]
                } else { // middle
                    &path[(dot_locations[step] + 1)..dot_locations[step + 1]]
                };

                match query_object {
                    NP_Schema_Args::NULL => {
                        return None;
                    }
                    NP_Schema_Args::TRUE => {
                        return None;
                    }
                    NP_Schema_Args::FALSE => {
                        return None;
                    }
                    NP_Schema_Args::STRING (_data) => {
                        return None;
                    }
                    NP_Schema_Args::NUMBER (_data) => {
                        return None;
                    }
                    NP_Schema_Args::MAP (data) => {
                        if let Some(item) = data.get(use_path) {
                            query_object = item;
                        } else {
                            return None;
                        }
                    }
                    NP_Schema_Args::LIST (data) => {
                        if let Ok(index) = use_path.parse::<usize>() {
                            if let Some(item) = data.get(index) {
                                query_object = item;
                            } else {
                                return None;
                            }
                        }
                    }
                }
            }

            step += 1;
        }

        return None;
    }
}