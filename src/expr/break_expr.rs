use crate::env::Env;
use crate::expr::block::Block;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub struct Break {
    pub body: Block,
}

impl Break {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("💔", s)?;

        let (s, body) = Block::implicit(s)?;

        Ok((s, Break { body }))
    }

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        Ok(Val::Break(Box::new(self.body.eval(env)?)))
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
            let 👩‍🚀 = 📦
                ❓ 🅱️ > 0
                    💔 🅱️ 🧑‍🦲
                🧑‍🦲 💪

                0 - 🅱️
            🧑‍🦲";

        let mut env = Env::new();
        let (_, stmt) = Stmt::new(input).unwrap();

        for k in -10..10 {
            env.store_binding("🅱️".to_string(), Val::Number(k));
            let _ = stmt.eval(&mut env).unwrap();

            let expected = if k > 0 {
                Val::Break(Box::new(Val::Number(k)))
            } else {
                Val::Number(-k)
            };

            assert_eq!(env.get_binding("👩‍🚀"), Ok(expected));
        }
    }
}
