mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::system::System;
use crate::val::Val;
use rustfn::{FnState, RustFn};

pub struct Builtins<S: System> {
    system: S,
}

impl<S: System> Builtins<S> {
    pub fn new(system: S) -> Self {
        Builtins { system }
    }
}

impl<S: System> Eval for Builtins<S> {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        env.eval(&fns::BuiltinFns::new(
            self.system.print(),
            self.system.read(),
        ))?;
        env.eval(&objects::BuiltinObjects::new(self.system.args()))?;

        Ok(Val::Unit)
    }
}
