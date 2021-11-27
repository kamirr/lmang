use crate::val::Val;
use std::fmt;

pub trait Callee {
    fn call(&self, args: &[Val], env: &mut crate::env::Env) -> Result<Val, String>;
    fn clone_box(&self) -> Box<dyn Callee>;
    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub struct DynFunc(pub Box<dyn Callee>);

impl Clone for DynFunc {
    fn clone(&self) -> Self {
        DynFunc(self.0.clone_box())
    }
}

impl PartialEq for DynFunc {
    fn eq(&self, other: &DynFunc) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl fmt::Debug for DynFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_debug(f)
    }
}
