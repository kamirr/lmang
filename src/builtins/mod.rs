mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::val::Val;
use rustfn::{FnState, RustFn};
use std::borrow::Cow;
use std::cell::RefCell;

pub struct Builtins {
    args: RefCell<Option<Box<dyn Iterator<Item = String>>>>,
}

impl Builtins {
    pub fn new(args: impl Iterator<Item = String> + 'static) -> Self {
        Builtins {
            args: RefCell::new(Some(Box::new(args))),
        }
    }
}

impl Eval for Builtins {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        let args = self.args.borrow_mut().take().unwrap();

        env.eval(&fns::BuiltinFns)?;
        env.eval(&objects::BuiltinObjects::new(args))?;

        Ok(Cow::Owned(Val::Unit))
    }
}
