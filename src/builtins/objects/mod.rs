mod deque;
mod rng;
mod rustobj;

use crate::env::{Env, Eval};
use crate::val::{DynObject, Val};
use std::borrow::Cow;

use deque::make_deque_builtin;
use rng::make_rng_builtin;

pub struct BuiltinObjects;

impl Eval for BuiltinObjects {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.store_binding(
            "rng".to_string(),
            Val::Object(DynObject(make_rng_builtin())),
        );
        env.store_binding(
            "deque".to_string(),
            Val::Object(DynObject(make_deque_builtin())),
        );

        Ok(Cow::Owned(Val::Unit))
    }
}
