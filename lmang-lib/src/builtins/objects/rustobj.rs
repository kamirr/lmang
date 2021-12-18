use crate::builtins::rustfn::RustFn;
use crate::error::RuntimeError;
use crate::val::{Object, Val};
use std::fmt;

#[derive(Clone, Debug)]
pub struct RustObj {
    name: String,
    funcs: Vec<RustFn>,
}

impl RustObj {
    pub fn boxed(name: impl Into<String>, funcs: Vec<RustFn>) -> Box<Self> {
        let name = name.into();
        Box::new(RustObj { name, funcs })
    }
}

impl Object for RustObj {
    fn member_names(&self) -> Vec<String> {
        self.funcs.iter().map(|f| f.name.clone()).collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        for func in self.funcs.iter() {
            if func.name == name {
                return Ok(func.clone().into_val());
            }
        }

        Err(RuntimeError::NoKey(name.into()))
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
