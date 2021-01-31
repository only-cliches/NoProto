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
//! {"msg": "address", "type": "struct", "fields": [
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
//! Methods can always access messages in their own namespace using `self`.  Otherwise, the top of the name space is `mod` and messages in other namespaces can be used by their names.  For example, let's say we had a message named `delete` inside the `modify` RPC module inside the `user` RPC module.  That message could be accessed by any RPC method with `mod::user::modify::delete`.
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
//! assert_eq!(a_request.rpc_name(), "get_count");
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
//! assert_eq!(count_response.rpc_name(), "get_count");
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
//! assert_eq!(a_request.rpc_name(), "user.del_user");
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
//! assert_eq!(del_response.rpc_name(), "user.del_user");
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
//! assert_eq!(a_request.rpc_name(), "user.del_user");
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
//! assert_eq!(del_response.rpc_name(), "user.del_user");
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
//! assert_eq!(a_request.rpc_name(), "user.get_user_id");
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
//! assert_eq!(del_response.rpc_name(), "user.get_user_id");
//! // confirm that we got data in the response
//! assert_eq!(del_response.kind, NP_ResponseKinds::None);
//! // with NONE response there is no data
//! 
//! # Ok::<(), NP_Error>(()) 
//! ```
//! 
//! 
//! 

use alloc::str::from_utf8_unchecked;
use crate::{NP_Schema_Bytes, hashmap::{SEED, murmurhash3_x86_32}, memory::NP_Memory_Owned};

use crate::{hashmap::NP_HashMap, pointer::uuid::NP_UUID, utils::opt_err};
use crate::NP_Factory;
use crate::NP_Schema;
use alloc::prelude::v1::Box;
use crate::json_decode;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::string::ToString;
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

impl From<u8> for RPC_Fn_Kinds {
    fn from(value: u8) -> Self {
        if value > 2 { return RPC_Fn_Kinds::normal; }
        unsafe { core::mem::transmute(value) }
    }
}

#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
struct NP_Str_Addr {
    idx: usize,
    len: usize
}

/// RPC Specifications
/// 
#[derive(Debug)]
#[doc(hidden)]
enum NP_RPC_Spec<'spec> {
    /// RPC Function
    RPC { 
        /// Full path (module_path::name)
        full_name: NP_Str_Addr,
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
        /// Factory for this message
        factory: NP_Factory<'spec>
    }
}

/// RPC Factory
#[derive(Debug)]
pub struct NP_RPC_Factory<'fact> {
    /// Name of API
    name: NP_Str_Addr,
    /// API Author
    author: NP_Str_Addr,
    /// Specification for this factory
    spec: NP_RPC_Specification<'fact>,
    method_hash: NP_HashMap,
    /// blank buffer
    empty: NP_Factory<'fact>
}

#[derive(Debug)]
#[doc(hidden)]
enum NP_RCP_Spec<'spec> {
    Owned(Vec<u8>),
    Borrwed(&'spec [u8])
}

impl<'spec> NP_RCP_Spec<'spec> {
    #[inline(always)]
    pub fn write(&mut self) -> Result<&mut Vec<u8>, NP_Error> {
        match self {
            NP_RCP_Spec::Owned(vec) => Ok(vec),
            _ => Err(NP_Error::Unreachable)
        }
    }
    #[inline(always)]
    pub fn read(&self) -> &[u8] {
        match self {
            NP_RCP_Spec::Owned(vec) => vec,
            NP_RCP_Spec::Borrwed(vec) => *vec
        }
    }
}

/// RPC Specification
#[derive(Debug)]
#[doc(hidden)]
pub struct NP_RPC_Specification<'spec> {
    /// Specification for this factory
    specs: Vec<NP_RPC_Spec<'spec>>,
    bytes: NP_RCP_Spec<'spec>,
    /// Message HashMap
    spec_msg_hash: NP_HashMap,
    id_hash: [u8; 4]
}

impl<'spec> NP_RPC_Specification<'spec> {
    fn read_str(&self, addr: &NP_Str_Addr) -> &str {
        let end = addr.idx + addr.len;
        if end > self.bytes.read().len() {
            ""
        } else {
            unsafe { from_utf8_unchecked(&self.bytes.read()[addr.idx..end]) }
        }
    }
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

#[inline(always)]
fn read_u16(bytes: &[u8], offset: usize) -> usize {
    u16::from_be_bytes(unsafe { *(&bytes[offset..(offset + 2)] as *const [u8] as *const [u8; 2])}) as usize
}

impl<'fact> NP_RPC_Factory<'fact> {

    /// Parse a JSON RPC spec into an RPC Factory
    /// 
    pub fn new(json_rcp_spec: &str) -> Result<Self, NP_Error> {

        let parsed = json_decode(String::from(json_rcp_spec))?;

        let version = String::from(match &parsed["version"] { NP_JSON::String(version) => { version }, _ => { "" } }).split(".").map(|s| s.parse::<u8>().unwrap_or(0)).collect::<Vec<u8>>();
        let author_str = match &parsed["author"] { NP_JSON::String(author) => { author }, _ => { "" } };
        let id_str = String::from(match &parsed["id"] { NP_JSON::String(id) => { id }, _ => { "" } }).replace("-", "");
        let name_str = match &parsed["name"] { NP_JSON::String(name) => { name }, _ => { "" } };

        if name_str.len() > core::u16::MAX as usize {
            return Err(NP_Error::new("API name cannot be longer than 2^16 UTF8 bytes"));
        }

        if author_str.len() > core::u16::MAX as usize {
            return Err(NP_Error::new("Author cannot be longer than 2^16 UTF8 bytes"));
        }

        if version.len() != 3 {
            return Err(NP_Error::new("There must be 3 version points. X.X.X"));
        }

        if id_str.len() != 32 {
            return Err(NP_Error::new("id property must be a V4 UUID."));
        }

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

        let mut id_hash = [0u8; 4];
        for (x, b) in murmurhash3_x86_32(&id_bytes, SEED).to_be_bytes().iter().enumerate() {
            id_hash[x] = *b;
        }

        let mut compiled = Vec::with_capacity(1024);

        // first 2 bytes contains the offset of the first rpc method (uknown right now)
        compiled.extend_from_slice(&0u16.to_be_bytes());

        // next 19 bytes are version
        compiled.extend_from_slice(&id_bytes);
        
        // next bytes are name
        compiled.extend_from_slice(&(name_str.len() as u16).to_be_bytes());
        let name_addr = NP_Str_Addr { idx: compiled.len(), len: name_str.len() };
        compiled.extend_from_slice(&name_str.as_bytes());

        // next bytes are author
        compiled.extend_from_slice(&(author_str.len() as u16).to_be_bytes());
        let author_addr = NP_Str_Addr { idx: compiled.len(), len: author_str.len() };
        compiled.extend_from_slice(&author_str.as_bytes());

        let mut spec = NP_RPC_Specification { id_hash, specs: Vec::with_capacity(1024), bytes: NP_RCP_Spec::Owned(compiled), spec_msg_hash: NP_HashMap::new() };

        // now the messages
        NP_RPC_Factory::parse_json_msg("mod", &parsed, &mut spec)?;
        if spec.bytes.read().len() > core::u16::MAX as usize {
            return Err(NP_Error::new("Too many messages in spec, can't compile."))
        }

        // set first 2 bytes to correct offset after we've inserted all messages
        for (x, b) in (spec.bytes.read().len() as u16).to_be_bytes().iter().enumerate() {
            spec.bytes.write()?[x] = *b;
        }

        // and finally the methods
        NP_RPC_Factory::parse_json_rpc("", "mod", &parsed, &mut spec)?;

        let mut method_hash: NP_HashMap = NP_HashMap::new();

        for (idx, one_spec) in spec.specs.iter().enumerate() {
            match one_spec {
                NP_RPC_Spec::RPC { full_name, .. } => {
                    method_hash.insert(spec.read_str(full_name), idx)?;
                },
                _ => {}
            }
        }

        Ok(Self {
            name: name_addr,
            author: author_addr,
            method_hash,
            spec: spec,
            empty: NP_Factory::new_bytes(&[0u8])?
        })
    }

    /// Get API name
    pub fn get_name(&self) -> &str {
        self.spec.read_str(&self.name)
    }

    /// Get API author
    pub fn get_author(&self) -> &str {
        self.spec.read_str(&self.author)
    }

    /// Get API ID
    pub fn get_id(&self) -> String {
        let mut uuid_value = [0u8; 16];
        for x in 0..16usize {
            uuid_value[x] = self.spec.bytes.read()[x + 2];
        }

        NP_UUID { value: uuid_value }.to_string()
    }

    /// Get API Version
    pub fn get_version(&self) -> String {

        let mut version: String = String::from("");
        for x in 0..3usize {
            version.push_str(self.spec.bytes.read()[18 + x].to_string().as_str());
            if x != 2 {
                version.push_str(".");
            }
        }

        version
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
                                schema_bytes: NP_Schema_Bytes::Owned(schema.1)
                            };
                            let full_name = format!("{}::{}", module, msg_name);

                            // insert this message address
                            // spec.compiled_msg_hash.insert(&full_name, spec.compiled.len())?;

                            let bytes_w = spec.bytes.write()?;

                            let schema = factory.export_schema_bytes();
                            bytes_w.extend_from_slice(&(schema.len() as u16).to_be_bytes());
                            bytes_w.extend(schema);

                            spec.spec_msg_hash.insert(&full_name, spec.specs.len())?;
                            spec.specs.push(NP_RPC_Spec::MSG { 
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

                                    let bytes_w = spec.bytes.write()?;

                                    // compile the RPC spec
                                    bytes_w.extend_from_slice(&(full_name.len() as u16).to_be_bytes());
                                    let f_addr = NP_Str_Addr { idx: bytes_w.len(), len: full_name.len() };
                                    bytes_w.extend_from_slice(&full_name.as_bytes());
                                    bytes_w.push(parsed_def.kind as u8);

                                    if parsed_def.arg.len() == 0 { 
                                        bytes_w.extend_from_slice(&0u16.to_be_bytes());
                                    } else {
                                        let arg_addr = opt_err(spec.spec_msg_hash.get(&parsed_def.arg))?;
                                        bytes_w.extend_from_slice(&(*arg_addr as u16 + 1).to_be_bytes());                                        
                                    }

                                    if parsed_def.result.len() == 0 || parsed_def.result == "()" {
                                        bytes_w.extend_from_slice(&0u16.to_be_bytes());
                                    } else {
                                        let result_addr = opt_err(spec.spec_msg_hash.get(&parsed_def.result))?;
                                        bytes_w.extend_from_slice(&(*result_addr as u16 + 1).to_be_bytes());      
                                    }

                                    if parsed_def.kind == RPC_Fn_Kinds::result {
                                        if parsed_def.err.len() == 0 || parsed_def.err == "()" { 
                                            bytes_w.extend_from_slice(&0u16.to_be_bytes());
                                        } else { 
                                            let err_addr = opt_err(spec.spec_msg_hash.get(&parsed_def.err))?;
                                            bytes_w.extend_from_slice(&(*err_addr as u16 + 1).to_be_bytes());   
                                        }                                        
                                    }

                                    // provide struct data
                                    let rpc = NP_RPC_Spec::RPC { 
                                        // name: if module == "" { f_addr } else { NP_Str_Addr { idx: f_addr.idx + module.len() + 1, len: rpc_name.len() } },
                                        // module_path: NP_Str_Addr { idx: f_addr.idx, len: module.len() },
                                        full_name: f_addr,
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
                                    };
                                    spec.specs.push(rpc);
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
 
        match spec.spec_msg_hash.get(msg_name) {
            Some(idx) => {
                Ok(*idx)
            },
            None => {
                let mut name = msg_name.clone();
                name.push_str("Can't find rpc message '");
                name.push_str(msg_name);
                name.push_str("'.");
                Err(NP_Error::new(name.as_str()))
            }
        }
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

    /// Parse a byte rpc spec into an RPC Factory.
    /// 
    /// This method is orders of magnitude faster than the `new` method since there's no JSON to parse and only a few memory allocations.
    /// 
    pub fn new_bytes(bytes_rpc_spec: &'fact [u8]) -> Result<Self, NP_Error>  {

        let mut id_hash = [0u8; 4];
        for (x, b) in murmurhash3_x86_32(&bytes_rpc_spec[2..21], SEED).to_be_bytes().iter().enumerate() {
            id_hash[x] = *b;
        }

        let mut offset: usize = 21;
        let name_len = read_u16(bytes_rpc_spec, offset);
        let name_addr = NP_Str_Addr { idx: offset + 2, len: name_len };

        offset += 2 + name_len;

        let author_len = read_u16(bytes_rpc_spec, offset);
        let author_addr = NP_Str_Addr { idx: offset + 2, len: author_len };

        offset += 2 + author_len;

        // now at begnning of messages
        let end_of_messages = read_u16(bytes_rpc_spec, 0);

        let mut spec = NP_RPC_Specification { id_hash, specs: Vec::with_capacity(1024), bytes: NP_RCP_Spec::Borrwed(bytes_rpc_spec), spec_msg_hash: NP_HashMap::empty() };

        let read_bytes = spec.bytes.read();

        while offset < end_of_messages {
            let schema_len = read_u16(bytes_rpc_spec, offset);
            offset += 2;
            // we're bypassing rust's lifetime system here and creating a self referential struct
            // it's safe because everything is immutable plus spec.specs and spec.bytes have the same lifetime
            spec.specs.push(NP_RPC_Spec::MSG { 
                factory: unsafe { NP_Factory::new_bytes_ptr(&spec.bytes.read()[offset..(offset + schema_len)] as *const [u8])? }
            });
            offset += schema_len;
        }
        
        // messages are now parsed, time for RPC methods
        offset = end_of_messages;

        let mut method_hash: NP_HashMap = NP_HashMap::new();

        while offset < read_bytes.len() {
            let name_len = read_u16(bytes_rpc_spec, offset);
            offset += 2;
            let full_name = NP_Str_Addr { idx: offset, len: name_len };
            offset += name_len;
            
            let fn_kind = RPC_Fn_Kinds::from(read_bytes[offset]);
            offset += 1;

            let arg_addr = read_u16(bytes_rpc_spec, offset);
            offset += 2;
            let result_addr = read_u16(bytes_rpc_spec, offset);
            offset += 2;    

            let err_addr = if fn_kind == RPC_Fn_Kinds::result {
                let addr = read_u16(bytes_rpc_spec, offset);
                offset += 2;  
                addr
            } else {
                0
            };

            method_hash.insert(spec.read_str(&full_name), spec.specs.len())?;

            spec.specs.push(NP_RPC_Spec::RPC { 
                full_name: full_name,
                arg: if arg_addr == 0 { None } else { Some(arg_addr - 1) },
                result: if result_addr == 0 { None } else { Some(result_addr - 1) },
                err: if err_addr == 0 { None } else { Some(err_addr - 1) },
                kind: fn_kind
            });
        }
        
        // methods are now parsed
        Ok(Self {
            name: name_addr,
            author: author_addr,
            method_hash,
            spec: spec,
            empty: NP_Factory::new_bytes(&[0u8])?
        })
    }

    /// Get a copy of the compiled byte array specification
    /// 
    /// The compiled byte array is *much* faster to parse and takes up *much* less space.
    /// 
    /// If you don't need the verbosity of the JSON spec, use this instead.
    /// 
    pub fn compile_spec(&self) -> &[u8] {
        self.spec.bytes.read()
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
                            spec: &self.spec,
                            rpc: *full_name,
                            empty: self.empty.empty_buffer(None),
                            data: match *arg {
                                Some(arg) => {
                                    match &self.spec.specs[arg] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::Unreachable)
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
        let id_bytes = &bytes[..4];
        if id_bytes != self.spec.id_hash {
            return Err(NP_Error::new("API ID or Version mismatch."))
        }

        // next 2 bytes is rpc address
        let rpc_addr = read_u16(&bytes, 4);

        // next 1 byte is request/response byte
        match RPC_Type::from(bytes[6]) {
            RPC_Type::Request => { },
            _ => return Err(NP_Error::new("Attempted to open non request buffer with request method."))
        };

        match &self.spec.specs[rpc_addr] {
            NP_RPC_Spec::RPC { full_name, arg,  .. } => {
                Ok(NP_RPC_Request {
                    rpc_addr,
                    spec: &self.spec,
                    rpc: *full_name,
                    empty: self.empty.empty_buffer(None),
                    data: match *arg {
                        Some(arg) => {
                            match &self.spec.specs[arg] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[7..].to_vec()),
                                _ => return Err(NP_Error::Unreachable)
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
                            rpc: *full_name,
                            spec: &self.spec,
                            kind: NP_ResponseKinds::None,
                            has_err: *err != Option::None,
                            data: match *result {
                                Some(result) => {
                                    match &self.spec.specs[result] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::Unreachable)
                                    }
                                },
                                None => self.empty.empty_buffer(None)
                            },
                            error: match *err {
                                Some(err) => {
                                    match &self.spec.specs[err] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                        _ => return Err(NP_Error::Unreachable)
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
        // first 4 bytes are id hash (version + uuid)
        let id_bytes = &bytes[..4];
        if id_bytes != self.spec.id_hash {
            return Err(NP_Error::new("API ID or Version mismatch."))
        }

        // next 2 bytes is rpc address
        let rpc_addr = read_u16(&bytes, 4);

        // next 1 byte is request/response byte
        match RPC_Type::from(bytes[6]) {
            RPC_Type::Response => { },
            _ => return Err(NP_Error::new("Attempted to open non response buffer with response method."))
        };

        match NP_ResponseKinds::from(bytes[7]) {
            NP_ResponseKinds::None => {
                match &self.spec.specs[rpc_addr] {
                    NP_RPC_Spec::RPC { full_name, err, .. } => {
                        Ok(NP_RPC_Response {
                            rpc_addr,
                            kind: NP_ResponseKinds::None,
                            has_err: *err != Option::None,
                            spec: &self.spec,
                            rpc: *full_name,
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
                            kind: NP_ResponseKinds::Ok,
                            has_err: *err != Option::None,
                            rpc: *full_name,
                            spec: &self.spec,
                            data: match *result {
                                Some(result) => {
                                    match &self.spec.specs[result] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[8..].to_vec()),
                                        _ => return Err(NP_Error::Unreachable)
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
                            kind: NP_ResponseKinds::Error,
                            rpc: *full_name,
                            spec: &self.spec,
                            has_err: *err != Option::None,
                            data: self.empty.empty_buffer(None),
                            error: match *err {
                                Some(err) => {
                                    match &self.spec.specs[err] {
                                        NP_RPC_Spec::MSG { factory, .. } => factory.open_buffer(bytes[8..].to_vec()),
                                        _ => return Err(NP_Error::Unreachable)
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
    spec: &'request NP_RPC_Specification<'request>,
    /// the name of the rpc function
    rpc: NP_Str_Addr,
    /// the request data
    pub data: NP_Buffer<NP_Memory_Owned>,
    empty: NP_Buffer<NP_Memory_Owned>
}

impl<'request> NP_RPC_Request<'request> {

    /// Get the API id of the RPC schema this came from.
    pub fn api_id(&self) -> &str {
        todo!()
    }

    /// Get the API version of the RPC schema this came from.
    pub fn api_version(&self) -> &str {
        todo!()
    }

    /// Get the name of this RPC method
    pub fn rpc_name(&self) -> &str {
        self.spec.read_str(&self.rpc)
    }

    /// Get empty response for this request
    pub fn new_response(&self) -> Result<NP_RPC_Response, NP_Error> {
        match &self.spec.specs[self.rpc_addr] {
            NP_RPC_Spec::RPC { full_name, result, err, .. } => {
                return Ok(NP_RPC_Response {
                    rpc_addr: self.rpc_addr,
                    kind: NP_ResponseKinds::None,
                    rpc: *full_name,
                    spec: &self.spec,
                    has_err: *err != Option::None,
                    data: match *result {
                        Some(result) => {
                            match &self.spec.specs[result] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::Unreachable)
                            }
                        },
                        None => self.empty.clone()
                    },
                    error: match *err {
                        Some(err) => {
                            match &self.spec.specs[err] {
                                NP_RPC_Spec::MSG { factory, .. } => factory.empty_buffer(None),
                                _ => return Err(NP_Error::Unreachable)
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

        response_bytes.extend_from_slice(&self.spec.id_hash);
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
    /// error message is set
    has_err: bool,
    /// what kind of response is this?
    pub kind: NP_ResponseKinds,
    /// the name of the rpc function
    rpc: NP_Str_Addr,
    spec: &'response NP_RPC_Specification<'response> ,
    /// the data of this response
    pub data: NP_Buffer<NP_Memory_Owned>,
    /// if this is an error, the error data
    pub error: NP_Buffer<NP_Memory_Owned>
}



impl<'request> NP_RPC_Response<'request> {

    
    /// Get the API id of the RPC schema this came from.
    pub fn api_id(&self) -> &str {
        todo!()
    }

    /// Get the API version of the RPC schema this came from.
    pub fn api_version(&self) -> &str {
        todo!()
    }

    /// Get the name of this RPC method
    pub fn rpc_name(&self) -> &str {
        self.spec.read_str(&self.rpc)
    }

    /// Close this response
    /// 
    /// The only failure condition is if you set the `kind` to `NP_ResponseKinds::Error` but didn't have an error type declared in the rpc method.
    /// 
    pub fn rpc_close(self) -> Result<Vec<u8>, NP_Error> {
        let mut response_bytes: Vec<u8> = Vec::with_capacity(self.data.read_bytes().len() + 19 + 4);

        response_bytes.extend_from_slice(&self.spec.id_hash);
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
        "author": "Jeb Kermin",
        "id": "CC419A66-9BBE-48DB-AD1C-E0FFA2A2376F",
        "version": "1.2.3",
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

    // checks that compiled byte specs work
    assert_eq!(rpc_factory.compile_spec().len(), 128); // JSON schema above is 467 bytes without whitespace
    let rpc_factory = NP_RPC_Factory::new_bytes(&rpc_factory.compile_spec())?;

    assert_eq!(rpc_factory.get_name(), "test api");
    assert_eq!(rpc_factory.get_author(), "Jeb Kermin");
    assert_eq!(rpc_factory.get_id(), "CC419A66-9BBE-48DB-AD1C-E0FFA2A2376F");
    assert_eq!(rpc_factory.get_version(), "1.2.3");

    // === CLIENT ===
    // generate request
    let get_count: NP_RPC_Request = rpc_factory.new_request("get_count")?;
    // close request
    let count_req_bytes: Vec<u8> = get_count.rpc_close();
    assert_eq!(count_req_bytes.len(), 11);

    // === SEND count_req_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(count_req_bytes)?;
    assert_eq!(a_request.rpc_name(), "get_count");
    // generate a response
    let mut count_response: NP_RPC_Response = a_request.new_response()?;
    // set response data
    count_response.data.set(&[] as &[&str], 20u32)?;
    // set response kind
    count_response.kind = NP_ResponseKinds::Ok;
    // close response
    let respond_bytes = count_response.rpc_close()?;
    assert_eq!(respond_bytes.len(), 16);

    // === SEND respond_bytes to CLIENT ====

    // === CLIENT ===
    let count_response = rpc_factory.open_response(respond_bytes)?;
    // confirm our response matches the same request RPC we sent
    assert_eq!(count_response.rpc_name(), "get_count");
    // confirm that we got data in the response
    assert_eq!(count_response.kind, NP_ResponseKinds::Ok);
    // confirm it's the same data the server sent
    assert_eq!(count_response.data.get(&[])?, Some(20u32));


    // Now do a result request with error

    // === CLIENT ===
    // generate request
    let mut del_user: NP_RPC_Request = rpc_factory.new_request("user.del_user")?;
    del_user.data.set(&[] as &[&str], 50u32)?;
    let del_user_bytes: Vec<u8> = del_user.rpc_close();

    // === SEND del_user_bytes to SERVER ===

    // === SERVER ===
    // ingest request
    let a_request: NP_RPC_Request = rpc_factory.open_request(del_user_bytes)?;
    assert_eq!(a_request.rpc_name(), "user.del_user");
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
    assert_eq!(del_response.rpc_name(), "user.del_user");
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
    assert_eq!(a_request.rpc_name(), "user.del_user");
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
    assert_eq!(del_response.rpc_name(), "user.del_user");
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
    assert_eq!(a_request.rpc_name(), "user.get_user_id");
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
    assert_eq!(del_response.rpc_name(), "user.get_user_id");
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
    assert_eq!(a_request.rpc_name(), "user.get_user_id");
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
    assert_eq!(del_response.rpc_name(), "user.get_user_id");
    // confirm that we got data in the response
    assert_eq!(del_response.kind, NP_ResponseKinds::None);
    // with NONE response there is no data

    Ok(())
}