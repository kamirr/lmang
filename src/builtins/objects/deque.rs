use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::{Object, Val};
use std::collections::VecDeque;
use std::fmt;

#[derive(Clone, Debug)]
pub struct DequeBuiltin { }

impl DequeBuiltin {
    pub fn boxed() -> Box<Self> {
        Box::new(DequeBuiltin { })
    }

    fn len(args: &[Val], _: &mut Env, _: FnState) -> Result<Val, String> {
        let dq_val = args[0].try_match_type(&Val::Deque(Box::new(VecDeque::new())))?;
        let dq = dq_val.as_deque()?;

        Ok(Val::Number(dq.len() as i32))
    }
}

impl Object for DequeBuiltin {
    fn member_names(&self) -> Vec<String> {
        vec!["len".to_string()]
    }

    fn member(&self, name: &str) -> Result<Val, String> {
        match name {
            "len" => {
                let func = RustFn::new("len", DequeBuiltin::len).into_val();
                Ok(func)
            }
            _ => Err(format!("no member {}", name)),
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}