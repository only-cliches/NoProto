use crate::schema::ast_parser::AST_STR;
use crate::hashmap::NP_HashMap;
use alloc::prelude::v1::Vec;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum NP_Schema_Args {
    NULL,
    TRUE,
    FALSE,
    STRING (AST_STR),
    NUMBER (AST_STR),
    MAP (NP_HashMap<NP_Schema_Args>),
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
    MAP (&'a NP_HashMap<NP_Schema_Args>),
    LIST (&'a Vec<NP_Schema_Args>)
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

            if step >= num_dots {
                return match query_object {
                    NP_Schema_Args::NULL => {
                        Some(NP_Args::NULL)
                    }
                    NP_Schema_Args::TRUE => {
                        Some(NP_Args::TRUE)
                    }
                    NP_Schema_Args::FALSE => {
                        Some(NP_Args::FALSE)
                    }
                    NP_Schema_Args::STRING(data) => {
                        Some(NP_Args::STRING(data.read(str_source)))
                    }
                    NP_Schema_Args::NUMBER(data) => {
                        Some(NP_Args::NUMBER(data.read(str_source)))
                    }
                    NP_Schema_Args::MAP(data) => {
                        Some(NP_Args::MAP(data))
                    }
                    NP_Schema_Args::LIST(data) => {
                        Some(NP_Args::LIST(data))
                    }
                }
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