mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::val::Val;
use rustfn::{FnState, RustFn};

pub struct Builtins;

impl Eval for Builtins {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        env.eval(&fns::BuiltinFns)?;
        env.eval(&objects::BuiltinObjects)?;

        Ok(Val::Unit)
    }
}
