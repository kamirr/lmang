mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::system::System;
use crate::val::Val;
use rustfn::{FnState, RustFn};
use std::borrow::Cow;

pub struct Builtins<S: System> {
    system: S,
}

impl<S: System> Builtins<S> {
    pub fn new(system: S) -> Self {
        Builtins { system }
    }
}

impl<S: System> Eval for Builtins<S> {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        env.eval(&fns::BuiltinFns::new(
            self.system.print(),
            self.system.read(),
        ))?;
        env.eval(&objects::BuiltinObjects::new(self.system.args()))?;

        Ok(Cow::Owned(Val::Unit))
    }
}
