//! Remote Procedure Call APIs
//! 

use alloc::vec::Vec;
use crate::{NP_JSON, buffer::NP_Buffer, error::NP_Error};

pub struct NP_RPC_Factory {

}

impl NP_RPC_Factory {

    pub fn new(json_rcp_schema: &str) -> Result<Self, NP_Error> {
        todo!()
    }

    pub fn new_compiled(bytes_rpc_schema: Vec<u8>) -> Result<Self, NP_Error>  {
        todo!()
    }

    pub fn compile_schema(&self) -> Vec<u8> {
        todo!()
    }

    pub fn export_schema(&self) -> Result<NP_JSON, NP_Error> {
        todo!()
    }

    pub fn new_request(&self) -> NP_RCP_Request {
        todo!()
    }

    pub fn open_request(&self, bytes: Vec<u8>) -> NP_RCP_Request {
        todo!()
    }

    pub fn new_reponse(&self) -> NP_RCP_Response {
        todo!()
    }

    pub fn open_response(&self, bytes: Vec<u8>) -> NP_RCP_Response {
        todo!()
    }
}

pub struct NP_RCP_Request<'response> {
    pub name: &'response str,
    pub data: NP_Buffer<'response>
}

impl<'response> NP_RCP_Request<'response> {
    pub fn close() {
        todo!()
    }
}

pub struct NP_RCP_Response<'response> {
    pub name: &'response str,
    pub data: NP_Buffer<'response>
}

impl<'response> NP_RCP_Response<'response> {
    pub fn close() {
        todo!()
    }
}

