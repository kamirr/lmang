use crate::env::{Env, Eval};
use crate::val::{Callee, DynFunc, Val};
use std::fmt;

type BuiltinImpl = fn(&[Val], &mut Env) -> Result<Val, String>;

fn print(args: &[Val], _env: &mut Env) -> Result<Val, String> {
    if args.len() > 0 {
        for arg in &args[0..args.len() - 1] {
            print!("{} ", arg);
        }
        println!("{}", args.last().unwrap());
    } else {
        println!("");
    }

    Ok(Val::Unit)
}

#[derive(Clone)]
struct BuiltinFn {
    dbg_name: String,
    func: BuiltinImpl,
}

impl BuiltinFn {
    fn new(name: impl Into<String>, func: BuiltinImpl) -> Self {
        BuiltinFn {
            dbg_name: name.into(),
            func,
        }
    }

    fn into_val(self) -> Val {
        Val::Func(DynFunc(Box::new(self)))
    }

    #[cfg(test)]
    fn nop() -> Self {
        BuiltinFn {
            dbg_name: "nop".to_string(),
            func: |_, _| Ok(Val::Unit),
        }
    }

    #[cfg(test)]
    fn id() -> Self {
        BuiltinFn {
            dbg_name: "id".to_string(),
            func: |args, _| Ok(args[0].clone()),
        }
    }
}

impl fmt::Debug for BuiltinFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BuiltinFn")
            .field("dbg_name", &self.dbg_name)
            .field("func", &"[...]")
            .finish()
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

pub struct Builtins;

impl Eval for Builtins {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        env.store_binding(
            "🗣️".to_string(),
            BuiltinFn::new("🗣️__print", print).into_val(),
        );

        Ok(Val::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
