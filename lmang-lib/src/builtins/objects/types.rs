use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::Val;

fn to_char(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    args[0].apply_to_root(|v| -> Result<_, RuntimeError> {
        Ok(Val::Char(*v.as_number()? as u8 as char))
    })?
}

pub(crate) fn make_types_builtin() -> Box<RustObj> {
    RustObj::boxed("types", vec![RustFn::new("char", to_char)])
}
