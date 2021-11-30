mod deque;
mod rng;

use crate::env::{Env, Eval};
use crate::val::{DynObject, Val};
use std::borrow::Cow;

use deque::DequeBuiltin;
use rng::RngBuiltin;

pub struct BuiltinObjects;

impl Eval for BuiltinObjects {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.store_binding(
            "rng".to_string(),
            Val::Object(DynObject(RngBuiltin::boxed())),
        );
        env.store_binding(
            "deque".to_string(),
            Val::Object(DynObject(DequeBuiltin::boxed())),
        );

        Ok(Cow::Owned(Val::Unit))
    }
}
