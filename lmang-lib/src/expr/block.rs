use crate::env::{Env, Eval};
use crate::error::ParseError;
use crate::expr::{Expr, Literal};
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub(crate) exprs: Vec<Expr>,
}

impl Block {
    pub(crate) fn explicit(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BLOCK_OPEN, s)?;

        Self::strong_implicit(s)
    }

    pub(crate) fn implicit(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = match utils::tag(kwords::BLOCK_OPEN, s) {
            Ok(sub) => sub,
            Err(_) => s,
        };

        Self::strong_implicit(s)
    }

    fn strong_implicit(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);

        let mut s = s;
        let mut exprs = Vec::new();
        let mut trailing_sep = false;

        while let Ok((new_s, expr)) = Expr::new(s) {
            exprs.push(expr);

            let (new_s, _) = utils::extract_whitespace(new_s);
            s = new_s;

            s = match utils::tag(kwords::EXPR_SEP, s) {
                Ok(new_s) => {
                    trailing_sep = true;
                    new_s
                }
                Err(_) => {
                    trailing_sep = false;
                    break;
                }
            };

            let (new_s, _) = utils::extract_whitespace(s);
            s = new_s;
        }

        if trailing_sep {
            exprs.push(Expr::Literal(Literal(Val::Unit)));
        }

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BLOCK_CLOSE, s)?;

        Ok((s, Block { exprs }))
    }
}
impl Eval for Block {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        let len = self.exprs.len();

        if len == 0 {
            Ok(Val::Unit)
        } else {
            env.push();

            for expr in &self.exprs[0..len - 1] {
                let intermediate = match env.eval(expr) {
                    Ok(v) => v,
                    Err(e) => {
                        env.pop();
                        return Err(e);
                    }
                };
                if let Val::Break(_) = &intermediate {
                    env.pop();
                    return Ok(intermediate);
                }
            }

            let result = env.eval(&self.exprs[len - 1]);
            env.pop();

            result
        }
    }
}

pub struct FormatImplicit<'a>(pub &'a Block);

impl<'a> crate::expr::Format for FormatImplicit<'a> {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        let (exprs, trailing_sep) = if let Some(e) = self.0.exprs.last() {
            if e == &Expr::Literal(Literal(Val::Unit)) {
                (&self.0.exprs[0..self.0.exprs.len() - 1], true)
            } else {
                (&self.0.exprs[..], false)
            }
        } else {
            (&self.0.exprs[..], false)
        };

        for (i, e) in exprs.iter().enumerate() {
            Self::indent(w, depth + 1)?;
            e.format(w, depth + 1)?;
            if i != exprs.len() - 1 || trailing_sep {
                writeln!(w, " {}", kwords::EXPR_SEP)?;
            } else {
                writeln!(w, "")?;
            }
        }
        Self::indent(w, depth)?;
        write!(w, "{}", kwords::BLOCK_CLOSE)?;

        Ok(())
    }
}

pub struct FormatExplicit<'a>(pub &'a Block);

impl<'a> crate::expr::Format for FormatExplicit<'a> {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        writeln!(w, "{}", kwords::BLOCK_OPEN)?;
        FormatImplicit(self.0).format(w, depth)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_update::Mode, BindingUpdate, BindingUsage, Op};

    #[test]
    fn parse_empty_block() {
        assert_eq!(
            Block::implicit("🧑‍🦲"),
            Ok(("", Block { exprs: Vec::new() }))
        );
        assert_eq!(
            Block::implicit("📦 🧑‍🦲"),
            Ok(("", Block { exprs: Vec::new() }))
        );
        assert_eq!(
            Block::explicit("📦🧑‍🦲"),
            Ok(("", Block { exprs: Vec::new() }))
        );
    }

    #[test]
    fn parse_block_tailing_sep() {
        let block_e = Block::explicit("📦 2 💪 🧑‍🦲");
        let expected = Block {
            exprs: vec![
                Expr::Literal(Literal(Val::Number(2))),
                Expr::Literal(Literal(Val::Unit)),
            ],
        };

        assert_eq!(block_e, Ok(("", expected)));
    }

    #[test]
    fn parse_block_missing_token() {
        assert_eq!(
            Block::implicit("📦"),
            Err(ParseError::ExpectedTag("🧑‍🦲".into()))
        );
        assert_eq!(
            Block::explicit("📦"),
            Err(ParseError::ExpectedTag("🧑‍🦲".into()))
        );
        assert_eq!(
            Block::explicit("🧑‍🦲"),
            Err(ParseError::ExpectedTag("📦".into()))
        );
    }

    #[test]
    fn parse_block_nexted() {
        assert_eq!(
            Block::explicit("📦📦🧑‍🦲🧑‍🦲"),
            Ok((
                "",
                Block {
                    exprs: vec![Expr::Block(Block { exprs: Vec::new() })]
                }
            ))
        );
        assert_eq!(
            Block::explicit("📦📦📦🧑‍🦲🧑‍🦲🧑‍🦲"),
            Ok((
                "",
                Block {
                    exprs: vec![Expr::Block(Block {
                        exprs: vec![Expr::Block(Block { exprs: Vec::new() })],
                    })]
                }
            ))
        );
    }

    #[test]
    fn parse_block_with_one_expr() {
        let blocks = [Block::explicit("📦5🧑‍🦲"), Block::implicit("2*2🧑‍🦲")];
        let res_exprs = [
            Expr::Literal(Literal(Val::Number(5))),
            Expr::Operation {
                lhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
                op: Op::Mul,
            },
        ];

        for (block, res_expr) in blocks.into_iter().zip(res_exprs.into_iter()) {
            assert_eq!(
                block,
                Ok((
                    "",
                    Block {
                        exprs: vec![res_expr],
                    },
                )),
            );
        }
    }

    #[test]
    fn parse_block_many_exprs() {
        let block = Block::explicit(
            "📦
            👶 a = 10💪
            👶 b = a 💪
            
            b
        🧑‍🦲",
        );

        let expected = Block {
            exprs: vec![
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Literal(Literal(Val::Number(10))),
                    mode: Mode::CreateLocal,
                })),
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "b".to_string(),
                    val: Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    }),
                    mode: Mode::CreateLocal,
                })),
                Expr::BindingUsage(BindingUsage {
                    name: "b".to_string(),
                }),
            ],
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn parse_block_nested_sum() {
        let block = Block::explicit("📦📦📦a🧑‍🦲 + 📦b🧑‍🦲🧑‍🦲🧑‍🦲");

        let expected = Block {
            exprs: vec![Expr::Block(Block {
                exprs: vec![Expr::Operation {
                    lhs: Box::new(Expr::Block(Block {
                        exprs: vec![Expr::BindingUsage(BindingUsage { name: "a".into() })],
                    })),
                    rhs: Box::new(Expr::Block(Block {
                        exprs: vec![Expr::BindingUsage(BindingUsage { name: "b".into() })],
                    })),
                    op: Op::Add,
                }],
            })],
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn parse_block_sum() {
        let block = Expr::new("📦a🧑‍🦲 + 📦b🧑‍🦲");

        let expected = Expr::Operation {
            lhs: Box::new(Expr::Block(Block {
                exprs: vec![Expr::BindingUsage(BindingUsage { name: "a".into() })],
            })),
            rhs: Box::new(Expr::Block(Block {
                exprs: vec![Expr::BindingUsage(BindingUsage { name: "b".into() })],
            })),
            op: Op::Add,
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn eval_block_empty() {
        let (_, block) = Block::implicit("🧑‍🦲").unwrap();

        let mut env = Env::test();
        let value = env.eval(&block);

        assert_eq!(value, Ok(Val::Unit));
    }

    #[test]
    fn eval_block_one_expr() {
        let (_, block) = Block::implicit("📦44🧑‍🦲").unwrap();

        let mut env = Env::test();
        let value = env.eval(&block);

        assert_eq!(value, Ok(Val::Number(44)));
    }

    #[test]
    fn eval_block_many_exprs() {
        let (_, block) = Block::implicit(
            "
            👶 a = 2 * 2💪
            👶 b = a * 2💪

            b + a
        🧑‍🦲",
        )
        .unwrap();

        let mut env = Env::test();
        let value = env.eval(&block);

        assert_eq!(value, Ok(Val::Number(12)));
    }

    #[test]
    fn format_implicit() {
        let (_, block) = Block::implicit("👶a=2*2💪👶b=a*2💪b+a🧑‍🦲").unwrap();
        let expected = "    👶 a = 2 * 2 💪\n    👶 b = a * 2 💪\n    b + a\n🧑‍🦲";

        assert_eq!(
            format!("{}", crate::expr::Display(&FormatImplicit(&block))),
            expected
        );
    }

    #[test]
    fn format_implicit_trailing_sep() {
        let (_, block) = Block::implicit("👶a=2*2💪👶b=a*2💪b+a💪🧑‍🦲").unwrap();
        let expected = "    👶 a = 2 * 2 💪\n    👶 b = a * 2 💪\n    b + a 💪\n🧑‍🦲";

        assert_eq!(
            format!("{}", crate::expr::Display(&FormatImplicit(&block))),
            expected
        );
    }

    #[test]
    fn format_explicit() {
        let (_, block) = Block::implicit("👶a=2*2💪👶b=a*2💪b+a🧑‍🦲").unwrap();
        let expected = "📦\n    👶 a = 2 * 2 💪\n    👶 b = a * 2 💪\n    b + a\n🧑‍🦲";

        assert_eq!(
            format!("{}", crate::expr::Display(&FormatExplicit(&block))),
            expected
        );
    }

    #[test]
    fn format_explicit_trailing_sep() {
        let (_, block) = Block::implicit("👶a=2*2💪👶b=a*2💪b+a💪🧑‍🦲").unwrap();
        let expected = "📦\n    👶 a = 2 * 2 💪\n    👶 b = a * 2 💪\n    b + a 💪\n🧑‍🦲";

        assert_eq!(
            format!("{}", crate::expr::Display(&FormatExplicit(&block))),
            expected
        );
    }
}
