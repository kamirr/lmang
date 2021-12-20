use crate::env::Env;
use crate::val::{Callee, Val};

use std::any::Any;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone)]
pub struct FnState(pub Rc<RefCell<dyn Any>>);

impl Default for FnState {
    fn default() -> Self {
        FnState(Rc::new(RefCell::new(())))
    }
}

type BuiltinImpl = fn(&mut [Val], &mut Env, FnState) -> Result<Val, Val>;

#[derive(Clone)]
pub struct RustFn {
    pub name: String,
    pub func: BuiltinImpl,
    pub state: FnState,
}

impl RustFn {
    pub fn new(name: impl Into<String>, func: BuiltinImpl) -> Self {
        let rc = Rc::new(RefCell::new(()));
        Self::stateful(name, func, &rc)
    }

    pub fn stateful<T: 'static>(
        name: impl Into<String>,
        func: BuiltinImpl,
        state: &Rc<RefCell<T>>,
    ) -> Self {
        RustFn {
            name: name.into(),
            func,
            state: FnState(state.clone()),
        }
    }

    pub fn into_val(self) -> Val {
        Val::from_func(self)
    }

    #[cfg(test)]
    fn nop() -> Self {
        Self::new("nop", |_, _, _| Ok(Val::Unit))
    }

    #[cfg(test)]
    fn id() -> Self {
        Self::new("nop", |args, _, _| Ok(args[0].clone()))
    }

    #[cfg(test)]
    fn cnt() -> Self {
        fn func(_: &mut [Val], _: &mut Env, s: FnState) -> Result<Val, Val> {
            let mut borrow = s.0.borrow_mut();
            let n: &mut i32 = borrow.downcast_mut::<i32>().unwrap();
            let res = *n;
            *n += 1;

            Ok(Val::Number(res))
        }
        let state = Rc::new(RefCell::new(0));

        Self::stateful("cnt", func, &state)
    }
}

impl fmt::Debug for RustFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RustFn")
            .field("name", &self.name)
            .field("func", &"[...]")
            .finish()
    }
}

impl Callee for RustFn {
    fn call(&self, args: &mut [Val], env: &mut Env) -> Result<Val, Val> {
        (self.func)(args, env, self.state.clone())
    }

    fn clone_box(&self) -> Box<dyn Callee> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn dyn_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "🦀")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::Expr;

    #[test]
    fn test_rustfn_store_get() {
        let builtin_val = Val::from_func(RustFn::nop().clone());

        let mut env = Env::test();
        env.store_binding("f".to_string(), builtin_val.clone());
        let result = env.get_binding("f").unwrap();

        assert_eq!(result, builtin_val);
    }

    #[test]
    fn eval_rustfn() {
        let builtin_nop = Val::from_func(RustFn::nop().clone());
        let builtin_id = Val::from_func(RustFn::id().clone());

        let mut env = Env::test();
        env.store_binding("nop".to_string(), builtin_nop.clone());
        env.store_binding("id".to_string(), builtin_id.clone());
        env.store_binding("x".to_string(), Val::Number(42));

        let (_, nop_e) = Expr::new("📞 nop x").unwrap();
        let (_, id_e) = Expr::new("📞 id x").unwrap();

        let result_nop = env.eval(&nop_e);
        assert_eq!(result_nop, Ok(Val::Unit));

        let result_id = env.eval(&id_e);
        assert_eq!(result_id, Ok(Val::Number(42)));
    }

    #[test]
    fn eval_stateful_rustfn() {
        let rustfn_cnt = Val::from_func(RustFn::cnt().clone());

        let mut env = Env::test();
        env.store_binding("cnt".to_string(), rustfn_cnt.clone());

        for k in 0..100 {
            let (_, call_e) = Expr::new("📞 cnt").unwrap();
            let result = env.eval(&call_e);

            assert_eq!(result, Ok(Val::Number(k)));
        }
    }
}
