mod deque;
mod file;
mod rng;
mod rustobj;
mod sys;
mod types;

use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::val::{DynObject, Val};
use std::borrow::Cow;
use std::cell::RefCell;

use deque::make_deque_builtin;
use file::make_file_builtin;
use rng::make_rng_builtin;
use sys::make_sys_builtin;
use types::make_types_builtin;

pub(crate) struct BuiltinObjects {
    args: RefCell<Option<Box<dyn Iterator<Item = String>>>>,
}

impl BuiltinObjects {
    pub(crate) fn new(args: Box<dyn Iterator<Item = String>>) -> Self {
        Self {
            args: RefCell::new(Some(Box::new(args))),
        }
    }
}

impl Eval for BuiltinObjects {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        env.store_binding(
            "file".to_string(),
            Val::Object(DynObject(make_file_builtin())),
        );
        env.store_binding(
            "rng".to_string(),
            Val::Object(DynObject(make_rng_builtin())),
        );
        env.store_binding(
            "deque".to_string(),
            Val::Object(DynObject(make_deque_builtin())),
        );
        env.store_binding(
            "sys".to_string(),
            Val::Object(DynObject(make_sys_builtin(
                self.args.borrow_mut().take().unwrap(),
            ))),
        );
        env.store_binding(
            "types".to_string(),
            Val::Object(DynObject(make_types_builtin())),
        );

        Ok(Cow::Owned(Val::Unit))
    }
}
