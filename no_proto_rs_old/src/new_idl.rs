//! ES6 IDL for Schemas
//! 
//! Supports a *very* limited subset of ES6/Javascript parsing for schemas and rpcs.
//! 
use alloc::prelude::v1::Box;
use crate::error::NP_Error;
use alloc::string::String;
use alloc::vec::Vec;

/// Parsed AST String
#[derive(PartialEq, Clone, Copy, Debug)]
pub struct AST_STR { 
    start: usize, 
    end: usize 
}



/// AST object of schema
#[allow(missing_docs)]
#[derive(Debug)]
pub enum AST {
    colon,
    comma,
    arrow,
    token { addr: AST_STR },
    arrows { items: Vec<AST> },
    method { call: Box<AST>, result: Box<AST>},
    args { items: Vec<AST> },
    list { items: Vec<AST> },
    closure { items: Vec<AST> },
    string { addr: AST_STR },
    number { addr: AST_STR }
}

enum ast_cursor_state {
    searching,
    token,
    parens { open_idx: usize },
    arrows { open_idx: usize, },
    single_quote { open_idx: usize },
    double_quote { open_idx: usize },
    brackets { open_idx: usize },
    curly { open_idx: usize },
    number
}

struct ast_state {
    start: usize,
    end: usize,
    state: ast_cursor_state,
    escaped: bool,
    level: i16
}


impl AST {

    /// Convert an ASCII string into AST
    pub fn parse(input: &str) -> Result<Vec<Self>, NP_Error> {
        let mut result: Vec<Self> = Vec::new();
        AST::recursive_parse(0, &mut result, input, AST_STR { start: 0, end: input.len() })?;
        Ok(result)
    }

    /// Recursive AST parser
    pub fn recursive_parse(depth: usize, result: &mut Vec<AST>, source: &str, ast: AST_STR) -> Result<(), NP_Error> {

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

        let chars: Vec<char> = source.chars().collect();

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
                        'A'..='Z' => { // A - Z
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::token;
                        },
                        'a'..='z' => { // a - z
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::token;
                        },
                        '0'..='9' => {  // 0 - 9
                            cursor.start = cursor.end;
                            cursor.state = ast_cursor_state::number;
                        }
                        '{' => { // {
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::curly { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '(' => { // (
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::parens { open_idx: cursor.end };
                            cursor.level += 1;
                        }
                        '\'' => { // '
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::single_quote { open_idx: cursor.end };
                        }
                        '"' => { // "
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::double_quote { open_idx: cursor.end };
                        }
                        ':' => {  // :
                            result.push(AST::colon);
                        }
                        ',' => { // ,
                            result.push(AST::comma);
                        }
                        '-' => { // -
                            if cursor.end + 1 < ast.end && chars[cursor.end + 1] == '>' { // >
                                result.push(AST::arrow);
                                cursor.end +=1;
                            }
                        }
                        '<' => { // <
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::arrows { open_idx: cursor.end };
                        }
                        '[' => { // [
                            cursor.start = cursor.end + 1;
                            cursor.state = ast_cursor_state::brackets { open_idx: cursor.end };
                        }
                        '\n' | '\r' => { // new line

                        }
                        _ => {

                        }
                    }
                    
                }
                ast_cursor_state::number => {
                    if (*curr_char >= '0' && *curr_char <= '9') || *curr_char == '.' || *curr_char == '_' || *curr_char == '^' || *curr_char == 'e' || *curr_char == ',' {
                        // valid number chars (0 - 9 || . || _ || ^ || e)
                    } else {
                        result.push(AST::number { addr: AST_STR { start: cursor.start, end: cursor.end }});
                        cursor.state = ast_cursor_state::searching;
                        cursor.end -=1;
                    }
                },
                ast_cursor_state::arrows { .. } => {
                    if *curr_char == '<' { // <
                        cursor.level +=1;
                    }
                    if *curr_char == '>' { // >
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, source, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::arrows { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }

                },
                ast_cursor_state::curly { .. } => {
                    if *curr_char == '{' { // {
                        cursor.level +=1;
                    }
                    if *curr_char == '}' { // }
                        cursor.level -=1;
                    }

                    if cursor.level == 0 {
                        let mut parans_args: Vec<AST> = Vec::new();
                        AST::recursive_parse(depth + 1, &mut parans_args, source, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::closure { items: parans_args });
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
                        AST::recursive_parse(depth + 1, &mut parans_args, source, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::args { items: parans_args });
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
                    } else if cursor.end + 1 < source.len() && *curr_char == ':' && chars[cursor.end + 1] == ':' { // ::
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
                        AST::recursive_parse(depth + 1, &mut parans_args, source, AST_STR { start: cursor.start, end: cursor.end})?;
                        result.push(AST::list { items: parans_args });
                        cursor.state = ast_cursor_state::searching;
                    }
                }
            }
        

            cursor.end += 1;
        }

        match cursor.state {
            ast_cursor_state::searching => {}
            ast_cursor_state::brackets { open_idx } => {
                let mut error = String::from("AST Error: Missing matching closing square bracket!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_[_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
                return Err(NP_Error::Custom { message: error})    
            }
            ast_cursor_state::arrows { open_idx } => { 
                let mut error = String::from("AST Error: Missing matching closing angle bracket!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_<_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::parens { open_idx } => {
                let mut error = String::from("AST Error: Missing matching closing paranthasees!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_(_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::single_quote { open_idx } => { 
                let mut error = String::from("AST Error: Missing matching closing single quotes!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_'_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::double_quote { open_idx } => { 
                let mut error = String::from("AST Error: Missing matching closing double quotes!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_\"_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
                return Err(NP_Error::Custom { message: error})
            }
            ast_cursor_state::curly { open_idx } => { 
                let mut error = String::from("AST Error: Missing matching closing curly brackets!: ");
                error.push_str(&source[(usize::max(0, open_idx - 15))..open_idx]);
                error.push_str("_{_");
                error.push_str(&source[(open_idx+1)..usize::min(open_idx + 15, source.len())]);
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

#[test]
fn test() {
    // println!("HELLO {:?}", );

    let schema = String::from(r##"
    # comment here
    rpc get_user (id: 4) { user::this -> result { string, string } }
    # comment here
    
    "##);

    
    match AST::parse(&schema.clone()) {
        Ok(ast) => {
            println!("{:#?}", ast);
        },
        Err(e) => {
            println!("{:?}", e);
        }
    }
}