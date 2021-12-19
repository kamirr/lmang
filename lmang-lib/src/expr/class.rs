use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::{func::FuncVal, Block};
use crate::utils::{self, kwords};
use crate::val::{DynFunc, DynObject, Object, Val, WeakWrapper};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Class(pub(crate) Block);

impl Class {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::CLASS, s)?;

        let (s, block_e) = Block::implicit(s)?;

        Ok((s, Class(block_e)))
    }
}

impl Eval for Class {
    fn eval(&self, env: &mut Env) -> Result<Val, RuntimeError> {
        env.eval(&self.0)?;

        let mut members = env.take_last_popped().unwrap();
        for (_, v) in members.iter_mut() {
            let owned_v = std::mem::replace(v, Val::Unit);
            *v = Val::Ref(Rc::new(RefCell::new(owned_v)));
        }

        let frozen = members.clone();

        for (key, v) in members.iter_mut() {
            if let Val::Ref(r) = v {
                let mut b = r.borrow_mut();

                if let Val::Func(DynFunc(df)) = &mut *b {
                    if let Some(func_val) = df.as_any_mut().downcast_mut::<FuncVal>() {
                        let mut subenv = frozen.clone();
                        let self_val = subenv.remove(key).unwrap();
                        let self_rc = self_val.as_val_ref()?;
                        let weak_val = Val::Weak(WeakWrapper(Rc::downgrade(self_rc)));
                        subenv.insert(key.clone(), weak_val);
                        func_val.parent = Some(subenv);
                    }
                }
            }
        }

        Ok(Val::Object(DynObject(Box::new(ClassObject { members }))))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClassObject {
    members: HashMap<String, Val, ahash::RandomState>,
}

impl Object for ClassObject {
    fn member_names(&self) -> Vec<String> {
        self.members.keys().map(|rs| rs.to_string()).collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        self.members
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::NoKey(name.to_string()))
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as fmt::Debug>::fmt(self, f)
    }

    fn name(&self) -> &str {
        "user-defined"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_update::Mode, BindingUpdate, Expr, Literal};

    #[test]
    fn parse_class_empty() {
        let class_e = Class::new("🧑‍🏫 🧑‍🦲");
        let expected = Class(Block { exprs: Vec::new() });
        assert_eq!(class_e, Ok(("", expected)));
    }

    #[test]
    fn parse_class_var() {
        let class_e = Class::new("🧑‍🏫 👶 x = 0 🧑‍🦲");
        let expected = Class(Block {
            exprs: vec![Expr::BindingUpdate(Box::new(BindingUpdate {
                name: "x".to_string(),
                val: Expr::Literal(Literal(Val::Number(0))),
                mode: Mode::CreateLocal,
            }))],
        });

        assert_eq!(class_e, Ok(("", expected)));
    }

    #[test]
    fn eval_class_member() {
        let (_, class_e) = Class::new("🧑‍🏫 👶 x = 0 🧑‍🦲").unwrap();
        let (_, use_e) = Expr::new("obj🪆x").unwrap();

        let mut env = Env::test();
        let class_val = env.eval(&class_e).unwrap();

        env.store_binding("obj".to_string(), class_val);

        let result = env.eval(&use_e);

        assert_eq!(result, Ok(Val::Ref(Rc::new(RefCell::new(Val::Number(0))))));
    }

    #[test]
    fn check_class_member_shared() {
        let (_, class_e) = Class::new("🧑‍🏫 👶 x = 0 🧑‍🦲").unwrap();
        let (_, let_e) = Expr::new("👶 oref = obj🪆x").unwrap();
        let (_, set_e) = Expr::new("♻️ oref = 4").unwrap();
        let (_, use_e) = Expr::new("obj🪆x").unwrap();

        let mut env = Env::test();
        let class_val = env.eval(&class_e).unwrap();
        env.store_binding("obj".to_string(), class_val);

        env.eval(&let_e).unwrap();
        env.eval(&set_e).unwrap();
        let result = env.eval(&use_e);

        assert_eq!(result, Ok(Val::Ref(Rc::new(RefCell::new(Val::Number(4))))));
    }

    #[test]
    fn check_class_member_separate() {
        let (_, class_e) = Class::new("🧑‍🏫 👶 x = 0 🧑‍🦲").unwrap();
        let (_, let_e) = Expr::new("👶 oref = obj1🪆x").unwrap();
        let (_, set_e) = Expr::new("♻️ oref = 4").unwrap();
        let (_, use_e) = Expr::new("obj2🪆x").unwrap();

        let mut env = Env::test();
        let class_val1 = env.eval(&class_e).unwrap();
        let class_val2 = env.eval(&class_e).unwrap();

        env.store_binding("obj1".to_string(), class_val1);
        env.store_binding("obj2".to_string(), class_val2);

        env.eval(&let_e).unwrap();
        env.eval(&set_e).unwrap();
        let result = env.eval(&use_e);

        assert_eq!(result, Ok(Val::Ref(Rc::new(RefCell::new(Val::Number(0))))));
    }

    #[test]
    fn check_class_method_scoping() {
        let (_, def_x) = Expr::new("👶 x = 10").unwrap();
        let (_, def_a) = Expr::new("👶 a = 10").unwrap();
        let (_, def_class) =
            Expr::new("👶 obj = 🧑‍🏫 👶 x = 0 💪 👶 sum = 🧰 ➡️ a + x 🧑‍🦲 🧑‍🦲").unwrap();
        let (_, method_call) = Expr::new("📞obj🪆sum").unwrap();
        let (_, get_x) = Expr::new("x").unwrap();
        let (_, get_a) = Expr::new("a").unwrap();

        let mut env = Env::test();
        env.eval(&def_x).unwrap();
        env.eval(&def_a).unwrap();
        env.eval(&def_class).unwrap();
        env.eval(&def_x).unwrap();
        env.eval(&def_a).unwrap();

        let result_call = env.eval(&method_call);
        assert_eq!(result_call, Ok(Val::Number(10)));

        let result_x = env.eval(&get_a);
        assert_eq!(result_x, Ok(Val::Number(10)));

        let result_a = env.eval(&get_x);
        assert_eq!(result_a, Ok(Val::Number(10)));
    }

    #[test]
    fn check_class_recurrent_method() {
        let (_, def_class) = Expr::new(
            "👶 obj = 🧑‍🏫
            👶 log2 = 🧰 n ➡️
                ❓ n < 1
                    0
                🧑‍🦲 😡
                    1 + 📞 log2 📦 n / 2 🧑‍🦲
                🧑‍🦲
            🧑‍🦲
        🧑‍🦲",
        )
        .unwrap();
        let (_, eval_log2_10) = Expr::new("📞 obj🪆log2 10").unwrap();

        let mut env = Env::test();
        env.eval(&def_class).unwrap();
        let result = env.eval(&eval_log2_10);

        assert_eq!(result, Ok(Val::Number(4)));
    }
}
