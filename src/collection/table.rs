use crate::NoProtoMemory;
use crate::pointer::NoProtoPointer;
use json::JsonValue;
use std::rc::Rc;
use std::cell::RefCell;

pub struct NoProtoColumn {
    name: String,
    index: u8,
    type_string: String
}

pub struct NoProtoTable {
    address: u32, // pointer location
    head: u32,
    memory: Rc<RefCell<NoProtoMemory>>,
    model: Rc<RefCell<JsonValue>>,
    columns: Vec<NoProtoColumn>
}

impl NoProtoTable {

    pub fn new(address: u32, memory: Rc<RefCell<NoProtoMemory>>, model: Rc<RefCell<JsonValue>>) -> Self {

        let addr = address as usize;
        let mut head: [u8; 4] = [0; 4];

        let mut columns = Vec::new();

        {
            let b_bytes = &memory.borrow().bytes;
            head.copy_from_slice(&b_bytes[addr..(addr+4)]);

            let json_data = model.borrow();
            
        }

        



        NoProtoTable {
            head: u32::from_le_bytes(head),
            address: address,
            memory: memory,
            model: model,
            columns: columns
        }
    }

    

    /*pub fn select(&self, column: &str) -> NoProtoPointer {

    }*/

    pub fn set_head(&mut self, addr: u32) {
        let mut memory = self.memory.borrow_mut();

        let addr_bytes = addr.to_le_bytes();

        for x in 0..addr_bytes.len() {
            memory.bytes[(self.address + x as u32) as usize] = addr_bytes[x as usize];
        }
    }

    pub fn delete(&mut self, column: &str) -> bool {
        false
    }

    pub fn clear(&mut self) {
        self.set_head(0);
    }

    pub fn has(&self, column: &str) {

    }

}