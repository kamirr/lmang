use crate::env::Env;
use crate::stmt::Stmt;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn explicit(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("📦", s)?;

        Self::strong_implicit(s)
    }

    pub fn implicit(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = match utils::tag("📦", s) {
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

            s = match utils::tag("💪", s) {
                Ok(new_s) => new_s,
                Err(_) => break,
            };

            let (new_s, _) = utils::extract_whitespace(s);
            s = new_s;
        }
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag("🧑‍🦲", s)?;

        Ok((s, Block { stmts }))
    }

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let len = self.stmts.len();

        if len == 0 {
            Ok(Val::Unit)
        } else {
            for stmt in &self.stmts[0..len - 1] {
                let intermediate = stmt.eval(env)?;
                if let Val::Break(_) = &intermediate {
                    return Ok(intermediate);
                }
            }

            self.stmts[len - 1].eval(env)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{BindingUsage, Expr, Number, Op};
    use super::*;
    use crate::binding_def::BindingDef;

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
                Stmt::BindingDef(BindingDef {
                    name: "a".to_string(),
                    val: Expr::Number(Number(10)),
                }),
                Stmt::BindingDef(BindingDef {
                    name: "b".to_string(),
                    val: Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    }),
                }),
                Stmt::Expr(Expr::BindingUsage(BindingUsage {
                    name: "b".to_string(),
                })),
            ],
        };

        assert_eq!(block, Ok(("", expected)));
    }

    #[test]
    fn eval_block_empty() {
        let (_, block) = Block::implicit("🧑‍🦲").unwrap();

        let mut env = Env::new();
        let value = block.eval(&mut env);

        assert_eq!(value, Ok(Val::Unit));
    }

    #[test]
    fn eval_block_one_stmt() {
        let (_, block) = Block::implicit("📦44🧑‍🦲").unwrap();

        let mut env = Env::new();
        let value = block.eval(&mut env);

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

        let mut env = Env::new();
        let value = block.eval(&mut env);

        assert_eq!(value, Ok(Val::Number(12)));
    }
}
