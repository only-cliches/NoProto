//! Remote Procedure Call APIs
//! 
//! You can optionally omit all the RPC related code with `features = []` in your cargo.toml
//! 
//! The NoProto RPC framework builds on top of NoProto's format and Rust's conventions to provide a flexible, powerful and safe RPC protocol.
//! 
//! This RPC framework has *zero* transport code and is transport agnostic.  You can send bytes between the server/client using any method you'd like.
//! 
//! It's also possible to send messages in either direction, the Client & Server both have the ability to encode/decode requests and responses.
//! 
//! # RPC JSON Spec
//! 
//! Before you can send bytes between servers and clients, you must let NoProto know the shape and format of your endpoints, requests and responses.  Like schemas, RPC specs are written as JSON.
//! 
//! Any fields in your spec not required by the library will simply be ignored.
//! 
//! ## Required Fields
//! 
//! ### id, version
//! The `id` property should be a V4 UUID you've generated yourself. This [website](https://www.uuidgenerator.net/version4) can help generate a UUID for you. The `version` property should be a semver string like `0.1.0` or `1.0.0` or `0.0.23`.
//! 
//! The `id` and `version` data is encoded in every request and response.  If you attempt to open a request or response that does not match the `version` and `id` of the specification you're using, the request/response will fail to open.
//! 
//! ### name
//! The `name` property is the title for your specification.  Should be something appropriate like "Todo App RPC" or "User Account RPC".
//! 
//! ### author
//! The `author` property is a string and can contain any value. You can put your name here, your companies name, your email, whatever you'd like.
//! 
//! ### spec
//! Is an array of RPC specifications described below, this is the root of your specifications.  The array should be at property `spec`.
//! 
//! ## RPC Specifications
//! 
//! There are 4 different kinds of values allowed in the `spec` array.  They can be in any order and you can have as many of each type as you'd like.
//! 
//! 
//! #### 1. Message
//! RPC messages are named NoProto schemas.  They must have a `msg` property with the name of the schema, then a `type` property for the schema type.  The messages MUST be valid NoProto schemas.
//! 
//! ```text
//! // Some valid messages
//! {"msg": "user_id", "type": "u32"}
//! 
//! {"msg": "address", "type": "table", "columns": [
//!     ["street", {"type": "string"}],
//!     ["city", {"type": "string"}]
//! ]}
//! 
//! {"msg": "tags", "type": "list", "of": {"type": "string"}}
//! ```
//! 
//! #### 2. RPC Method
//! Methods are named endpoints with arguments and responses.  The arguments and responses MUST reference messages.  They always contain a `rpc` property and an `fn` property which describes the endpoint arguments and return types.
//! 
//! RPC methods can have between 0 and 1 arguments and can return nothing, a value T, an option&#60;T&#62; or, Result&#60;T,E&#62;
//! ```text
//! // Some valid RPC methods
//! {"rpc": "get_count", "fn": "() -> self::count"}
//! 
//! {"rpc": "get_user", "fn": "(self::user_id) -> Option<self::user>"}
//! 
//! {"rpc": "del_user", "fn": "(self::user_id) -> Result<(), self::error>"}
//! 
//! {"rpc": "add_one", "fn": "(self::add_arg) -> Result<self::add_arg, self::error>"}
//! 
//! {"rpc": "trigger_action", "fn": "() -> ()"}
//! ```
//! 
//! #### 3. RPC Module
//! You can create nested namespaces inside your specification that contain their own specification.  Namespaces require a `mod` property and `spec` property.
//! 
//! ```text
//! // a valid RPC module
//! {"mod": "user", "spec": [
//!     {"msg": "user_id", "type": "u32"},
//!     {"msg": "user_name", "type": "string"},
//!     {"rpc": "get_username", "fn": "(self::user_id) -> Option<self::user_name>"}
//! ]}
//! ```
//! 
//! #### 4. Comments
//! You can insert string comments anywhere in your spec.
//! 
//! ### RPC Namespaces & Modules
//! 
//! I'm sure you've noticed the `self` being used above in the function definitions.  You can create messages anywhere in your specification and they can be accessed by any RPC method in any namespace using the namespace syntax.
//! 
//! Methods can always access messages in their own namespace using `self`.  Otherwise, the top of the name space is `mod` and messages in other namespaces can be used by their names.  For example, let's say we had a message named `delete` inside the `user` RPC module and a further `modify` module inside that.  That message could be accessed by any RPC method with `mod::user::modify::delete`.
//! 
//! That might be confusing so here's an example RPC spec with some fancy namespacing.
//! 
//! ## Example RPC JSON SPEC
//! 
//! ```text
//! {
//!     "name": "TEST API",
//!     "author": "Jeb Kermin",
//!     "id": "cc419a66-9bbe-48db-ad1c-e0ffa2a2376f",
//!     "version": "1.0.0",
//!     "spec": [
//!         {"msg": "Error", "type": "string" },
//!         {"msg": "Count", "type": "u32" },
//!         "this is a comment",
//!         {"rpc": "get_count", "fn": "() -> self::Count"},
//!         {"mod": "user", "spec": [
//!             {"msg": "username", "type": "string"},
//!             {"msg": "user_id", "type": "u32"},
//!             {"rpc": "get_user_id", "fn": "(self::username) -> Option<self::user_id>"},
//!             {"rpc": "del_user", "fn": "(self::user_id) -> Result<self::user_id, mod::Error>"},
//!             {"mod": "admin", "spec": [
//!                 {"rpc": "update_user", "fn": "(mod::user::user_id) -> Result<(), mod::Error>"}
//!             ]}
//!         ]}
//!     ]
//! }
//! ```
//! 
//! 
//! # Using the RPC Framework
//! 
//! ```rust
//! use no_proto::rpc::{NP_RPC_Factory, NP_ResponseKinds, NP_RPC_Response, NP_RPC_Request};
//! use no_proto::error::NP_Error;
//! 
//! // You can generate an RPC Factory with this method.
//! // Like NoProto Factories, this RPC factory can be used to encode/decode any number of requests/responses.
//! 
//! let rpc_factory = NP_RPC_Factory::new(r#"{
//!     "name": "Test API",
//!     "author": "Jeb Kermin",
//!     "id": "cc419a66-9bbe-48db-ad1c-e0ffa2a2376f",
//!     "version": "1.0.0",
//!     "spec": [
//!         {"msg": "Error", "type": "string" },
//!         {"msg": "Count", "type": "u32" },
//!         {"rpc": "get_count", "fn": "() -> self::Count"},
//!         {"mod": "user", "spec": [
//!             {"msg": "username", "type": "string"},
//!             {"msg": "user_id", "type": "u32"},
//!             {"rpc": "get_user_id", "fn": "(self::username) -> Option<self::user_id>"},
//!             {"rpc": "del_user", "fn": "(self::user_id) -> Result<self::user_id, mod::Error>"},
//!         ]}
//!     ]
//! }"#)?;
//! 
//! // rpc_factory should be initilized on server and client using an identical JSON RPC SPEC
//! // Both server and client can encode/decode responses and requests so the examples below are only a convention.
//! 
//! 
//! 
//! // SIMPLE EXAMPLE
//! 
//! // === CLIENT ===
//! // generate request
//! let get_count: NP_RPC_Request = rpc_factory.new_request("get_count")?;
//! // close request (request has no arguments)
//! let count_req_bytes: Vec<u8> = get_count.rpc_close();
//!
//! // === SEND count_req_bytes to SERVER ===
//!
//! // === SERVER ===
//! // ingest request
//! let a_request: NP_RPC_Request = rpc_factory.open_request(count_req_bytes)?;
//! assert_eq!(a_request.rpc, "get_count");
//! // generate a response
//! let mut count_response: NP_RPC_Response = a_request.new_response()?;
//! // set response data
//! count_response.data.set(&[], 20u32)?;
//! // set response kind
//! count_response.kind = NP_ResponseKinds::Ok;
//! // close response
//! let respond_bytes = count_response.rpc_close()?;
//!
//! // === SEND respond_bytes to CLIENT ====
//!
//! // === CLIENT ===
//! let count_response = rpc_factory.open_response(respond_bytes)?;
//! // confirm our response matches the same request RPC we sent
//! assert_eq!(count_response.rpc, "get_count");
//! // confirm that we got data in the response
//! assert_eq!(count_response.kind, NP_ResponseKinds::Ok);
//! // confirm it's the same data the server sent
//! assert_eq!(count_response.data.get(&[])?, Some(20u32));
//! 
//! 
//! 
//! // RESULT EXAMPLE
//! 
//! // === CLIENT ===
//! // generate request
//! let mut del_user: NP_RPC_Request = rpc_factory.new_request("user.del_user")?;
//! del_user.data.set(&[], 50u32)?;
//! let del_user_bytes: Vec<u8> = del_user.rpc_close();
//!
//! // === SEND del_user_bytes to SERVER ===
//!
//! // === SERVER ===
//! // ingest request
//! let a_request: NP_RPC_Request = rpc_factory.open_request(del_user_bytes)?;
//! assert_eq!(a_request.rpc, "user.del_user");
//! // generate a response
//! let mut del_response: NP_RPC_Response = a_request.new_response()?;
//! // set response as ok with data
//! del_response.data.set(&[], 50u32)?;
//! del_response.kind = NP_ResponseKinds::Ok;
//! // close response
//! let respond_bytes = del_response.rpc_close()?;
//!
//! // === SEND respond_bytes to CLIENT ====
//!
//! // === CLIENT ===
//! let del_response = rpc_factory.open_response(respond_bytes)?;
//! // confirm our response matches the same request RPC we sent
//! assert_eq!(del_response.rpc, "user.del_user");
//! // confirm that we got data in the response
//! assert_eq!(del_response.kind, NP_ResponseKinds::Ok);
//! // confirm it's the same data set on the server
//! assert_eq!(del_response.data.get(&[])?, Some(50u32));
//! 
//! 
//! 
//! // RESULT EXAMPLE 2
//! 
//! // === CLIENT ===
//! // generate request
//! let mut del_user: NP_RPC_Request = rpc_factory.new_request("user.del_user")?;
//! del_user.data.set(&[], 50u32)?;
//! let del_user_bytes: Vec<u8> = del_user.rpc_close();
//! 
//! // === SEND del_user_bytes to SERVER ===
//! 
//! // === SERVER ===
//! // ingest request
//! let a_request: NP_RPC_Request = rpc_factory.open_request(del_user_bytes)?;
//! assert_eq!(a_request.rpc, "user.del_user");
//! // generate a response
//! let mut del_response: NP_RPC_Response = a_request.new_response()?;
//! // set response as error
//! del_response.error.set(&[], "Can't find user.")?;
//! del_response.kind = NP_ResponseKinds::Error;
//! // close response
//! let respond_bytes = del_response.rpc_close()?;
//! 
//! // === SEND respond_bytes to CLIENT ====
//! 
//! // === CLIENT ===
//! let del_response = rpc_factory.open_response(respond_bytes)?;
//! // confirm our response matches the same request RPC we sent
//! assert_eq!(del_response.rpc, "user.del_user");
//! // confirm we recieved error response
//! assert_eq!(del_response.kind, NP_ResponseKinds::Error);
//! // get the error information
//! assert_eq!(del_response.error.get(&[])?, Some("Can't find user."));
//! 
//! 
//! 
//! // OPTION EXAMPLE
//! 
//! // === CLIENT ===
//! // generate request
//! let mut get_user: NP_RPC_Request = rpc_factory.new_request("user.get_user_id")?;
//! get_user.data.set(&[], "username")?;
//! let get_user_bytes: Vec<u8> = get_user.rpc_close();
//! 
//! // === SEND get_user_bytes to SERVER ===
//! 
//! // === SERVER ===
//! // ingest request
//! let a_request: NP_RPC_Request = rpc_factory.open_request(get_user_bytes)?;
//! assert_eq!(a_request.rpc, "user.get_user_id");
//! // generate a response
//! let mut del_response: NP_RPC_Response = a_request.new_response()?;
//! // set response as none
//! del_response.kind = NP_ResponseKinds::None;
//! // close response
//! let respond_bytes = del_response.rpc_close()?;
//! 
//! // === SEND respond_bytes to CLIENT ====
//! 
//! // === CLIENT ===
//! let del_response = rpc_factory.open_response(respond_bytes)?;
//! // confirm our response matches the same request RPC we sent
//! assert_eq!(del_response.rpc, "user.get_user_id");
//! // confirm that we got data in the response
//! assert_eq!(del_response.kind, NP_ResponseKinds::None);
//! // with NONE response there is no data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! 

use crate::{hashmap::NP_HashMap, utils::opt_err};
use crate::NP_Factory;
use crate::NP_Schema;
use alloc::prelude::v1::Box;
use crate::json_decode;
use alloc::string::String;
use alloc::vec::Vec;
use crate::{NP_JSON, buffer::NP_Buffer, error::NP_Error};


/// The different kinds of rpc functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[doc(hidden)]
#[repr(u8)]
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
#[doc(hidden)]
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
        arg: Option<usize>, 
        /// RPC Message result address
        result: Option<usize>, 
        /// RPC message error address (f this is a result kind of function)
        err: Option<usize>, 
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
    spec: NP_RPC_Specification,
    method_hash: NP_HashMap,
    /// blank buffer
    empty: NP_Factory
}

/// RPC Specification
#[derive(Debug)]
#[doc(hidden)]
pub struct NP_RPC_Specification {
    /// Specification for this factory
    pub specs: Vec<NP_RPC_Spec>,
    /// Compiled spec
    pub compiled: Vec<u8>,
    owned_strings: Vec<String>,
    /// Message HashMap
    message_hash: NP_HashMap
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

        let mut spec = NP_RPC_Specification { specs: Vec::with_capacity(1024), compiled: Vec::with_capacity(1024), message_hash: NP_HashMap::new(), owned_strings: Vec::with_capacity(1024) };

        // first 2 bytes contains the offset of the first rpc method
        spec.compiled.extend_from_slice(&0u16.to_be_bytes());

        if name_str.len() > core::u16::MAX as usize {
            return Err(NP_Error::new("API name cannot be longer than 2^16 UTF8 bytes"));
        }
        
        // next bytes are name
        spec.compiled.extend_from_slice(&(name_str.len() as u16).to_be_bytes());
        spec.compiled.extend_from_slice(&name_str.as_bytes());

        // next 19 bytes are version
        spec.compiled.extend_from_slice(&id_bytes);

        // now the messages
        NP_RPC_Factory::parse_json_msg("mod", &parsed, &mut spec)?;
        if spec.compiled.len() > core::u16::MAX as usize {
            return Err(NP_Error::new("Too many messages in spec, can't compile."))
        }
        // set first 2 bytes to correct offset after we've inserted all messages
        for (x, b) in (spec.compiled.len() as u16).to_be_bytes().iter().enumerate() {
            spec.compiled[x] = *b;
        }
        // and finally the methods
        NP_RPC_Factory::parse_json_rpc("", "mod", &parsed, &mut spec)?;

        let mut method_hash: NP_HashMap = NP_HashMap::new();

        for (idx, one_spec) in spec.specs.iter().enumerate() {
            match one_spec {
                NP_RPC_Spec::RPC { full_name, .. } => {
                    method_hash.insert(full_name, idx)?;
                },
                _ => {}
            }
        }

        Ok(Self {
            name: String::from(name_str),
            author: String::from(author_str),
            method_hash,
            id: id_bytes,
            spec: spec,
            empty: NP_Factory::new_compiled([0u8].to_vec())
        })
    }

    /// Parse RPC messages
    fn parse_json_msg(module: &str, json: &NP_JSON, spec: &mut NP_RPC_Specification) -> Result<(), NP_Error> {
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
                            let full_name = format!("{}::{}", module, msg_name);

                            // insert this message address
                            spec.message_hash.insert(&full_name, spec.compiled.len())?;

                            let schema = factory.compile_schema();
                            spec.compiled.extend_from_slice(&(schema.len() as u16).to_be_bytes());
                            spec.compiled.extend(schema);

                            spec.specs.push(NP_RPC_Spec::MSG { 
                                name: msg_name.clone(), 
                                module_path: String::from(module), 
                                full_name, 
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
    fn parse_json_rpc(module: &str, msg_module: &str, json: &NP_JSON, spec: &mut NP_RPC_Specification) -> Result<(), NP_Error> {
        match &json["spec"] {
            NP_JSON::Array(json_spec) => {
                for jspec in json_spec.iter() {
                    match &jspec["rpc"] { // rpc type
                        NP_JSON::String(rpc_name) => {
                            match &jspec["fn"] {
                                NP_JSON::String(fn_def) => {
                                    let parsed_def = NP_RPC_Factory::method_string_parse(msg_module, fn_def)?;

                                    let full_name = if module == "" { String::from(rpc_name) } else { format!("{}.{}", module, rpc_name) };

                                    // compile the RPC spec
                                    spec.compiled.extend_from_slice(&(full_name.len() as u16).to_be_bytes());
                                    spec.compiled.extend_from_slice(&full_name.as_bytes());
                                    spec.compiled.push(parsed_def.kind as u8);

                                    if parsed_def.arg.len() == 0 { 
                                        spec.compiled.extend_from_slice(&0u16.to_be_bytes());
                                    } else {
                                        let arg_addr = opt_err(spec.message_hash.get(&parsed_def.arg))?;
                                        spec.compiled.extend_from_slice(&(*arg_addr as u16).to_be_bytes());                                        
                                    }

                                    if parsed_def.result.len() == 0 || parsed_def.result == "()" {
                                        spec.compiled.extend_from_slice(&0u16.to_be_bytes());
                                    } else {
                                        let result_addr = opt_err(spec.message_hash.get(&parsed_def.result))?;
                                        spec.compiled.extend_from_slice(&(*result_addr as u16).to_be_bytes());      
                                    }

                                    if parsed_def.kind == RPC_Fn_Kinds::result {
                                        if parsed_def.err.len() == 0 || parsed_def.err == "()" { 
                                            spec.compiled.extend_from_slice(&0u16.to_be_bytes());
                                        } else { 
                                            let err_addr = opt_err(spec.message_hash.get(&parsed_def.err))?;
                                            spec.compiled.extend_from_slice(&(*err_addr as u16).to_be_bytes());   
                                        }                                        
                                    }

                                    // provide struct data
                                    spec.specs.push(NP_RPC_Spec::RPC { 
                                        name: rpc_name.clone(),
                                        module_path: String::from(module),
                                        full_name,
                                        arg: if parsed_def.arg.len() == 0 { 
                                            None
                                        } else {
                                            Some(NP_RPC_Factory::find_msg(&parsed_def.arg, &spec)?)
                                        },
                                        result: if parsed_def.result.len() == 0 || parsed_def.result == "()" {
                                            None
                                        } else {
                                            Some(NP_RPC_Factory::find_msg(&parsed_def.result, &spec)?)
                                        },
                                        err: if parsed_def.err.len() == 0 || parsed_def.err == "()" { 
                                            None 
                                        } else { 
                                            Some(NP_RPC_Factory::find_msg(&parsed_def.err, &spec)?) 
                                        },
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
                                    let mut new_msg_mod = String::from(msg_module);
                                    new_msg_mod.push_str("::");
                                    new_msg_mod.push_str(mod_name);
                                    NP_RPC_Factory::parse_json_rpc(&new_mod, &new_msg_mod, &jspec, spec)?;
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

    /// Find a particular message in the spec vec
    /// 
    fn find_msg(msg_name: &String, spec: &NP_RPC_Specification) -> Result<usize, NP_Error> {
        if msg_name == "" { return Err(NP_Error::new("Missing message decleration in rpc method.")) }
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
    /// "() -> self::get"
    /// "() => ()"
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
*/
    /// Get a copy of the compiled byte array spec
    /// 
    pub fn compile_schema(&self) -> Vec<u8> {
        self.spec.compiled.clone()
    }

    /// Generate a new request object for a given rpc function
    /// 
    pub fn new_request(&self, rpc_name: &str) -> Result<NP_RPC_Request, NP_Error> {

        match self.method_hash.get(rpc_name) {
            Some(idx) => {
                match &self.spec.specs[*idx] {
                    NP_RPC_Spec::RPC { full_name, arg,   .. } => {
                        return Ok(NP_RPC_Request {
                            rpc_addr: *idx,
                            id: self.id,
                            spec: &self.spec,
                            rpc: full_name,
                            empty: self.empty.empty_buffer(None),
                            data: match *arg {
                                Some(arg) => {
                                    match &self.spec.specs[arg] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::new("unreachable"))
                                    }
                                },
                                None => self.empty.empty_buffer(None)
                            }
                        })
                    },
                    _ => Err(NP_Error::new("Cannot find request."))
                }
            },
            None => Err(NP_Error::new("Cannot find request."))
        }
    }

    /// Open a request.  The request spec and version must match the current spec and version of this factory.
    /// 
    pub fn open_request(&self, bytes: Vec<u8>) -> Result<NP_RPC_Request, NP_Error> {
        // first 19 bytes are id + version
        let id_bytes = &bytes[..19];
        if id_bytes != self.id {
            return Err(NP_Error::new("API ID or Version mismatch."))
        }

        // next 2 bytes is rpc address
        let rpc_addr = u16::from_be_bytes(unsafe { *(&bytes[19..21] as *const [u8] as *const [u8; 2]) }) as usize;

        // next 1 byte is request/response byte
        match RPC_Type::from(bytes[21]) {
            RPC_Type::Request => { },
            _ => return Err(NP_Error::new("Attempted to open non request buffer with request method."))
        };

        match &self.spec.specs[rpc_addr] {
            NP_RPC_Spec::RPC { full_name, arg,  .. } => {
                Ok(NP_RPC_Request {
                    rpc_addr,
                    id: self.id,
                    spec: &self.spec,
                    rpc: full_name,
                    empty: self.empty.empty_buffer(None),
                    data: match *arg {
                        Some(arg) => {
                            match &self.spec.specs[arg] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[22..].to_vec()),
                                _ => return Err(NP_Error::new("unreachable"))
                            }
                        },
                        None => self.empty.empty_buffer(None)
                    }
                })
            },
            _ => return Err(NP_Error::new("Can't find associated RPC Method."))
        }
    }

    /// Generate a new response object for a given rpc function
    /// 
    pub fn new_response(&self, rpc_name: &str) -> Result<NP_RPC_Response, NP_Error> {
        match self.method_hash.get(rpc_name) {
            Some(idx) => {
                match &self.spec.specs[*idx] {
                    NP_RPC_Spec::RPC { full_name, result, err,   .. } => {
                        return Ok(NP_RPC_Response {
                            rpc_addr: *idx,
                            rpc: full_name,
                            kind: NP_ResponseKinds::None,
                            id: self.id,
                            has_err: *err != Option::None,
                            data: match *result {
                                Some(result) => {
                                    match &self.spec.specs[result] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::new("unreachable"))
                                    }
                                },
                                None => self.empty.empty_buffer(None)
                            },
                            error: match *err {
                                Some(err) => {
                                    match &self.spec.specs[err] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::new("unreachable"))
                                    }
                                },
                                None => self.empty.empty_buffer(None)
                            }
                        })
                    },
                    _ => Err(NP_Error::new("Cannot find response!"))
                }
            },
            None => Err(NP_Error::new("Cannot find response!"))
        }

    }

    /// Open a response.  The response spec and version must match the current spec and version of this factory.
    /// 
    pub fn open_response(&self, bytes: Vec<u8>) -> Result<NP_RPC_Response, NP_Error> {
        // first 19 bytes are id + version
        let id_bytes = &bytes[..19];
        if id_bytes != self.id {
            return Err(NP_Error::new("API ID or Version mismatch."))
        }

        // next 2 bytes is rpc address
        let rpc_addr = u16::from_be_bytes(unsafe { *(&bytes[19..21] as *const [u8] as *const [u8; 2]) }) as usize;

        // next 1 byte is request/response byte
        match RPC_Type::from(bytes[21]) {
            RPC_Type::Response => { },
            _ => return Err(NP_Error::new("Attempted to open non response buffer with response method."))
        };

        match NP_ResponseKinds::from(bytes[22]) {
            NP_ResponseKinds::None => {
                match &self.spec.specs[rpc_addr] {
                    NP_RPC_Spec::RPC { full_name, err, .. } => {
                        Ok(NP_RPC_Response {
                            rpc_addr,
                            id: self.id,
                            kind: NP_ResponseKinds::None,
                            has_err: *err != Option::None,
                            rpc: full_name,
                            data: self.empty.empty_buffer(None),
                            error: self.empty.empty_buffer(None)
                        })
                    },
                    _ => return Err(NP_Error::new("Can't find associated RPC Method."))
                }
            },
            NP_ResponseKinds::Ok => {
                match &self.spec.specs[rpc_addr] {
                    NP_RPC_Spec::RPC { full_name, result, err, .. } => {
                        Ok(NP_RPC_Response {
                            rpc_addr,
                            id: self.id,
                            kind: NP_ResponseKinds::Ok,
                            has_err: *err != Option::None,
                            rpc: full_name,
                            data: match *result {
                                Some(result) => {
                                    match &self.spec.specs[result] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[23..].to_vec()),
                                        _ => return Err(NP_Error::new("unreachable"))
                                    }
                                },
                                None => self.empty.empty_buffer(None)
                            },
                            error: self.empty.empty_buffer(None)
                        })
                    },
                    _ => return Err(NP_Error::new("Can't find associated RPC Method."))
                }
            },
            NP_ResponseKinds::Error => {
                match &self.spec.specs[rpc_addr] {
                    NP_RPC_Spec::RPC { full_name, err, .. } => {
                        Ok(NP_RPC_Response {
                            rpc_addr,
                            id: self.id,
                            kind: NP_ResponseKinds::Error,
                            rpc: full_name,
                            has_err: *err != Option::None,
                            data: self.empty.empty_buffer(None),
                            error: match *err {
                                Some(err) => {
                                    match &self.spec.specs[err] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[23..].to_vec()),
                                        _ => return Err(NP_Error::new("unreachable"))
                                    }
                                },
                                None => return Err(NP_Error::new("Got error result on RPC method with no error type."))
                            }
                        })
                    },
                    _ => return Err(NP_Error::new("Can't find associated RPC Method."))
                }
            }
        }
    }
}

/// The different kinds of responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NP_ResponseKinds {
    /// Ok response is the default 
    Ok,
    /// Response contains an error
    Error,
    /// Response doesn't contain a value
    None
}

impl From<u8> for NP_ResponseKinds {
    fn from(value: u8) -> Self {
        if value > 2 { return NP_ResponseKinds::Ok; }
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug)]
#[repr(u8)]
#[allow(missing_docs)]
#[doc(hidden)]
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
    pub data: NP_Buffer<'request>,
    empty: NP_Buffer<'request>
}

impl<'request> NP_RPC_Request<'request> {
    /// Get empty response for this request
    pub fn new_response(&self) -> Result<NP_RPC_Response, NP_Error> {
        match &self.spec.specs[self.rpc_addr] {
            NP_RPC_Spec::RPC { full_name, result, err, .. } => {
                return Ok(NP_RPC_Response {
                    rpc_addr: self.rpc_addr,
                    kind: NP_ResponseKinds::None,
                    rpc: full_name,
                    has_err: *err != Option::None,
                    id: self.id,
                    data: match *result {
                        Some(result) => {
                            match &self.spec.specs[result] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::new("unreachable"))
                            }
                        },
                        None => self.empty.clone()
                    },
                    error: match *err {
                        Some(err) => {
                            match &self.spec.specs[err] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::new("unreachable"))
                            }
                        },
                        None => self.empty.clone()
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
    /// error message is set
    has_err: bool,
    /// what kind of response is this?
    pub kind: NP_ResponseKinds,
    /// the name of the rpc function
    pub rpc: &'response str,
    /// the data of this response
    pub data: NP_Buffer<'response>,
    /// if this is an error, the error data
    pub error: NP_Buffer<'response>
}



impl<'request> NP_RPC_Response<'request> {
    /// Close this response
    /// 
    /// The only failure condition is if you set the `NP_ResponseKinds` to `Error` but didn't have an error type declared in the rpc method.
    /// 
    pub fn rpc_close(self) -> Result<Vec<u8>, NP_Error> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 4);

        response_bytes.extend_from_slice(&self.id);
        response_bytes.extend_from_slice(&(self.rpc_addr as u16).to_be_bytes());
        response_bytes.push(RPC_Type::Response as u8);
        response_bytes.push(self.kind as u8);
        match &self.kind {
            NP_ResponseKinds::Ok => {
                response_bytes.extend(self.data.close());
            },
            NP_ResponseKinds::None => { },
            NP_ResponseKinds::Error => {
                if self.has_err {
                    response_bytes.extend(self.error.close());
                } else {
                    return Err(NP_Error::new("Attempted to close response as error type without error message defined in rpc method."))
                }
            }
        }

        Ok(response_bytes)
    }
}


#[test]
fn rpc_test() -> Result<(), NP_Error> {
    let rpc_factory = NP_RPC_Factory::new(r#"{
        "name": "test api",
        "description": "",
        "author": "Scott Lott",
        "id": "cc419a66-9bbe-48db-ad1c-e0ffa2a2376f",
        "version": "1.0.0",
        "spec": [
            {"msg": "Error", "type": "string" },
            {"msg": "Count", "type": "u32" },
            {"rpc": "get_count", "fn": "() -> self::Count"},
            {"mod": "user", "spec": [
                {"msg": "username", "type": "string"},
                {"msg": "user_id", "type": "u32"},
                {"rpc": "get_user_id", "fn": "(self::username) -> Option<self::user_id>"},
                {"rpc": "del_user", "fn": "(self::user_id) -> Result<self::user_id, mod::Error>"},
            ]}
        ]
    }"#)?;

    // === CLIENT ===
    // generate request
    let get_count: NP_RPC_Request = rpc_factory.new_request("get_count")?;
    // close request
    let count_req_bytes: Vec<u8> = get_count.rpc_close();

    // === SEND count_req_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(count_req_bytes)?;
    assert_eq!(a_request.rpc, "get_count");
    // generate a response
    let mut count_response: NP_RPC_Response = a_request.new_response()?;
    // set response data
    count_response.data.set(&[], 20u32)?;
    // set response kind
    count_response.kind = NP_ResponseKinds::Ok;
    // close response
    let respond_bytes = count_response.rpc_close()?;

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let count_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(count_response.rpc, "get_count");
    // confirm that we got data in the response
    assert_eq!(count_response.kind, NP_ResponseKinds::Ok);
    // confirm it's the same data the server sent
    assert_eq!(count_response.data.get(&[])?, Some(20u32));


    // Now do a result request with error

    // === CLIENT ===
    // generate request
    let mut del_user: NP_RPC_Request = rpc_factory.new_request("user.del_user")?;
    del_user.data.set(&[], 50u32)?;
    let del_user_bytes: Vec<u8> = del_user.rpc_close();

    // === SEND del_user_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(del_user_bytes)?;
    assert_eq!(a_request.rpc, "user.del_user");
    // generate a response
    let mut del_response: NP_RPC_Response = a_request.new_response()?;
    // set response as error
    del_response.error.set(&[], "Can't find user.")?;
    del_response.kind = NP_ResponseKinds::Error;
    // close response
    let respond_bytes = del_response.rpc_close()?;

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let del_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(del_response.rpc, "user.del_user");
    // confirm we recieved error response
    assert_eq!(del_response.kind, NP_ResponseKinds::Error);
    // get the error information
    assert_eq!(del_response.error.get(&[])?, Some("Can't find user."));

    // Now do a result request with an ok return

    // === CLIENT ===
    // generate request
    let mut del_user: NP_RPC_Request = rpc_factory.new_request("user.del_user")?;
    del_user.data.set(&[], 50u32)?;
    let del_user_bytes: Vec<u8> = del_user.rpc_close();

    // === SEND del_user_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(del_user_bytes)?;
    assert_eq!(a_request.rpc, "user.del_user");
    // generate a response
    let mut del_response: NP_RPC_Response = a_request.new_response()?;
    // set response as error
    del_response.data.set(&[], 50u32)?;
    del_response.kind = NP_ResponseKinds::Ok;
    // close response
    let respond_bytes = del_response.rpc_close()?;

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let del_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(del_response.rpc, "user.del_user");
    // confirm that we got data in the response
    assert_eq!(del_response.kind, NP_ResponseKinds::Ok);
    // confirm it's the same data set on the server
    assert_eq!(del_response.data.get(&[])?, Some(50u32));

    // Now do an option request with an ok return

    // === CLIENT ===
    // generate request
    let mut get_user: NP_RPC_Request = rpc_factory.new_request("user.get_user_id")?;
    get_user.data.set(&[], "username")?;
    let get_user_bytes: Vec<u8> = get_user.rpc_close();

    // === SEND get_user_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(get_user_bytes)?;
    assert_eq!(a_request.rpc, "user.get_user_id");
    // generate a response
    let mut del_response: NP_RPC_Response = a_request.new_response()?;
    // set response as ok with data
    del_response.data.set(&[], 50u32)?;
    del_response.kind = NP_ResponseKinds::Ok;
    // close response
    let respond_bytes = del_response.rpc_close()?;

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let del_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(del_response.rpc, "user.get_user_id");
    // confirm that we got data in the response
    assert_eq!(del_response.kind, NP_ResponseKinds::Ok);
    // confirm it's the same data set on the server
    assert_eq!(del_response.data.get(&[])?, Some(50u32));

    // Now do an option request with a none return

    // === CLIENT ===
    // generate request
    let mut get_user: NP_RPC_Request = rpc_factory.new_request("user.get_user_id")?;
    get_user.data.set(&[], "username")?;
    let get_user_bytes: Vec<u8> = get_user.rpc_close();

    // === SEND get_user_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(get_user_bytes)?;
    assert_eq!(a_request.rpc, "user.get_user_id");
    // generate a response
    let mut del_response: NP_RPC_Response = a_request.new_response()?;
    // set response as none
    del_response.kind = NP_ResponseKinds::None;
    // close response
    let respond_bytes = del_response.rpc_close()?;

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let del_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(del_response.rpc, "user.get_user_id");
    // confirm that we got data in the response
    assert_eq!(del_response.kind, NP_ResponseKinds::None);
    // with NONE response there is no data

    Ok(())
}