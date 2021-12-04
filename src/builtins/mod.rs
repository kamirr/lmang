mod fns;
mod objects;
mod rustfn;

use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::val::Val;
use rustfn::{FnState, RustFn};
use std::borrow::Cow;
use std::cell::RefCell;

pub struct Builtins {
    args: RefCell<Option<Box<dyn Iterator<Item = String>>>>,
    print_impl: RefCell<Option<fns::PrintImpl>>,
}

impl Builtins {
    pub fn new(args: impl Iterator<Item = String> + 'static, print_impl: fns::PrintImpl) -> Self {
        Builtins {
            args: RefCell::new(Some(Box::new(args))),
            print_impl: RefCell::new(Some(print_impl)),
        }
    }
}

impl Eval for Builtins {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        let args = self.args.borrow_mut().take().unwrap();
        let print_impl = self.print_impl.borrow_mut().take().unwrap();

        env.eval(&fns::BuiltinFns::new(print_impl))?;
        env.eval(&objects::BuiltinObjects::new(args))?;

        Ok(Cow::Owned(Val::Unit))
    }
}
