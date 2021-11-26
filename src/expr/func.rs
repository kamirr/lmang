use crate::env::{Env, Eval};
use crate::expr::block::Block;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Arg(pub String);

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub args: Vec<Arg>,
    pub body: Block,
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
    fn eval(&self, _env: &mut Env) -> Result<Val, String> {
        Ok(Val::Func(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_update::BindingUpdate;
    use crate::expr::{binding_usage::BindingUsage, Expr, Op};
    use crate::stmt::Stmt;

    #[test]
    fn func_parse_id() {
        let func_e = Func::new("🧰 a ➡️ a 🧑‍🦲");
        let expected = Func {
            args: vec![Arg("a".to_string())],
            body: Block {
                stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                }))],
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
                stmts: vec![Stmt::Expr(Expr::Operation {
                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    })),
                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "b".to_string(),
                    })),
                    op: Op::Add,
                })],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn func_parse_id_in_block() {
        let stmt = Stmt::new("📦 👶 id = 🧰 a ➡️ a 🧑‍🦲 💪 id 🧑‍🦲");
        let expected = Stmt::Expr(Expr::Block(Block {
            stmts: vec![
                Stmt::BindingUpdate(BindingUpdate {
                    name: "id".to_string(),
                    val: Expr::Func(Box::new(Func {
                        args: vec![Arg("a".to_string())],
                        body: Block {
                            stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            }))],
                        },
                    })),
                    set: false,
                }),
                Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "id".to_string(),
                })),
            ],
        }));

        assert_eq!(stmt, Ok(("", expected)));
    }

    #[test]
    fn func_eval_id() {
        let (_, func_e) = Func::new("🧰 a ➡️ a 🧑‍🦲").unwrap();
        let expected = Func {
            args: vec![Arg("a".to_string())],
            body: Block {
                stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                }))],
            },
        };

        let mut env = Env::test();
        let result = env.eval(&func_e);

        assert_eq!(result, Ok(Val::Func(expected)));
    }

    #[test]
    fn func_var_shadowing() {
        let (_, func_def) = Stmt::new(
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
        let (_, func_call) = Stmt::new("📞 f x").unwrap();

        let mut env = Env::test();
        env.eval(&func_def).unwrap();

        env.store_binding("c".to_string(), Val::Number(0));
        env.store_binding("x".to_string(), Val::Number(8));
        let res = env.eval(&func_call);

        assert_eq!(res, Ok(Val::Number(1)));
        assert_eq!(env.get_binding("c"), Ok(Val::Number(1)));
        assert_eq!(env.get_binding("x"), Ok(Val::Number(1)));
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
