use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::Val;
use std::borrow::Borrow;
use std::collections::VecDeque;

fn len(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let len =
        args[0].apply_to_root(|v| -> Result<_, RuntimeError> { Ok(v.as_deque()?.len()) })??;

    Ok(Val::Number(len as i32))
}

fn append(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgsN);
    }

    let val = args[1].clone();
    args[0]
        .as_val_ref()
        .borrow()
        .as_ref()?
        .borrow_mut()
        .apply_to_root_mut(|v| -> Result<_, RuntimeError> {
            v.as_deque_mut()?.push_back(val);
            Ok(())
        })??;

    Ok(Val::Unit)
}

fn at(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgsN);
    }

    let idx = args[1].apply_to_root(|v| v.as_number().map(|n| *n))??;
    let result = args[0].apply_to_root(|v| -> Result<_, RuntimeError> {
        Ok(v.as_deque()?[idx as usize].clone())
    })??;

    Ok(result)
}

fn at_mut(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError::WrongArgsN);
    }

    let idx = args[1].apply_to_root(|v| v.as_number().map(|n| *n))??;
    let result = match args[0].as_val_ref().borrow().as_ref() {
        Ok(vr) => vr
            .borrow_mut()
            .apply_to_root_mut(|v| -> Result<_, RuntimeError> {
                let val_ref = v.as_deque_mut()?[idx as usize].make_ref();

                Ok(val_ref)
            })??,
        _ => args[0].as_deque()?[idx as usize].clone(),
    };

    Ok(result)
}

fn remove(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    use std::ops::Sub;

    if args.len() != 2 {
        return Err(RuntimeError::WrongArgsN);
    }

    let idx = args[1].apply_to_root(|v| v.as_number().map(|n| *n))??;
    let result = match args[0].as_val_ref().borrow().as_ref() {
        Ok(vr) => vr
            .borrow_mut()
            .apply_to_root_mut(|v| -> Result<_, RuntimeError> {
                let dq_len = v.as_deque()?.len();
                let idx_rel = if idx >= 0 {
                    idx as usize
                } else {
                    dq_len.sub((-idx) as usize)
                };

                v.as_deque_mut()?
                    .remove(idx_rel)
                    .ok_or_else(|| RuntimeError::OutOfBounds { idx, len: dq_len })
            })??,
        _ => args[0].as_deque()?[idx as usize].clone(),
    };

    Ok(result)
}

fn flatten(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError::WrongArgsN);
    }

    let mut res = VecDeque::new();
    fn flatten_impl(v: &Val, dq: &mut VecDeque<Val>) {
        v.apply_to_root(move |v| match v.as_deque() {
            Ok(inner_dq) => {
                for v_inner in inner_dq.iter() {
                    flatten_impl(v_inner, dq)
                }
            },
            Err(_) => {
                dq.push_back(v.clone())
            }
        }).unwrap()
    }

    flatten_impl(&args[0], &mut res);
    Ok(Val::Deque(Box::new(res)))
}

pub(crate) fn make_deque_builtin() -> Box<RustObj<()>> {
    RustObj::boxed(
        "deque",
        vec![
            RustFn::new("len", len),
            RustFn::new("append", append),
            RustFn::new("at", at),
            RustFn::new("mut", at_mut),
            RustFn::new("remove", remove),
            RustFn::new("flatten", flatten),
        ],
        None,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::val::DynObject;
    use std::borrow::Cow;
    use std::cell::RefCell;
    use std::collections::VecDeque;
    use std::rc::Rc;

    fn deque_123_val() -> Val {
        let mut vd = VecDeque::new();
        vd.push_back(Val::Number(1));
        vd.push_back(Val::Number(2));
        vd.push_back(Val::Number(3));

        Val::Deque(Box::new(vd))
    }

    fn deque_1234_val() -> Val {
        let mut vd = VecDeque::new();
        vd.push_back(Val::Number(1));
        vd.push_back(Val::Number(2));
        vd.push_back(Val::Number(3));
        vd.push_back(Val::Number(4));

        Val::Deque(Box::new(vd))
    }

    fn deque_test_env() -> Env {
        let mut env = Env::test();
        env.store_binding("d".to_string(), deque_123_val());
        env.store_binding(
            "d_test".to_string(),
            Val::Object(DynObject(make_deque_builtin())),
        );

        env
    }

    #[test]
    fn test_len() {
        let mut env = deque_test_env();
        let (_, call_val_e) = Expr::new("📞 d_test🪆len d").unwrap();
        let (_, call_ref_e) = Expr::new("📞 d_test🪆len 🔖d").unwrap();
        let expected = Ok(Cow::Owned(Val::Number(3)));

        let result_val = env.eval(&call_val_e);
        assert_eq!(result_val, expected);

        let result_ref = env.eval(&call_ref_e);
        assert_eq!(result_ref, expected);
    }

    #[test]
    fn test_append() {
        let mut env = deque_test_env();
        let (_, call_val_e) = Expr::new("📞 d_test🪆append d 4").unwrap();

        let result_val = env.eval(&call_val_e);
        assert_eq!(
            result_val,
            Err(RuntimeError::CastError {
                from: "😵‍💫😵‍💫".into(),
                to: "🔖".into()
            })
        );

        let (_, call_ref_e) = Expr::new("📞 d_test🪆append 🔖d 4").unwrap();
        assert_eq!(env.eval(&call_ref_e), Ok(Cow::Owned(Val::Unit)));

        let result_ref = env.get_binding("d");
        assert_eq!(
            result_ref,
            Ok(Cow::Owned(Val::Ref(Rc::new(
                RefCell::new(deque_1234_val())
            ))))
        );
    }

    #[test]
    fn test_at() {
        let mut env = deque_test_env();
        let (_, call_val_e) = Expr::new("📞 d_test🪆at d 2").unwrap();
        let (_, call_ref_e) = Expr::new("📞 d_test🪆at 🔖d 2").unwrap();

        let result_val = env.eval(&call_val_e);
        assert_eq!(result_val, Ok(Cow::Owned(Val::Number(3))));

        let result_val = env.eval(&call_ref_e);
        assert_eq!(result_val, Ok(Cow::Owned(Val::Number(3))));
    }

    #[test]
    fn test_at_mut() {
        let mut env = deque_test_env();
        let (_, call_e) = Expr::new("📞 d_test🪆mut 🔖d 2").unwrap();

        let result_val = env.eval(&call_e);
        assert_eq!(
            result_val,
            Ok(Cow::Owned(Val::Ref(Rc::new(RefCell::new(Val::Number(3))))))
        );

        let dq = env.get_binding("d").unwrap();
        let expected_dq = {
            let mut tmp = VecDeque::new();
            tmp.push_back(Val::Number(1));
            tmp.push_back(Val::Number(2));
            tmp.push_back(Val::Ref(Rc::new(RefCell::new(Val::Number(3)))));

            tmp
        };
        let expected_dq_val = Val::Ref(Rc::new(RefCell::new(Val::Deque(Box::new(expected_dq)))));
        let expected = Cow::Borrowed(&expected_dq_val);
        assert_eq!(dq, expected);
    }

    #[test]
    fn test_remove() {
        let mut env = deque_test_env();
        let (_, remove_2_e) = Expr::new("📞 d_test🪆remove 🔖d 2").unwrap();
        let (_, remove_20_e) = Expr::new("📞 d_test🪆remove 🔖d 20").unwrap();

        let result = env.eval(&remove_2_e);
        assert_eq!(result, Ok(Cow::Owned(Val::Number(3))));

        let result = env.eval(&remove_20_e);
        assert_eq!(result, Err(RuntimeError::OutOfBounds { idx: 20, len: 2 }));

        let dq = env.get_binding("d").unwrap();
        let expected_dq = {
            let mut tmp = VecDeque::new();
            tmp.push_back(Val::Number(1));
            tmp.push_back(Val::Number(2));

            tmp
        };
        let expected_dq_val = Val::Ref(Rc::new(RefCell::new(Val::Deque(Box::new(expected_dq)))));
        let expected = Cow::Borrowed(&expected_dq_val);
        assert_eq!(dq, expected);
    }

    #[test]
    fn test_flatten() {
        let mut env = deque_test_env();
        let (_, append_ab_e) = Expr::new("📞 d_test🪆append 🔖d 🧵ab🧵").unwrap();
        let (_, flatten_e) = Expr::new("👶 d = 📞 d_test🪆flatten 🔖d").unwrap();
        let expected = vec![
            Val::Number(1),
            Val::Number(2),
            Val::Number(3),
            Val::Char('a'),
            Val::Char('b'),
            Val::Char('a'),
            Val::Char('b'),
            Val::Char('a'),
            Val::Char('b'),
        ].into_iter().collect();

        for _ in 0..3 {
            env.eval(&append_ab_e).unwrap();
        }
        env.eval(&flatten_e).unwrap();

        let val = env.get_binding("d");
        assert_eq!(val, Ok(Cow::Owned(Val::Deque(Box::new(expected)))));
    }
}
