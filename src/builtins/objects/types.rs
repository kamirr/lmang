use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::Val;

fn to_char(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, String> {
    args[0]
        .apply_to_root(|v| -> Result<_, String> { Ok(Val::Char(*v.as_number()? as u8 as char)) })?
}

pub fn make_types_builtin() -> Box<RustObj<()>> {
    RustObj::boxed("types", vec![RustFn::new("char", to_char)], None)
}
