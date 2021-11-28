mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::val::Val;
use rustfn::{FnState, RustFn};
use std::borrow::Cow;

pub struct Builtins;

impl Eval for Builtins {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.eval(&fns::BuiltinFns)?;
        env.eval(&objects::BuiltinObjects)?;

        Ok(Cow::Owned(Val::Unit))
    }
}
