use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::view::{self, foreach, take_n, test_consumed, view1, view2, DequeExt as _};
use crate::val::Val;
use std::collections::VecDeque;

fn len(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (val, tail) =
        view1::<view::AnyRef<view::Deque>, _, _>(args, |dq| Ok(Val::Number(dq.len() as i32)))?;
    test_consumed(tail)?;

    Ok(val)
}

fn append(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res, tail) = view2::<view::Ref<view::Deque>, view::Bottom, _, _>(args, |dq, new_val| {
        dq.push_back(new_val.clone());
        Ok(Val::Unit)
    })?;
    test_consumed(tail)?;

    Ok(res)
}

fn concat(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res, tail) = view2::<view::Ref<view::Deque>, view::AnyRef<view::Deque>, _, _>(args, |dq1, dq2| {
        dq1.extend(dq2.iter().cloned());
        Ok(Val::Unit)
    })?;
    test_consumed(tail)?;

    Ok(res)
}

fn at(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res, tail) = view2::<view::AnyRef<view::Deque>, view::Number, _, _>(args, |dq, idx| {
        Ok(dq.try_get(*idx)?.clone())
    })?;
    test_consumed(tail)?;

    Ok(res)
}

fn at_mut(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res, tail) = view2::<view::Ref<view::Deque>, view::Number, _, _>(args, |dq, idx| {
        Ok(dq.try_get(*idx)?.make_ref())
    })?;
    test_consumed(tail)?;

    Ok(res)
}

fn remove(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (res, tail) = view2::<view::Ref<view::Deque>, view::Number, _, _>(args, |dq, &mut idx| {
        dq.try_remove(idx)
    })?;
    test_consumed(tail)?;

    Ok(res)
}

fn flatten(args: &mut [Val], _env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let mut res = VecDeque::new();
    fn flatten_impl(v: &mut Val, res: &mut VecDeque<Val>) {
        let mut wrapped = [v.clone()];

        let recurrence_res =
            foreach::<view::AnyRef<view::Deque>, view::Bottom, _, _>(&mut wrapped, |v| {
                flatten_impl(v, res);
                Ok(())
            });

        if recurrence_res.is_err() {
            let [v] = wrapped;
            res.push_back(v);
        }
    }

    let ([val], tail) = take_n::<1>(args)?;
    flatten_impl(val, &mut res);
    test_consumed(tail)?;

    Ok(Val::Deque(Box::new(res)))
}

pub(crate) fn make_deque_builtin() -> Box<RustObj> {
    RustObj::boxed(
        "deque",
        vec![
            RustFn::new("len", len),
            RustFn::new("append", append),
            RustFn::new("concat", concat),
            RustFn::new("at", at),
            RustFn::new("mut", at_mut),
            RustFn::new("remove", remove),
            RustFn::new("flatten", flatten),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::val::DynObject;
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
        let expected = Ok(Val::Number(3));

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
        assert_eq!(env.eval(&call_ref_e), Ok(Val::Unit));

        let result_ref = env.get_binding("d");
        assert_eq!(
            result_ref,
            Ok(Val::Ref(Rc::new(RefCell::new(deque_1234_val()))))
        );
    }

    #[test]
    fn test_at() {
        let mut env = deque_test_env();
        let (_, call_val_e) = Expr::new("📞 d_test🪆at d 📦0-1🧑‍🦲").unwrap();
        let (_, call_ref_e) = Expr::new("📞 d_test🪆at 🔖d 2").unwrap();

        let result_val = env.eval(&call_val_e);
        assert_eq!(result_val, Ok(Val::Number(3)));

        let result_val = env.eval(&call_ref_e);
        assert_eq!(result_val, Ok(Val::Number(3)));
    }

    #[test]
    fn test_at_mut() {
        let mut env = deque_test_env();
        let (_, call_e) = Expr::new("📞 d_test🪆mut 🔖d 📦0-1🧑‍🦲").unwrap();

        let result_val = env.eval(&call_e);
        assert_eq!(
            result_val,
            Ok(Val::Ref(Rc::new(RefCell::new(Val::Number(3)))))
        );

        let dq = env.get_binding("d").unwrap();
        let expected_dq = {
            let mut tmp = VecDeque::new();
            tmp.push_back(Val::Number(1));
            tmp.push_back(Val::Number(2));
            tmp.push_back(Val::Ref(Rc::new(RefCell::new(Val::Number(3)))));

            tmp
        };
        let expected = Val::Ref(Rc::new(RefCell::new(Val::Deque(Box::new(expected_dq)))));
        assert_eq!(dq, expected);
    }

    #[test]
    fn test_at_at_mut_oob() {
        let mut env = deque_test_env();
        let (_, at_e) = Expr::new("📞 d_test🪆at 🔖d 10").unwrap();
        let (_, at_mut_e) = Expr::new("📞 d_test🪆mut 🔖d 10").unwrap();

        let expected = RuntimeError::OutOfBounds { len: 3, idx: 10 };

        assert_eq!(env.eval(&at_e), Err(expected.clone()));
        assert_eq!(env.eval(&at_mut_e), Err(expected));
    }

    #[test]
    fn test_remove() {
        let mut env = deque_test_env();
        let (_, remove_2_e) = Expr::new("📞 d_test🪆remove 🔖d 2").unwrap();
        let (_, remove_20_e) = Expr::new("📞 d_test🪆remove 🔖d 20").unwrap();

        let result = env.eval(&remove_2_e);
        assert_eq!(result, Ok(Val::Number(3)));

        let result = env.eval(&remove_20_e);
        assert_eq!(result, Err(RuntimeError::OutOfBounds { idx: 20, len: 2 }));

        let dq = env.get_binding("d").unwrap();
        let expected_dq = {
            let mut tmp = VecDeque::new();
            tmp.push_back(Val::Number(1));
            tmp.push_back(Val::Number(2));

            tmp
        };
        let expected = Val::Ref(Rc::new(RefCell::new(Val::Deque(Box::new(expected_dq)))));
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
        ]
        .into_iter()
        .collect();

        for _ in 0..3 {
            env.eval(&append_ab_e).unwrap();
        }
        env.eval(&flatten_e).unwrap();

        let val = env.get_binding("d");
        assert_eq!(val, Ok(Val::Deque(Box::new(expected))));
    }

    #[test]
    fn test_too_many_args() {
        let mut env = deque_test_env();
        let (_, len_e) = Expr::new("📞 d_test🪆len d 🧵extra_arg🧵").unwrap();
        let (_, append_e) = Expr::new("📞 d_test🪆append 🔖d 24 🧵extra_arg🧵").unwrap();
        let (_, at_e) = Expr::new("📞 d_test🪆at 🔖d 0 🧵extra_arg🧵").unwrap();
        let (_, at_mut_e) = Expr::new("📞 d_test🪆mut 🔖d 0 🧵extra_arg🧵").unwrap();
        let (_, remove_e) = Expr::new("📞 d_test🪆remove 🔖d 0 🧵extra_arg🧵").unwrap();
        let (_, flatten_e) = Expr::new("📞 d_test🪆flatten d 🧵extra_arg🧵").unwrap();

        assert_eq!(env.eval(&len_e), Err(RuntimeError::WrongArgsN));
        assert_eq!(env.eval(&append_e), Err(RuntimeError::WrongArgsN));
        assert_eq!(env.eval(&at_e), Err(RuntimeError::WrongArgsN));
        assert_eq!(env.eval(&at_mut_e), Err(RuntimeError::WrongArgsN));
        assert_eq!(env.eval(&remove_e), Err(RuntimeError::WrongArgsN));
        assert_eq!(env.eval(&flatten_e), Err(RuntimeError::WrongArgsN));

        let expected_dq = {
            let mut dq = VecDeque::new();
            dq.push_back(Val::Number(2));
            dq.push_back(Val::Number(3));
            dq.push_back(Val::Number(24));

            dq
        };
        let expected_dq_val = Val::Deque(Box::new(expected_dq));
        let expected_val = Val::Ref(Rc::new(RefCell::new(expected_dq_val)));
        assert_eq!(env.get_binding("d"), Ok(expected_val));
    }
}
