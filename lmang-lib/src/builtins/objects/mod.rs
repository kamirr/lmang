mod deque;
mod file;
mod rng;
mod rustobj;
mod sys;
mod types;

#[cfg(feature = "web")]
mod web;

use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::val::Val;
use std::cell::RefCell;

use deque::make_deque_builtin;
use file::make_file_builtin;
use rng::make_rng_builtin;
use sys::make_sys_builtin;
use types::make_types_builtin;

#[cfg(feature = "web")]
pub use web::make_web_builtin;

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
    fn eval(&self, env: &mut Env) -> Result<Val, RuntimeError> {
        env.store_binding("file".to_string(), Val::from_obj(make_file_builtin()));
        env.store_binding("rng".to_string(), Val::from_obj(make_rng_builtin()));
        env.store_binding("deque".to_string(), Val::from_obj(make_deque_builtin()));
        env.store_binding(
            "sys".to_string(),
            Val::from_obj(make_sys_builtin(self.args.borrow_mut().take().unwrap())),
        );
        env.store_binding("types".to_string(), Val::from_obj(make_types_builtin()));

        #[cfg(feature = "web")]
        #[cfg(target_arch = "wasm32")]
        env.store_binding("js".to_string(), Val::from(make_web_builtin()));

        Ok(Val::Unit)
    }
}
