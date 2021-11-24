use crate::val::Val;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Default)]
pub struct Env {
    bindings: HashMap<String, Val>,
}

impl Env {
    pub fn new() -> Self {
        Env { ..Default::default() }
    }
    
    pub fn store_binding(&mut self, name: String, val: Val) {
        self.bindings.insert(name, val);
    }

    pub fn get_binding(&self, name: &str) -> Result<Val, String> {
        self.bindings
            .get(name)
            .cloned()
            .ok_or_else(|| format!("binding with name ‘{}’ does not exist", name))
    }
}