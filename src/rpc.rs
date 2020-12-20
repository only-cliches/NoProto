//! Remote Procedure Call APIs
//! 

use crate::utils::opt_err;
use crate::NP_Factory;
use crate::NP_Schema;
use alloc::prelude::v1::Box;
use crate::json_decode;
use alloc::string::String;
use alloc::vec::Vec;
use crate::{NP_JSON, buffer::NP_Buffer, error::NP_Error};

/// The different kinds of rpc functions
#[derive(Debug)]
pub enum RPC_Fn_Kinds {
    /// Normal function, doesn't return result or option
    normal,
    /// This function returns a result
    result,
    /// This function returns an option
    option
}

/// RPC Specifications
/// 
#[derive(Debug)]
pub enum NP_RPC_Spec {
    /// RPC Function
    RPC { 
        /// Function name
        name: String,
        /// Function module path
        module_path: String,
        /// Full path (module_path::name)
        full_name: String,
        /// RPC Message argument address 
        arg: usize, 
        /// RPC Message result address
        result: usize, 
        /// RPC message error address (f this is a result kind of function)
        err: usize, 
        /// The kind of function this is
        kind: RPC_Fn_Kinds 
    },
    /// RPC Message
    MSG { 
        /// Message name
        name: String, 
        /// Message module path
        module_path: String,
        /// Full path (module_path::name)
        full_name: String,
        /// Factory for this message
        factory: NP_Factory
    },
    /// RPC Module
    MOD { 
        /// Module name
        name: String, 
        /// Module path
        module_path: String 
    }
}

/// RPC Factory
#[derive(Debug)]
pub struct NP_RPC_Factory {
    /// Name of API
    pub name: String,
    /// API Author
    pub author: String,
    /// ID + version
    pub id: [u8; 19],
    /// Specification for this factory
    spec: NP_RPC_Specification
}

/// RPC Specification
#[derive(Debug)]
pub struct NP_RPC_Specification {
    /// Specification for this factory
    pub specs: Vec<NP_RPC_Spec>,
    /// Compiled spec
    pub compiled: Vec<u8>
}

struct Parsed_Fn {
    /// RPC Message argument address 
    pub arg: String,
    /// RPC Message result address
    pub result: String,
    /// RPC message error address (f this is a result kind of function)
    pub err: String,
    /// The kind of function this is
    pub kind: RPC_Fn_Kinds 
}

impl NP_RPC_Factory {

    /// Parse a JSON RPC spec into an RPC Factory
    /// 
    pub fn new(json_rcp_spec: &str) -> Result<Self, NP_Error> {

        let parsed = json_decode(String::from(json_rcp_spec))?;


        let mut spec = NP_RPC_Specification { specs: Vec::new(), compiled: Vec::new() };

        NP_RPC_Factory::parse_json_msg("mod", &parsed, &mut spec)?;
        NP_RPC_Factory::parse_json_rpc("", &parsed, &mut spec)?;

        let version = String::from(match &parsed["version"] { NP_JSON::String(version) => { version }, _ => { "" } }).split(".").map(|s| s.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>();
        let author_str = match &parsed["author"] { NP_JSON::String(author) => { author }, _ => { "" } };
        let id_str = String::from(match &parsed["id"] { NP_JSON::String(id) => { id }, _ => { "" } }).replace("-", "");
        let name_str = match &parsed["name"] { NP_JSON::String(name) => { name }, _ => { "" } };

        // id
        let mut id_bytes = [0u8; 19];
        for x in 0..16 {
            let step = x * 2;
            match u8::from_str_radix(&id_str[step..(step + 2)], 16) {
                Ok(b) => { id_bytes[x] = b },
                Err(_e) => {}
            }
        }

        // version
        id_bytes[16] = version[0];
        id_bytes[17] = version[1];
        id_bytes[18] = version[2];

        Ok(NP_RPC_Factory {
            name: String::from(name_str),
            author: String::from(author_str),
            id: id_bytes,
            spec: spec
        })
    }

    /// Parse RPC messages
    pub fn parse_json_msg(module: &str, json: &NP_JSON, spec: &mut NP_RPC_Specification) -> Result<(), NP_Error> {
        match &json["spec"] {
            NP_JSON::Array(json_spec) => {
                for jspec in json_spec.iter() {
                    match &jspec["msg"] { // msg type
                        NP_JSON::String(msg_name) => {
                            let schema = NP_Schema::from_json(Vec::new(), &Box::new(jspec.clone()))?;
                            let factory = NP_Factory {
                                schema: NP_Schema { is_sortable: schema.0, parsed: schema.2 },
                                schema_bytes: schema.1
                            };
                            spec.specs.push(NP_RPC_Spec::MSG { 
                                name: msg_name.clone(), 
                                module_path: String::from(module), 
                                full_name: format!("{}::{}", module, msg_name), 
                                factory: factory 
                            });
                        },
                        _ => {
                            match &jspec["mod"] { // module
                                NP_JSON::String(mod_name) => {
                                    let mut new_mod = String::from(module);
                                    new_mod.push_str("::");
                                    new_mod.push_str(mod_name);
                                    NP_RPC_Factory::parse_json_msg(&new_mod, &jspec, spec)?;
                                },
                                _ => {
                                 
                                }
                            }
                        }
                    }
                }
            },
            _ => { return Err(NP_Error::new("RPC Objects must have a 'spec' property!")) }
        }

        Ok(())
    }

    /// Parse RPC methods
    pub fn parse_json_rpc(module: &str, json: &NP_JSON, spec: &mut NP_RPC_Specification) -> Result<(), NP_Error> {
        match &json["spec"] {
            NP_JSON::Array(json_spec) => {
                for jspec in json_spec.iter() {
                    match &jspec["rpc"] { // rpc type
                        NP_JSON::String(rpc_name) => {
                            match &jspec["fn"] {
                                NP_JSON::String(fn_def) => {
                                    let parsed_def = NP_RPC_Factory::method_string_parse(module, fn_def)?;

                                    spec.specs.push(NP_RPC_Spec::RPC { 
                                        name: rpc_name.clone(),
                                        module_path: String::from(module),
                                        full_name: if module == "" { String::from(rpc_name) } else { format!("{}.{}", module, rpc_name) } ,
                                        arg: NP_RPC_Factory::find_msg(&parsed_def.arg, &spec)?,
                                        result: NP_RPC_Factory::find_msg(&parsed_def.result, &spec)?,
                                        err: NP_RPC_Factory::find_msg(&parsed_def.err, &spec)?,
                                        kind: parsed_def.kind 
                                    });
                                },
                                _ => return Err(NP_Error::new("RPC methods must have an 'fn' property!"))
                            }
                        },
                        _ => {
                            match &jspec["mod"] { // module
                                NP_JSON::String(mod_name) => {
                                    let mut new_mod = String::from(module);
                                    if module.len() > 0 {
                                        new_mod.push_str(".");
                                    }
                                    new_mod.push_str(mod_name);
                                    NP_RPC_Factory::parse_json_rpc(&new_mod, &jspec, spec)?;
                                },
                                _ => {
                                 
                                }
                            }
                        }
                    }
                }
            },
            _ => { return Err(NP_Error::new("RPC Objects must have a 'spec' property!")) }
        }

        Ok(())
    }

    fn find_msg(msg_name: &String, spec: &NP_RPC_Specification) -> Result<usize, NP_Error> {
        if msg_name == "" { return Ok(0) }

        let mut idx = 0usize;
        for msg in &spec.specs {
            match msg {
                NP_RPC_Spec::MSG { full_name, ..} => {
                    if full_name == msg_name {
                        return Ok(idx);
                    }
                },
                _ => {}
            }
            idx +=1;
        }
        let mut name = msg_name.clone();
        name.push_str("Can't find rpc message '");
        name.push_str(msg_name);
        name.push_str("'.");
        Err(NP_Error::new(name.as_str()))
    }
    
    /// Parse an FN method string into it's parts
    /// 
    /// Handle these different kinds of signatures:
    /// "(self::get) -> Result<self::get, self::error>"
    /// "(self::get) -> Option<self::get>"
    /// "(self::get) -> self::get"
    /// 
    fn method_string_parse(module: &str, function_str: &str) -> Result<Parsed_Fn, NP_Error> {
        let fn_kind = {
            if function_str.contains("Result<") {
                RPC_Fn_Kinds::result
            } else if function_str.contains("Option<") {
                RPC_Fn_Kinds::option
            } else {
                RPC_Fn_Kinds::normal
            }
        };

        let open_paren = opt_err(function_str.find("("))? + 1;
        let close_paren = opt_err(function_str.find(")"))?;

        let arg_name = function_str[open_paren..close_paren].trim();

        let after_arrow = opt_err(function_str.find("->"))? + 2;
        let return_name = function_str[after_arrow..].trim();

        match &fn_kind {
            RPC_Fn_Kinds::normal => {
                Ok(Parsed_Fn { arg: String::from(arg_name).replace("self", module), result: String::from(return_name).replace("self", module), err: String::from(""), kind: fn_kind})
            },
            RPC_Fn_Kinds::option => {
                let open = opt_err(return_name.find("<"))? + 1;
                let close = opt_err(return_name.find(">"))?;
                Ok(Parsed_Fn { arg: String::from(arg_name).replace("self", module), result: String::from(&return_name[open..close]).replace("self", module), err: String::from(""), kind: fn_kind})
            },
            RPC_Fn_Kinds::result => {
                let open = opt_err(return_name.find("<"))? + 1;
                let close = opt_err(return_name.find(">"))?;
                let results = &return_name[open..close];
                let comma = opt_err(results.find(","))?;
                Ok(Parsed_Fn { arg: String::from(arg_name).replace("self", module), result: String::from(results[..comma].trim()).replace("self", module), err: String::from(results[(comma+1)..].trim()).replace("self", module), kind: fn_kind})
            },
        }

    }
/*
    /// Parse a byte rpc spec into an RPC Factory
    /// 
    pub fn new_compiled(bytes_rpc_spec: Vec<u8>) -> Result<Self, NP_Error>  {
        todo!()
    }

    /// Get a copy of the compiled byte array spec
    /// 
    pub fn compile_schema(&self) -> Vec<u8> {
        // self.spec.compiled.clone()
        todo!()
    }

    /// Export the spec of this factory to JSON.
    /// 
    pub fn export_spec(&self) -> Result<NP_JSON, NP_Error> {
        todo!()
    }
*/
    /// Generate a new request object for a given rpc function
    /// 
    pub fn new_request(&self, rpc_name: &str) -> Result<NP_RPC_Request, NP_Error> {
        let mut idx = 0usize;
        for spec in &self.spec.specs {
            match spec {
                NP_RPC_Spec::RPC { full_name, arg,   .. } => {
                    if full_name == rpc_name {
                        return Ok(NP_RPC_Request {
                            rpc_addr: idx,
                            id: self.id,
                            spec: &self.spec,
                            rpc: full_name,
                            data: match &self.spec.specs[*arg] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::new("unreachable"))
                            }
                        })
                    }
                },
                _ => { }
            }
            idx +=1;
        }

        Err(NP_Error::new("Cannot find request!"))
    }

    /// Open a request.  The request spec and version must match the current spec and version of this factory.
    /// 
    pub fn open_request(&self, bytes: Vec<u8>) -> Result<NP_RPC_Request, NP_Error> {
        todo!()
    }

    /// Generate a new response object for a given rpc function
    /// 
    pub fn new_response(&self, rpc_name: &str) -> Result<NP_RPC_Response, NP_Error> {
        let mut idx = 0usize;
        for spec in &self.spec.specs {
            match spec {
                NP_RPC_Spec::RPC { full_name, result, err,   .. } => {
                    if full_name == rpc_name {
                        return Ok(NP_RPC_Response {
                            rpc_addr: idx,
                            rpc: full_name,
                            id: self.id,
                            data: match &self.spec.specs[*result] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::new("unreachable"))
                            },
                            error: match &self.spec.specs[*err] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::new("unreachable"))
                            }
                        })
                    }
                },
                _ => { }
            }
            idx +=1;
        }

        Err(NP_Error::new("Cannot find response!"))
    }

    /// Open a response.  The response spec and version must match the current spec and version of this factory.
    /// 
    pub fn open_response(&self, bytes: Vec<u8>) -> Result<Option<NP_RPC_Response>, NP_Error> {
        todo!()
    }

    /// Open a response that is a result type.  The response spec and version must match the current spec and version of this factory.
    /// 
    pub fn open_response_result(&self, bytes: Vec<u8>) -> Result<Result<NP_RPC_Response, NP_RPC_Response>, NP_Error> {
        todo!()
    }

}

#[derive(Debug)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum ResponseType {
    OKorSome,
    Error,
    None
}

impl From<u8> for ResponseType {
    fn from(value: u8) -> Self {
        if value > 2 { return ResponseType::OKorSome; }
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum RPC_Type {
    None,
    Request,
    Response
}

impl From<u8> for RPC_Type {
    fn from(value: u8) -> Self {
        if value > 2 { return RPC_Type::None; }
        unsafe { core::mem::transmute(value) }
    }
}

/// RPC Request object
pub struct NP_RPC_Request<'request> {
    /// the address for this rcp message
    rpc_addr: usize,
    /// Parent spec object
    spec: &'request NP_RPC_Specification,
    /// ID of API 
    id: [u8; 19],
    /// the name of the rpc function
    pub rpc: &'request str,
    /// the request data
    pub data: NP_Buffer<'request>
}

impl<'request> NP_RPC_Request<'request> {
    /// Get empty response for this request
    pub fn get_response(&self) -> Result<NP_RPC_Response, NP_Error> {
        match &self.spec.specs[self.rpc_addr] {
            NP_RPC_Spec::RPC { full_name, result, err, .. } => {
                return Ok(NP_RPC_Response {
                    rpc_addr: self.rpc_addr,
                    rpc: full_name,
                    id: self.id,
                    data: match &self.spec.specs[*result] {
                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                        _ => return Err(NP_Error::new("unreachable"))
                    },
                    error: match &self.spec.specs[*err] {
                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                        _ => return Err(NP_Error::new("unreachable"))
                    }
                })
            },
            _ => { }
        };

        Err(NP_Error::new("Response not found!"))
    }
    /// Close this request and get bytes
    pub fn rpc_close(self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 3);

        response_bytes.extend_from_slice(&self.id);
        response_bytes.extend_from_slice(&(self.rpc_addr as u16).to_be_bytes());
        response_bytes.push(RPC_Type::Request as u8);
        response_bytes.extend(self.data.close());

        response_bytes
    }
}
/// RPC Response object
pub struct NP_RPC_Response<'response> {
    /// the address for this rpc message
    rpc_addr: usize,
    /// ID of API 
    id: [u8; 19],
    /// the name of the rpc function
    pub rpc: &'response str,
    /// the data of this response
    pub data: NP_Buffer<'response>,
    /// if this is an error, the error data
    pub error: NP_Buffer<'response>
}



impl<'request> NP_RPC_Response<'request> {
    /// Close this response as Ok() or Some()
    pub fn rpc_close(self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 4);

        response_bytes.extend_from_slice(&self.id);
        response_bytes.extend_from_slice(&(self.rpc_addr as u16).to_be_bytes());
        response_bytes.push(RPC_Type::Response as u8);
        response_bytes.push(ResponseType::OKorSome as u8);
        response_bytes.extend(self.data.close());

        response_bytes
    }
    /// Close this response as Err()
    pub fn rpc_close_error(self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 4);

        response_bytes.extend_from_slice(&self.id);
        response_bytes.extend_from_slice(&(self.rpc_addr as u16).to_be_bytes());
        response_bytes.push(RPC_Type::Response as u8);
        response_bytes.push(ResponseType::Error as u8);
        response_bytes.extend(self.error.close());

        response_bytes
    }
    /// Close this response as None()
    pub fn rpc_close_none(self) -> Vec<u8> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 4);

        response_bytes.extend_from_slice(&self.id);
        response_bytes.extend_from_slice(&(self.rpc_addr as u16).to_be_bytes());
        response_bytes.push(RPC_Type::Response as u8);
        response_bytes.push(ResponseType::None as u8);
 
        response_bytes
    }
}