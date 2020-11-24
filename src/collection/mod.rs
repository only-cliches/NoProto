//! Collections: NP_Table, NP_Tuple, NP_List & NP_Map

use crate::{error::NP_Error, memory::NP_Memory, pointer::{NP_Cursor_Addr}};

/// Table data type
pub mod table;
/// Map data type
pub mod map;
/// List data type
pub mod list;
/// Tuple data type
pub mod tuple;

#[doc(hidden)]
pub trait NP_Collection<'collection> {
    /// Step a pointer to the next item in the collection
    fn step_pointer(cursor_addr: &NP_Cursor_Addr, memory: &'collection NP_Memory<'collection>) -> Option<NP_Cursor_Addr>;
    /// Commit a virtual pointer into the buffer
    fn commit_pointer(cursor_addr: &NP_Cursor_Addr, memory: &'collection NP_Memory<'collection>) -> Result<NP_Cursor_Addr, NP_Error>;
    /// Generate this collection as an iterator
    fn start_iter(list_cursor_addr: &NP_Cursor_Addr, memory: &'collection NP_Memory<'collection>) -> Result<Self, NP_Error>;
}