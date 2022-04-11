use crate::error::NP_Error;
use crate::memory::NP_Memory;
use crate::json_flex::NP_JSON;


pub trait NP_Value: Sized {

    fn write_value(self, address: usize, memory: &NP_Memory) -> Result<(), NP_Error>;
    fn read_value(address: usize, memory: &NP_Memory) -> Result<Self, NP_Error>;

    fn write_json(json: &NP_JSON, address: usize, memory: &NP_Memory) -> Result<(), NP_Error>;
    fn read_json(address: usize, memory: &NP_Memory) -> Result<NP_JSON, NP_Error>;

    fn read_bytes(address: usize, memory: &NP_Memory) -> Result<&[u8], NP_Error>;
}