use crate::env::{Env, Eval};
use crate::expr::block::Block;
use crate::utils::{self, kwords};
use crate::val::{Callee, DynFunc, Val};
use std::borrow::Cow;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Arg(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub args: Vec<Arg>,
    pub body: Block,
}

impl Callee for Func {
    fn call(&self, args: &[Val], env: &mut Env) -> Result<Val, String> {
        if args.len() != self.args.len() {
            return Err("Invalid number of args".to_string());
        }

        env.push();
        for (Arg(arg_name), arg_val) in self.args.iter().zip(args.iter()) {
            env.store_binding(arg_name.clone(), arg_val.clone());
        }

        let result = env.eval(&self.body).map(|cow| cow.as_ref().to_owned());
        env.pop();

        result
    }

    fn clone_box(&self) -> Box<dyn Callee> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }
}

impl Func {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::FUNC, s)?;

        let mut s = s;
        let mut args = Vec::new();

        let (new_s, _) = utils::extract_whitespace(s);
        s = new_s;

        while let Ok((new_s, ident)) = utils::extract_ident(s) {
            let (new_s, _) = utils::extract_whitespace(new_s);
            s = new_s;

            args.push(Arg(ident.to_string()));
        }

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::FUNC_SEP, s)?;

        let (s, body) = Block::implicit(s)?;

        Ok((s, Func { args, body }))
    }
}

impl Eval for Func {
    fn eval<'a, 'b>(&'a self, _env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        Ok(Cow::Owned(Val::Func(DynFunc(Box::new(self.clone())))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_update::BindingUpdate, binding_usage::BindingUsage, Expr, Op};

    #[test]
    fn func_parse_id() {
        let func_e = Func::new("🧰 a ➡️ a 🧑‍🦲");
        let expected = Func {
            args: vec![Arg("a".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn func_parse_sum() {
        let func_e = Func::new("🧰 a b ➡️ a + b 🧑‍🦲");
        let expected = Func {
            args: vec![Arg("a".to_string()), Arg("b".to_string())],
            body: Block {
                exprs: vec![Expr::Operation {
                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    })),
                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "b".to_string(),
                    })),
                    op: Op::Add,
                }],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn func_parse_id_in_block() {
        let expr = Expr::new("📦 👶 id = 🧰 a ➡️ a 🧑‍🦲 💪 id 🧑‍🦲");
        let expected = Expr::Block(Block {
            exprs: vec![
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "id".to_string(),
                    val: Expr::Func(Box::new(Func {
                        args: vec![Arg("a".to_string())],
                        body: Block {
                            exprs: vec![Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })],
                        },
                    })),
                    set: false,
                })),
                Expr::BindingUsage(BindingUsage {
                    name: "id".to_string(),
                }),
            ],
        });

        assert_eq!(expr, Ok(("", expected)));
    }

    #[test]
    fn func_eval_id() {
        let (_, func_e) = Func::new("🧰 a ➡️ a 🧑‍🦲").unwrap();
        let expected = Func {
            args: vec![Arg("a".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })],
            },
        };

        let mut env = Env::test();
        let result = env.eval(&func_e);

        assert_eq!(result, Ok(Cow::Owned(Val::Func(DynFunc(Box::new(expected))))));
    }

    #[test]
    fn func_var_shadowing() {
        let (_, func_def) = Expr::new(
            "👶 f = 🧰 a ➡️
                👶 b = 2  💪
                set a = 1 💪
                set x = 1 💪
                set c = 1 💪
                set b = 4 💪
                
                a
            🧑‍🦲",
        )
        .unwrap();
        let (_, func_call) = Expr::new("📞 f x").unwrap();

        let mut env = Env::test();
        env.eval(&func_def).unwrap();

        env.store_binding("c".to_string(), Val::Number(0));
        env.store_binding("x".to_string(), Val::Number(8));
        let res = env.eval(&func_call);

        assert_eq!(res, Ok(Cow::Owned(Val::Number(1))));
        assert_eq!(env.get_binding("c"), Ok(Cow::Borrowed(&Val::Number(1))));
        assert_eq!(env.get_binding("x"), Ok(Cow::Borrowed(&Val::Number(1))));
        assert_eq!(
            env.get_binding("a"),
            Err("binding with name `a` does not exist".to_string())
        );
        assert_eq!(
            env.get_binding("b"),
            Err("binding with name `b` does not exist".to_string())
        );
    }
}
