use crate::env::{Env, Eval};
use crate::error::ParseError;
use crate::expr::block::{Block, FormatImplicit};
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Break {
    pub(crate) body: Block,
}

impl Break {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BREAK, s)?;

        let (s, body) = Block::implicit(s)?;

        Ok((s, Break { body }))
    }
}

impl Eval for Break {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        Ok(Val::Break(Box::new(env.eval(&self.body)?)))
    }
}

impl crate::expr::Format for Break {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        writeln!(w, "{}", kwords::BREAK)?;
        FormatImplicit(&self.body).format(w, depth)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Expr, Literal, Op};

    #[test]
    fn parse_empty_break() {
        let cases = ["ð ð§âðĶē", "ð ðĶ ð§âðĶē"];

        for case in cases {
            let expected = Break {
                body: Block { exprs: Vec::new() },
            };
            assert_eq!(Break::new(case), Ok(("", expected)));
        }
    }

    #[test]
    fn parse_valued_break() {
        let cases = [
            (
                "ð ð ð§âðĶē",
                Break {
                    body: Block {
                        exprs: vec![Expr::BindingUsage(BindingUsage {
                            name: "ð".to_string(),
                        })],
                    },
                },
            ),
            (
                "ð ðĶ a / 2 ð§âðĶē",
                Break {
                    body: Block {
                        exprs: vec![Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
                            op: Op::Div,
                        }],
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
        let input = "ð ð§âðĶē";

        let mut env = Env::test();
        let (_, expr) = Expr::new(input).unwrap();

        let result = env.eval(&expr);
        let expected = Ok(Val::Break(Box::new(Val::Unit)));

        assert_eq!(result, expected);
    }

    #[test]
    fn eval_break_in_let() {
        let input = "ðķ ðĐâð = ðĶ ð 2 ð§âðĶē ð§âðĶē";

        let mut env = Env::test();
        let (_, expr) = Expr::new(input).unwrap();

        let result = env.eval(&expr);
        let expected = Ok(Val::Break(Box::new(Val::Number(2))));

        assert_eq!(result, expected);
    }

    #[test]
    fn eval_break_in_loop() {
        let input = "ð
                ðķ _ = ðĶ
                    ð ð§âðĶē
                ð§âðĶē
            ð§âðĶē";

        let mut env = Env::test();
        let (_, expr) = Expr::new(input).unwrap();

        let result = env.eval(&expr);
        assert_eq!(result, Ok(Val::Unit));
    }

    #[test]
    fn format() {
        let (_, break_e) = Break::new("ððĶa/2ð§âðĶē").unwrap();
        assert_eq!(
            format!("{}", crate::expr::Display(&break_e)),
            "ð\n    a / 2\nð§âðĶē"
        );
    }
}
