use crate::env::{Env, Eval};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub exprs: Vec<Expr>,
}

impl Block {
    pub fn explicit(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BLOCK_OPEN, s)?;

        Self::strong_implicit(s)
    }

    pub fn implicit(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = match utils::tag(kwords::BLOCK_OPEN, s) {
            Ok(sub) => sub,
            Err(_) => s,
        };

        Self::strong_implicit(s)
    }

    fn strong_implicit(s: &str) -> Result<(&str, Self), String> {
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
                },
                Err(_) => {
                    trailing_sep = false;
                    break
                },
            };

            let (new_s, _) = utils::extract_whitespace(s);
            s = new_s;
        }

        if trailing_sep {
            exprs.push(Expr::Literal(Val::Unit));
        }

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::BLOCK_CLOSE, s)?;

        Ok((s, Block { exprs }))
    }
}
impl Eval for Block {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let len = self.exprs.len();

        if len == 0 {
            Ok(Val::Unit)
        } else {
            for expr in &self.exprs[0..len - 1] {
                let intermediate = env.eval(expr)?;
                if let Val::Break(_) = &intermediate {
                    return Ok(intermediate);
                }
            }

            env.eval(&self.exprs[len - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_update::BindingUpdate, binding_usage::BindingUsage, Expr, Op};

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
        let expected = Block { exprs: vec![
            Expr::Literal(Val::Number(2)),
            Expr::Literal(Val::Unit),
        ]};

        assert_eq!(block_e, Ok(("", expected)));
    }

    #[test]
    fn parse_block_missing_token() {
        assert_eq!(Block::implicit("📦"), Err("expected 🧑‍🦲".to_string()));
        assert_eq!(Block::explicit("📦"), Err("expected 🧑‍🦲".to_string()));
        assert_eq!(Block::explicit("🧑‍🦲"), Err("expected 📦".to_string()));
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
            Expr::Literal(Val::Number(5)),
            Expr::Operation {
                lhs: Box::new(Expr::Literal(Val::Number(2))),
                rhs: Box::new(Expr::Literal(Val::Number(2))),
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
                    val: Expr::Literal(Val::Number(10)),
                    set: false,
                })),
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "b".to_string(),
                    val: Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    }),
                    set: false,
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
                        exprs: vec![Expr::BindingUsage(BindingUsage {
                            name: "a".into(),
                        })],
                    })),
                    rhs: Box::new(Expr::Block(Block {
                        exprs: vec![Expr::BindingUsage(BindingUsage {
                            name: "b".into(),
                        })],
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
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".into(),
                })],
            })),
            rhs: Box::new(Expr::Block(Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "b".into(),
                })],
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
}
