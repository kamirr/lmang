use crate::env::{Env, Eval};
use crate::stmt::Stmt;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
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
        let mut stmts = Vec::new();

        while let Ok((new_s, stmt)) = Stmt::new(s) {
            stmts.push(stmt);

            let (new_s, _) = utils::extract_whitespace(new_s);
            s = new_s;

            s = match utils::tag(kwords::STMT_SEP, s) {
                Ok(new_s) => new_s,
                Err(_) => break,
            };

            let (new_s, _) = utils::extract_whitespace(s);
            s = new_s;
        }
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag(kwords::BLOCK_CLOSE, s)?;

        Ok((s, Block { stmts }))
    }
}
impl Eval for Block {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let len = self.stmts.len();

        if len == 0 {
            Ok(Val::Unit)
        } else {
            for stmt in &self.stmts[0..len - 1] {
                let intermediate = env.eval(stmt)?;
                if let Val::Break(_) = &intermediate {
                    return Ok(intermediate);
                }
            }

            env.eval(&self.stmts[len - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BindingUsage, Expr, Number, Op};
    use super::*;
    use crate::binding_update::BindingUpdate;

    #[test]
    fn parse_empty_block() {
        assert_eq!(
            Block::implicit("🧑‍🦲"),
            Ok(("", Block { stmts: Vec::new() }))
        );
        assert_eq!(
            Block::implicit("📦 🧑‍🦲"),
            Ok(("", Block { stmts: Vec::new() }))
        );
        assert_eq!(
            Block::explicit("📦🧑‍🦲"),
            Ok(("", Block { stmts: Vec::new() }))
        );
    }

    #[test]
    fn parse_blick_missing_token() {
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
                    stmts: vec![Stmt::Expr(Expr::Block(Block { stmts: Vec::new() }))]
                }
            ))
        );
        assert_eq!(
            Block::explicit("📦📦📦🧑‍🦲🧑‍🦲🧑‍🦲"),
            Ok((
                "",
                Block {
                    stmts: vec![Stmt::Expr(Expr::Block(Block {
                        stmts: vec![Stmt::Expr(Expr::Block(Block { stmts: Vec::new() }))],
                    }))]
                }
            ))
        );
    }

    #[test]
    fn parse_block_with_one_stmt() {
        let blocks = [Block::explicit("📦5🧑‍🦲"), Block::implicit("2*2🧑‍🦲")];
        let res_exprs = [
            Expr::Number(Number(5)),
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(2))),
                rhs: Box::new(Expr::Number(Number(2))),
                op: Op::Mul,
            },
        ];

        for (block, res_expr) in blocks.into_iter().zip(res_exprs.into_iter()) {
            assert_eq!(
                block,
                Ok((
                    "",
                    Block {
                        stmts: vec![Stmt::Expr(res_expr)],
                    },
                )),
            );
        }
    }

    #[test]
    fn parse_block_many_stmts() {
        let block = Block::explicit(
            "📦
            👶 a = 10💪
            👶 b = a 💪
            
            b
        🧑‍🦲",
        );

        let expected = Block {
            stmts: vec![
                Stmt::BindingUpdate(BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Number(Number(10)),
                    set: false,
                }),
                Stmt::BindingUpdate(BindingUpdate {
                    name: "b".to_string(),
                    val: Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    }),
                    set: false,
                }),
                Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "b".to_string(),
                })),
            ],
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn parse_block_nested_sum() {
        let block = Block::explicit("📦📦📦a🧑‍🦲 + 📦b🧑‍🦲🧑‍🦲🧑‍🦲");

        let expected = Block {
            stmts: vec![Stmt::Expr(Expr::Block(Block {
                stmts: vec![Stmt::Expr(Expr::Operation {
                    lhs: Box::new(Expr::Block(Block {
                        stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                            name: "a".into(),
                        }))],
                    })),
                    rhs: Box::new(Expr::Block(Block {
                        stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                            name: "b".into(),
                        }))],
                    })),
                    op: Op::Add,
                })],
            }))],
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn parse_block_sum() {
        let block = Stmt::new("📦a🧑‍🦲 + 📦b🧑‍🦲");

        let expected = Stmt::Expr(Expr::Operation {
            lhs: Box::new(Expr::Block(Block {
                stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "a".into(),
                }))],
            })),
            rhs: Box::new(Expr::Block(Block {
                stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "b".into(),
                }))],
            })),
            op: Op::Add,
        });

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
    fn eval_block_one_stmt() {
        let (_, block) = Block::implicit("📦44🧑‍🦲").unwrap();

        let mut env = Env::test();
        let value = env.eval(&block);

        assert_eq!(value, Ok(Val::Number(44)));
    }

    #[test]
    fn eval_block_many_stmts() {
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
