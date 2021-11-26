use crate::expr::func::Callee;
use crate::env::Env;
use crate::val::Val;
use std::fmt;

#[derive(Clone)]
pub struct BuiltinFn {
    dbg_name: String,
    func: fn(&[Val], &mut Env) -> Result<Val, String>,
}

impl BuiltinFn {
    #[cfg(test)]
    fn nop() -> BuiltinFn {
        BuiltinFn {
            dbg_name: "nop".to_string(),
            func: |_, _| Ok(Val::Unit),
        }
    }

    #[cfg(test)]
    fn id() -> BuiltinFn {
        BuiltinFn {
            dbg_name: "nop".to_string(),
            func: |args, _| Ok(args[0].clone()),
        }
    }
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinFn").field("dbg_name", &self.dbg_name).field("func", &"[...]").finish()
    }
}

impl Callee for BuiltinFn {
    fn call(&self, args: &[Val], env: &mut Env) -> Result<Val, String> {
        (self.func)(args, env)
    }

    fn clone_box(&self) -> Box<dyn Callee> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Env;
    use crate::expr::Expr;
    use crate::val::DynFunc;

    #[test]
    fn test_builtin_store_get() {
        let builtin_val = Val::Func(DynFunc(BuiltinFn::nop().clone_box()));

        let mut env = Env::test();
        env.store_binding("f".to_string(), builtin_val.clone());
        let result = env.get_binding("f").unwrap();

        assert_eq!(result, builtin_val);
    }

    #[test]
    fn eval_builtin() {
        let builtin_nop = Val::Func(DynFunc(BuiltinFn::nop().clone_box()));
        let builtin_id = Val::Func(DynFunc(BuiltinFn::id().clone_box()));

        let mut env = Env::test();
        env.store_binding("nop".to_string(), builtin_nop.clone());
        env.store_binding("id".to_string(), builtin_id.clone());
        env.store_binding("x".to_string(), Val::Number(42));

        let (_, nop_e) = Expr::new("📞 nop x").unwrap();
        let (_, id_e) = Expr::new("📞 id x").unwrap();

        let result_nop = env.eval(&nop_e);
        let result_id = env.eval(&id_e);

        assert_eq!(result_nop, Ok(Val::Unit));
        assert_eq!(result_id, Ok(Val::Number(42)));
    }
}