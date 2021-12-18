use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::view::{self, test_consumed, view1};
use crate::val::Val;
use rand::rngs::SmallRng;
use rand::{RngCore as _, SeedableRng};
use std::cell::RefCell;
use std::rc::Rc;

fn next(tail: &mut [Val], _: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    test_consumed(tail)?;

    let mut borrow = state.0.borrow_mut();
    let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();

    Ok(Val::Number(rng.next_u32() as i32))
}

fn seed(args: &mut [Val], _: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let (new_seed, tail) = view1::<view::Number, _, _>(args, |n| Ok(*n))?;
    test_consumed(tail)?;

    let mut borrow = state.0.borrow_mut();
    let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();
    *rng = SmallRng::seed_from_u64(new_seed as u64);

    Ok(Val::Unit)
}

pub(crate) fn make_rng_builtin() -> Box<RustObj> {
    let state = Rc::new(RefCell::new(SmallRng::seed_from_u64(0)));

    RustObj::boxed(
        "rng",
        vec![
            RustFn::stateful("next", next, &state),
            RustFn::stateful("seed", seed, &state),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;
    use crate::val::DynObject;

    #[test]
    fn rng_builtin_seedable() {
        let mut env = Env::test();
        env.store_binding(
            "rng".to_string(),
            Val::Object(DynObject(make_rng_builtin())),
        );

        let (_, seed_e) = Expr::new("📞 rng🪆seed 21").unwrap();
        let (_, next_e) = Expr::new("📞 rng🪆next").unwrap();

        env.eval(&seed_e).unwrap();
        let result_1 = *env.eval(&next_e).unwrap().as_number().unwrap();

        for _ in 0..24 {
            env.eval(&seed_e).unwrap();
            let result_n = *env.eval(&next_e).unwrap().as_number().unwrap();

            assert_eq!(result_n, result_1);
        }
    }
}
