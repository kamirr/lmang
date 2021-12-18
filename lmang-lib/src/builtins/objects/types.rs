use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::view::{self, test_consumed, view1};
use crate::val::Val;

fn to_char(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res_char, tail) =
        view1::<view::AnyRef<view::Number>, _, _>(args, |n| Ok(*n as u8 as char))?;
    test_consumed(tail)?;

    Ok(Val::Char(res_char))
}

pub(crate) fn make_types_builtin() -> Box<RustObj> {
    RustObj::boxed("types", vec![RustFn::new("char", to_char)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn val_to_char() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let val = Val::Number(24);
        let mut env = Env::test();
        let state = FnState(Rc::new(RefCell::new(())));

        assert_eq!(
            to_char([val.clone()].as_mut(), &mut env, state),
            Ok(Val::Char(24 as char))
        );
    }
}
