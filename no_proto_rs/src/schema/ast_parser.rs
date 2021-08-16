//! IDL Parser
//! 
//! Supports a custom IDL that is very similar to Rust syntax for data types.
//! 
use alloc::prelude::v1::Box;
use crate::error::NP_Error;
use alloc::string::String;
use alloc::vec::Vec;
use core::str;

/// Parsed AST String
#[allow(dead_code)]
#[derive(Default, PartialEq, Clone, Copy, Debug)]
pub struct AST_STR { 
    pub start: usize,
    pub end: usize
}

#[allow(dead_code)]
impl AST_STR {
    pub fn read<'read>(&self, source: &'read str) -> &'read str {
        &source[self.start..self.end]
    }
    pub fn read_bytes<'read>(&self, source: &'read [u8]) -> &'read str {
        unsafe { str::from_utf8_unchecked(&source[self.start..self.end])}
    }
}

// how many charecters to show before and after error location
#[allow(dead_code)]
const AST_ERROR_RANGE: usize = 20;

/// AST object of schema
#[allow(missing_docs)]
#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub enum AST {
    colon,
    comma,
    arrow,
    semicolon,
    newline,
    token { addr: AST_STR },
    xml { items: Vec<AST> },
    method { call: Box<AST>, result: Box<AST>},
    parans { items: Vec<AST> },
    square { items: Vec<AST> },
    curly { items: Vec<AST> },
    string { addr: AST_STR },
    number { addr: AST_STR }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
enum ast_cursor_state {
    searching,
    token,
    parens { open_idx: usize },
    xml { open_idx: usize, },
    single_quote { open_idx: usize },
    double_quote { open_idx: usize },
    brackets { open_idx: usize },
    curly { open_idx: usize },
    number
}

#[derive(Debug, Clone)]
struct ast_state {
    start: usize,
    end: usize,
    state: ast_cursor_state,
    escaped: bool,
    level: i16
}


#[allow(dead_code)]
impl AST {

    /// Convert an ASCII string into AST
    pub fn parse(input: &str) -> Result<Vec<Self>, NP_Error> {
        let mut result: Vec<Self> = Vec::new();
        let src_chars: Vec<char> = input.chars().collect();

        AST::recursive_parse(0, &mut result, &src_chars, AST_STR { start: 0, end: input.len() })?;
        Ok(result)
    }

    /// Recursive AST parser
    fn recursive_parse(depth: usize, result: &mut Vec<AST>, chars: &Vec<char>, ast: AST_STR) -> Result<(), NP_Error> {

        if depth > 255 {
            return Err(NP_Error::RecursionLimit)
        }

        let mut cursor = ast_state { 
            start: ast.start, 
            end: ast.start, 
            state: ast_cursor_state::searching,
            escaped: false,
            level: 0
        };

        while cursor.end < ast.end {
            let mut curr_char: &char = &chars[cursor.end];

            if *curr_char == '#' || (cursor.end + 1 < ast.end && *curr_char == '/' && chars[cursor.end + 1] == '/') { // # or //
                while *curr_char != '\n' && *curr_char != '\r' && cursor.end < ast.end { // new line
                    curr_char = &chars[cursor.end];
                    cursor.end += 1;
                }
            }

            match cursor.state {
                ast_cursor_state::searching => {

                    match *curr_char {
                        'A'..='Z' => {
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::token;
                        },
                        'a'..='z' => {
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::token;
                        },
                        '0'..='9' => {
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::number;
                        },
                        '-' => {
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::number;
                        }
                        '{' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::curly { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '(' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::parens { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '\'' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::single_quote { open_idx: cursor.end };
                        }
                        '"' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::double_quote { open_idx: cursor.end };
                        }
                        ';' => {
                            result.push(AST::semicolon);
                        },
                        ':' => {
                            result.push(AST::colon);
                        }
                        ',' => {
                            result.push(AST::comma);
                        }
                        '-' => {
                            if cursor.end + 1 < ast.end && chars[cursor.end + 1] == '>' { // >
                                result.push(AST::arrow);
                                cursor.end +=1;
                            }
                        }
                        '<' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::xml { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '[' => {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::brackets { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '\n' | '\r' => { // new line
                            let len = result.len();
                            if len > 0 && result[len - 1] != AST::newline {
                                result.push(AST::newline);
                            }
                        }
                        '}' => {
                            let src_str: String = chars.iter().collect();
                            let mut error = String::from("AST Error: Unexpected closing curly bracket!: ");
                            error.push_str(&src_str.as_str()[(usize::max(0, cursor.end - AST_ERROR_RANGE))..cursor.end]);
                            error.push_str("_}_");
                            error.push_str(&src_str.as_str()[(cursor.end+1)..usize::min(cursor.end + AST_ERROR_RANGE, chars.len())]);
                            return Err(NP_Error::Custom { message: error})
                        },
                        ']' => {
                            let src_str: String = chars.iter().collect();
                            let mut error = String::from("AST Error: Unexpected closing square bracket!: ");
                            error.push_str(&src_str.as_str()[(usize::max(0, cursor.end - AST_ERROR_RANGE))..cursor.end]);
                            error.push_str("_]_");
                            error.push_str(&src_str.as_str()[(cursor.end+1)..usize::min(cursor.end + AST_ERROR_RANGE, chars.len())]);
                            return Err(NP_Error::Custom { message: error})
                        },
                        ')' => {
                            let src_str: String = chars.iter().collect();
                            let mut error = String::from("AST Error: Unexpected closing parentheses!: ");
                            error.push_str(&src_str.as_str()[(usize::max(0, cursor.end - AST_ERROR_RANGE))..cursor.end]);
                            error.push_str("_)_");
                            error.push_str(&src_str.as_str()[(cursor.end+1)..usize::min(cursor.end + AST_ERROR_RANGE, chars.len())]);
                            return Err(NP_Error::Custom { message: error})
                        },
                        '>' => {
                            let src_str: String = chars.iter().collect();
                            let mut error = String::from("AST Error: Unexpected closing angle bracket!: ");
                            error.push_str(&src_str.as_str()[(usize::max(0, cursor.end - AST_ERROR_RANGE))..cursor.end]);
                            error.push_str("_)_");
                            error.push_str(&src_str.as_str()[(cursor.end+1)..usize::min(cursor.end + AST_ERROR_RANGE, chars.len())]);
                            return Err(NP_Error::Custom { message: error})
                        }
                        _ => {}
                    }
                    
                }
                ast_cursor_state::number => {
                    if (*curr_char >= '0' && *curr_char <= '9') || *curr_char == '.' || *curr_char == '_' || *curr_char == '^' || *curr_char == 'e' || *curr_char == '-' {
                        // valid number chars (0 - 9 || . || _ || ^ || e || -)
                    } else {
                        result.push(AST::number { addr: AST_STR { start: cursor.start, end: cursor.end }});
                        cursor.state = ast_cursor_state::searching;
                        cursor.end -= 1;
                    }
                }
                ast_cursor_state::xml { .. } => {
                    if *curr_char == '<' { // <
                        cursor.level +=1;
                    }
                    if *curr_char == '>' { // >
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, chars, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::xml { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }

                }
                ast_cursor_state::curly { .. } => {
                    if *curr_char == '{' { // {
                        cursor.level +=1;
                    }
                    if *curr_char == '}' { // }
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, chars, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::curly { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }

                },
                ast_cursor_state::parens { .. } => {
                    if *curr_char == '(' { // (
                        cursor.level +=1;
                    }
                    if *curr_char == ')' { // )
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, chars, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::parans { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }

                }
                ast_cursor_state::double_quote { .. } => {

                    if *curr_char == '"' && cursor.escaped == false {
                        result.push(AST::string { addr: AST_STR { start: cursor.start, end: cursor.end } });
                        cursor.state = ast_cursor_state::searching;
                    }                    

                    if *curr_char == '\\' { // '\'
                        cursor.escaped = true;
                    } else {
                        cursor.escaped = false;
                    }
                },
                ast_cursor_state::single_quote { .. } => {

                    if *curr_char == '\'' && cursor.escaped == false {
                        result.push(AST::string { addr: AST_STR { start: cursor.start, end: cursor.end } });
                        cursor.state = ast_cursor_state::searching;
                    }        

                    if *curr_char == '\\' { // '\'
                        cursor.escaped = true;
                    } else {
                        cursor.escaped = false;
                    }
                },
                ast_cursor_state::token => {
                    if (*curr_char >= 'a' && *curr_char <= 'z') || (*curr_char >= 'A' && *curr_char <= 'Z') || (*curr_char >= '0' && *curr_char <= '9') || *curr_char == '_' || *curr_char == '-' {
                        // valid token chars (a - z | A - Z | 0 - 9 | _ | - )
                    } else if cursor.end + 1 < chars.len() && *curr_char == ':' && chars[cursor.end + 1] == ':' { // ::
                        cursor.end += 1;
                    } else { // end of token
                        result.push(AST::token { addr: AST_STR { start: cursor.start, end: cursor.end }});
                        cursor.state = ast_cursor_state::searching;
                        cursor.end -=1;
                    }
                }
                ast_cursor_state::brackets { .. } => {
                    if *curr_char == '[' { // [
                        cursor.level +=1;
                    }
                    if *curr_char == ']' { // ]
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, chars, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::square { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }
                }
            }


            cursor.end += 1;
        }

        match cursor.state {
            ast_cursor_state::searching => {}
            ast_cursor_state::brackets { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing square bracket!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_[_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})    
            }
            ast_cursor_state::xml { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing angle bracket!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_<_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::parens { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing paranthasees!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_(_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::single_quote { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing single quotes!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_'_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::double_quote { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing double quotes!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_\"_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::curly { open_idx } => {
                let src_str: String = chars.iter().collect();
                let mut error = String::from("AST Error: Missing matching closing curly brackets!: ");
                error.push_str(&src_str.as_str()[(usize::max(0, open_idx - AST_ERROR_RANGE))..open_idx]);
                error.push_str("_{_");
                error.push_str(&src_str.as_str()[(open_idx+1)..usize::min(open_idx + AST_ERROR_RANGE, chars.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::token => {
                result.push(AST::token { addr: AST_STR { start: cursor.start, end: cursor.end }});
            }
            ast_cursor_state::number => {
                result.push(AST::number { addr: AST_STR { start: cursor.start, end: cursor.end }});
            }
        }

        Ok(())
    }
}

// #[test]
// fn test() {
//     // println!("HELLO {:?}", );
//
//     let schema = String::from(r##"
//
//
//
//         info [
//             title: "My Protocol",
//             author: "Scott Lott",
//             version: 1.0,
//             id: "481cfd47-5b6f-422c-9e0c-9d561e6c94d1"
//         ]
//
//         enum Result<X,Y> [id: 0, default: "Unset"] {
//             Unset,
//             Ok(X),
//             Error(Y)
//         }
//
//         struct user [id: 1] {
//             id: ulid,
//             name: string,
//             email: string,
//             something: [u32; 12]
//         }
//
//
//         # comment here
//         impl user {
//             get(id: u32) -> self,
//             update(self) -> Result<(), Error>
//         }
//         # comment here
//         // comment here
//         enum cursor_state<X> [id: 3] {
//             option1(arg1, X),
//             option2 { key: value }
//         }
//     "##);
//
//
//     match AST::parse(&schema.clone()) {
//         Ok(ast) => {
//             println!("{:#?}", ast);
//         },
//         Err(e) => {
//             println!("{:?}", e);
//         }
//     }
// }