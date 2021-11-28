use crate::env::{Env, Eval};
use crate::expr::block::Block;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub body: Block,
}

impl Break {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BREAK, s)?;

        let (s, body) = Block::implicit(s)?;

        Ok((s, Break { body }))
    }
}

impl Eval for Break {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        Ok(Val::Break(Box::new(env.eval(&self.body)?)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Expr, Number, Op};
    use crate::stmt::Stmt;

    #[test]
    fn parse_empty_break() {
        let cases = ["💔 🧑‍🦲", "💔 📦 🧑‍🦲"];

        for case in cases {
            let expected = Break {
                body: Block { stmts: Vec::new() },
            };
            assert_eq!(Break::new(case), Ok(("", expected)));
        }
    }

    #[test]
    fn parse_valued_break() {
        let cases = [
            (
                "💔 🚀 🧑‍🦲",
                Break {
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                            name: "🚀".to_string(),
                        }))],
                    },
                },
            ),
            (
                "💔 📦 a / 2 🧑‍🦲",
                Break {
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            rhs: Box::new(Expr::Number(Number(2))),
                            op: Op::Div,
                        })],
                    },
                },
            ),
        ];

        for case in cases {
            assert_eq!(Break::new(case.0), Ok(("", case.1)));
        }
    }

    #[test]
    fn eval_break() {
        let input = "
        👶 👩‍🚀 = 📦
                ❓ 🅱️ > 0
                    💔 🅱️ 🧑‍🦲
                🧑‍🦲 💪

                0 - 🅱️
            🧑‍🦲";

        let mut env = Env::test();
        let (_, stmt) = Stmt::new(input).unwrap();

        for k in -10..10 {
            env.store_binding("🅱️".to_string(), Val::Number(k));
            let _ = env.eval(&stmt).unwrap();

            let expected = if k > 0 {
                Val::Break(Box::new(Val::Number(k)))
            } else {
                Val::Number(-k)
            };

            assert_eq!(env.get_binding("👩‍🚀"), Ok(expected));
        }
    }
}
