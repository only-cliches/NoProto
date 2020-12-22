extern crate alloc;
extern crate wasm_bindgen;
extern crate wee_alloc;

use wasm_bindgen::prelude::*;
use alloc::string::String;
use alloc::vec::Vec;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct State {
    factories: Vec<NP_Factory<'static>>
}

impl State {
    pub fn facts(&mut self) -> &mut Vec<NP_Factory<'static>> {
        &mut self.factories
    }
}

#[wasm_bindgen]
pub fn new_factory(json: String) -> u32 {
    let mut factories = Factories.lock().unwrap();
    match NP_Factory::new(json) {
        Ok(fact) => {
            let len = factories.len();
            factories.push(fact);
            len as u32
        },
        _ => 0
    }
}