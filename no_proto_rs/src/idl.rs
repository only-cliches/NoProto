//! ES6 IDL for Schemas
//! 
//! Supports a *very* limited subset of ES6/Javascript parsing for schemas and rpcs.
//! 
use crate::error::NP_Error;
use alloc::string::String;
use alloc::vec::Vec;

/// Parsed AST String
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AST_STR { 
    start: usize, 
    end: usize 
}

/// AST object of es6 schema
#[allow(missing_docs)]
#[derive(Debug)]
pub enum JS_AST {
    empty,
    method { name: AST_STR, args: Vec<JS_AST> },
    object { properties: Vec<(AST_STR, JS_AST)> },
    bool { state: bool },
    string { addr: AST_STR },
    array { values: Vec<JS_AST> },
    number { addr: AST_STR },
    closure { expressions: Vec<JS_AST> }
}

#[allow(missing_docs)]
#[derive(Debug)]
/// Schema using ES6 syntax
pub struct JS_Schema {
    value: String,
    pub ast: JS_AST
}

#[derive(PartialEq)]
enum js_control {
    none,
    paran,
    curly,
    square,
    quote
}

impl JS_Schema {
    /// Parse a JS style schema into AST
    pub fn new(schema: String) -> Result<Self, NP_Error> {
        let mut no_comments: String = String::with_capacity(schema.len());

        schema.trim().split("\n").for_each(|f| {
            let trimmed = f.trim();

            if trimmed.len() > 0 {
                if let Some(idx) = trimmed.find("//") {
                    if idx > 0 {
                        no_comments.push_str(&trimmed[..idx]);
                    }
                } else {
                    no_comments.push_str(trimmed);
                };
            }
        });


        Ok(Self {
            ast: Self::parse(0, 0, no_comments.len(), &no_comments)?,
            value: no_comments,
        })
    }

    /// Get a str value from the schema
    pub fn get_str(&self, addr: &AST_STR) -> &str {
        &self.value[addr.start..addr.end]
    }

    fn parse(depth: usize, start: usize, end: usize, schema: &str) -> Result<JS_AST, NP_Error> {

        if start == end {
            return Err(NP_Error::new("empty request"));
        }

        if depth > 255 {
            return Err(NP_Error::new("too much depth!"));
        }


        let mut control_char = js_control::none;

        let mut index = start;
        while control_char == js_control::none && index < end {
            match &schema[index..(index + 1)] {
                "[" => { control_char = js_control::square; },
                "{" => { control_char = js_control::curly; },
                "(" => { control_char = js_control::paran; },
                "\"" => { control_char = js_control::quote; }
                _ => { }
            }

            index += 1;
        }

        static NESTING_DEFAULT: i16 = 0;

        let mut nesting = NESTING_DEFAULT;

        let mut closed = false;
        let mut moving_start = index;
        let mut escaped = false;
        let mut is_quoted = false;

        match control_char {
            js_control::none => { // number, bool or empty
                match schema[start..end].trim() {
                    "true" => Ok(JS_AST::bool { state: true }),
                    "false" => Ok(JS_AST::bool { state: false }),
                    "" => Ok(JS_AST::empty),
                    _ => Ok(JS_AST::number { addr: AST_STR { start, end }})
                }
            },
            js_control::square => { // array
                let mut arr: Vec<JS_AST> = Vec::new();

                while closed == false && index < end && nesting > -256 && nesting < 256 {

                    match &schema[index..(index + 1)] {
                        "]" => {
                            escaped = false;

                            if !is_quoted {
                                if nesting == NESTING_DEFAULT {
                                    if moving_start != index {
                                        arr.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                    }
                                    closed = true; 
                                } else {
                                    nesting -= 1;
                                }                                
                            }
                        },
                        "[" => { 
                            if !is_quoted {
                                escaped = false;
                                nesting += 1;
                            }
                        },
                        "{" => {
                            if !is_quoted {
                                escaped = false;
                                nesting += 1;                                
                            }
                        }
                        "}" => {
                            if !is_quoted {
                                escaped = false;
                                nesting -= 1;                                
                            }
                        },
                        "\\" => {
                            escaped = true;
                        },
                        "\"" => {
                            if escaped == false {
                                if is_quoted {
                                    nesting -= 1;
                                } else {
                                    nesting += 1;
                                }
                                is_quoted = !is_quoted;
                            }
                        },
                        "," => {
                            if nesting == NESTING_DEFAULT && !is_quoted {
                                if moving_start != index {
                                    arr.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                }
                                moving_start = index + 1;
                            }
                        },
                        _ => { 
                            escaped = false;
                        }
                    }
                    index += 1;
                }

                if closed == false {
                    let mut message = String::from("Missing matching square bracket for array! -> ");
                    message.push_str(&schema[start..usize::min(end, start + 20)]);
                    return Err(NP_Error::new(message.as_str()))
                }

                Ok(JS_AST::array { values: arr })
            },
            js_control::paran => { // function or closure
                if (index - 1) == start || schema[start..(index - 1)].trim().len() == 0 { // closure like (args) => { .. }

                    // we never use the args, so they just get skipped over.
                    let mut closed_first = false;
                    while closed_first == false && index < end {
                        match &schema[index..(index + 1)] {
                            "{" => { closed_first = true },
                            _ => { }
                        }
                        index += 1;
                    }

                    if closed_first == false {
                        let mut message = String::from("Missing closure open curly! -> ");
                        message.push_str(&schema[start..usize::min(end, start + 20)]);
                        return Err(NP_Error::new(message.as_str()))
                    }

                    moving_start = index;

                    let mut expressions: Vec<JS_AST> = Vec::new();

                    while closed == false && index < end && nesting > -256 && nesting < 256 {
                        match &schema[index..(index + 1)] {
                            "]" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting -= 1;                                
                                }
                            },
                            "[" => { 
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            },
                            "(" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            },
                            ")" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting -= 1;                                
                                }
                            },
                            "{" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            }
                            "}" => {
                                escaped = false;
                                if !is_quoted {
                                    if nesting == NESTING_DEFAULT {
                                        if moving_start != index {
                                            expressions.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                        }
                                        closed = true; 
                                    } else {
                                        nesting -= 1;
                                    }                                    
                                }
                            },
                            "\\" => {
                                escaped = true;
                            },
                            "\"" => {
                                if escaped == false {
                                    if is_quoted {
                                        nesting -= 1;
                                    } else {
                                        nesting += 1;
                                    }
                                    is_quoted = !is_quoted;
                                }
                            },
                            ";" => {
                                if nesting == NESTING_DEFAULT && !is_quoted {
                                    if moving_start != index {
                                        expressions.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                    }
                                    moving_start = index + 1;
                                }
                            },
                            _ => { 
                                escaped = false;
                            }
                        }
                        index += 1;
                    }

                    if closed == false {
                        let mut message = String::from("Missing matching paran for function! -> ");
                        message.push_str(&schema[start..usize::min(end, start + 20)]);
                        return Err(NP_Error::new(message.as_str()))
                    }

                    Ok(JS_AST::closure { expressions })
                } else { // function like some_name(...args)
                    let fn_name = AST_STR { start, end: index - 1 };

                    let mut args: Vec<JS_AST> = Vec::new();

                    while closed == false && index < end && nesting > -256 && nesting < 256 {
                        match &schema[index..(index + 1)] {
                            "]" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting -= 1;                                
                                }
                            },
                            "[" => { 
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            },
                            "(" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            },
                            ")" => {
                                escaped = false;
                                if !is_quoted {
                                    if nesting == NESTING_DEFAULT {
                                        if moving_start != index {
                                            args.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                        }
                                        closed = true; 
                                    } else {
                                        nesting -= 1;
                                    }                                    
                                }
                            },
                            "{" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting += 1;                                
                                }
                            }
                            "}" => {
                                if !is_quoted {
                                    escaped = false;
                                    nesting -= 1;                                
                                }
                            },
                            "\\" => {
                                escaped = true;
                            },
                            "\"" => {
                                if escaped == false {
                                    if is_quoted {
                                        nesting -= 1;
                                    } else {
                                        nesting += 1;
                                    }
                                    is_quoted = !is_quoted;
                                }
                            },
                            "," => {
                                if nesting == NESTING_DEFAULT  && !is_quoted {
                                    if moving_start != index {
                                        args.push(Self::parse(depth + 1, moving_start, index, schema)?);
                                    }
                                    moving_start = index + 1;
                                }
                            },
                            _ => { 
                                escaped = false;
                            }
                        }
                        index += 1;
                    }

                    if closed == false {
                        let mut message = String::from("Missing matching paran for function!\n");
                        message.push_str(&schema[start..usize::min(end, start + 10)]);
                        message.push_str("\n");
                        message.push_str("^------\n");
                        return Err(NP_Error::new(message.as_str()))
                    }

                    Ok(JS_AST::method { name: fn_name, args })
                }
            },
            js_control::curly => { // object
                let mut obj: Vec<(AST_STR, JS_AST)> = Vec::new();

                let mut key: Option<AST_STR> = None;

                while closed == false && index < end && nesting > -256 && nesting < 256 {
                    match &schema[index..(index + 1)] {
                        ":" => {
                            if !is_quoted {
                                if nesting == NESTING_DEFAULT {
                                    if moving_start != index {
                                        key = Some(AST_STR { start: moving_start, end: index});
                                    }
                                    moving_start = index + 1;
                                }                                
                            }

                        },
                        "]" => {
                            if !is_quoted {
                                escaped = false;
                                nesting -= 1;                                
                            }
                        },
                        "[" => { 
                            if !is_quoted {
                                escaped = false;
                                nesting += 1;                                
                            }
                        },
                        "{" => {
                            if !is_quoted {
                                escaped = false;
                                nesting += 1;                                
                            }
                        }
                        "}" => {
                            escaped = false;
                            if !is_quoted {
                                if nesting == NESTING_DEFAULT {
                                    if let Some(ast_key) = &key {
                                        if moving_start != index {
                                            obj.push((ast_key.clone(), Self::parse(depth + 1, moving_start, index, schema)?));
                                        }
                                        moving_start = index + 1;
                                        key = Option::None;
                                    } else {
                                        let mut message = String::from("Missing property name in object! -> ");
                                        message.push_str(&schema[moving_start..usize::min(end, moving_start + 10)]);
                                        return Err(NP_Error::new(message.as_str()))
                                    }
                                    closed = true; 
                                } else {
                                    nesting -= 1;
                                }                                
                            }
                        },
                        "\\" => {
                            escaped = true;
                        },
                        "\"" => {
                            if escaped == false {
                                if is_quoted {
                                    nesting -= 1;
                                } else {
                                    nesting += 1;
                                }
                                is_quoted = !is_quoted;
                            }
                        },
                        "," => {
                            if nesting == NESTING_DEFAULT && !is_quoted {
                                if let Some(ast_key) = &key {
                                    obj.push((ast_key.clone(), Self::parse(depth + 1, moving_start, index, schema)?));
                                    moving_start = index + 1;
                                    key = Option::None;
                                } else {
                                    let mut message = String::from("Missing property name in object! -> ");
                                    message.push_str(&schema[moving_start..usize::min(end, moving_start + 10)]);
                                    return Err(NP_Error::new(message.as_str()))
                                }
                            }
                        },
                        _ => { 
                            escaped = false;
                        }
                    }
                    index += 1;
                }

                if closed == false {
                    let mut message = String::from("Missing matching curly bracket for object! -> ");
                    message.push_str(&schema[start..usize::min(end, start + 20)]);
                    return Err(NP_Error::new(message.as_str()))
                }

                Ok(JS_AST::object{ properties: obj })
            },
            js_control::quote => { // string
                while closed == false && index < end {
                    match &schema[index..(index + 1)] {
                        "\\" => {
                            escaped = true;
                        },
                        "\"" => {
                            if escaped == false {
                                closed = true;
                            }
                        },
                        _ => { 
                            escaped = false;
                        }
                    }
                    index += 1;
                }

                if closed == false {
                    let mut message = String::from("Missing matching qutoes for string! -> ");
                    message.push_str(&schema[start..usize::min(end, start + 20)]);
                    return Err(NP_Error::new(message.as_str()))
                }

                Ok(JS_AST::string{ addr: AST_STR { start: moving_start, end: index} })
            }
        }
    }
}

#[test]
fn test() {
    println!("{:?}", JS_Schema::new(String::from("struct({key: string()})")));
}