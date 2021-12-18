use crate::{error::RuntimeError, system::System};

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    fn prompt() -> String;
}

pub struct Web;

impl System for Web {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(std::iter::empty())
    }
    fn print(&self) -> Box<dyn FnMut(String) -> Result<(), RuntimeError>> {
        Box::new(|s| Ok(log(&s)))
    }
    fn read(&self) -> Box<dyn FnMut() -> Result<String, RuntimeError>> {
        Box::new(|| Ok(prompt()))
    }
}
