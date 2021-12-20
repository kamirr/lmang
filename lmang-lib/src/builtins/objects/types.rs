use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::view::{self, test_consumed, view1};
use crate::val::Val;

fn to_char(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, Val> {
    let (res_char, tail) =
        view1::<view::AnyRef<view::Number>, _, _>(args, |n| Ok(*n as u8 as char))?;
    test_consumed(tail)?;

    Ok(Val::Char(res_char))
}

fn to_string(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, Val> {
    let (res, tail) = view1::<view::AnyRef<view::Bottom>, _, _>(args, |v| Ok(format!("{}", v)))?;
    test_consumed(tail)?;

    Ok(Val::from_str(res.as_ref()))
}

#[cfg(feature = "web")]
fn jv_to_val(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, Val> {
    let (val, tail) = view1::<view::Js, _, _>(args, |jv| Ok(Val::convert_from_jv(jv.clone())))?;
    test_consumed(tail)?;

    Ok(val)
}

pub(crate) fn make_types_builtin() -> RustObj {
    RustObj::new(
        "types",
        vec![
            RustFn::new("char", to_char),
            RustFn::new("string", to_string),
            #[cfg(feature = "web")]
            RustFn::new("fromJs", jv_to_val),
        ],
    )
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
