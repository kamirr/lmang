use crate::env::Env;
use crate::expr::{block::Block, Expr};
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub struct If {
    pub cond: Expr,
    pub body: Block,
    pub elifs: Vec<(Expr, Block)>,
    pub body_else: Option<Block>,
}

impl If {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("❓", s)?;

        let (s, _) = utils::extract_whitespace(s);
        let (s, cond) = Expr::new(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let (s, body) = Block::implicit(s)?;

        let mut s = s;
        let mut elifs = Vec::new();
        loop {
            let new_s = s;

            let (new_s, _) = utils::extract_whitespace(new_s);
            let new_s = match utils::tag("😠", new_s) {
                Ok(new_s) => new_s,
                Err(_) => break,
            };

            let (new_s, _) = utils::extract_whitespace(new_s);
            let (new_s, cond) = Expr::new(new_s)?;

            let (new_s, _) = utils::extract_whitespace(new_s);
            let (new_s, body) = Block::implicit(new_s)?;

            s = new_s;
            elifs.push((cond, body));
        }

        let (s, body_else) = || -> Result<(&str, Option<Block>), String> {
            let old_s = s;

            let (s, _) = utils::extract_whitespace(s);
            let s = match utils::tag("😡", s) {
                Ok(s) => s,
                Err(_) => return Ok((old_s, None)),
            };

            let (s, _) = utils::extract_whitespace(s);
            let (s, body_else) = Block::implicit(s)?;

            Ok((s, Some(body_else)))
        }()?;

        Ok((
            s,
            Self {
                cond,
                body,
                elifs,
                body_else,
            },
        ))
    }

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let cond_val = self.cond.eval(env)?;
        if cond_val.as_bool()? {
            self.body.eval(env)
        } else {
            for (elif_cond, elif_body) in &self.elifs {
                if elif_cond.eval(env)?.as_bool()? {
                    return elif_body.eval(env);
                }
            }
            match &self.body_else {
                Some(be) => be.eval(env),
                None => Ok(Val::Unit),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_usage::BindingUsage, Number, Op};
    use crate::stmt::Stmt;
    use crate::val::Val;

    #[test]
    fn parse_if() {
        assert_eq!(
            If::new(
                "
                ❓ a > 0
                    9
                🧑‍🦲"
            ),
            Ok((
                "",
                If {
                    cond: Expr::Operation {
                        lhs: Box::new(Expr::BindingUsage(BindingUsage {
                            name: "a".to_string()
                        })),
                        rhs: Box::new(Expr::Number(Number(0))),
                        op: Op::Greater
                    },
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(9)))]
                    },
                    elifs: Vec::new(),
                    body_else: None,
                },
            )),
        );
    }

    #[test]
    fn parse_if_else() {
        assert_eq!(
            If::new(
                "
                ❓ a > 0
                    9
                🧑‍🦲 😡
                    0
                🧑‍🦲"
            ),
            Ok((
                "",
                If {
                    cond: Expr::Operation {
                        lhs: Box::new(Expr::BindingUsage(BindingUsage {
                            name: "a".to_string()
                        })),
                        rhs: Box::new(Expr::Number(Number(0))),
                        op: Op::Greater
                    },
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(9)))]
                    },
                    elifs: Vec::new(),
                    body_else: Some(Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(0)))]
                    })
                },
            )),
        );
    }

    #[test]
    fn parse_if_elif() {
        assert_eq!(
            If::new(
                "
                ❓ a > 0
                    10
                🧑‍🦲 😠 a > 1
                    20
                🧑‍🦲"
            ),
            Ok((
                "",
                If {
                    cond: Expr::Operation {
                        lhs: Box::new(Expr::BindingUsage(BindingUsage {
                            name: "a".to_string()
                        })),
                        rhs: Box::new(Expr::Number(Number(0))),
                        op: Op::Greater
                    },
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(10)))]
                    },
                    elifs: vec![(
                        Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string()
                            })),
                            rhs: Box::new(Expr::Number(Number(1))),
                            op: Op::Greater
                        },
                        Block {
                            stmts: vec![Stmt::Expr(Expr::Number(Number(20)))]
                        }
                    )],
                    body_else: None,
                },
            )),
        );
    }

    #[test]
    fn parse_if_elif_else() {
        assert_eq!(
            If::new(
                "
                ❓ a > 0
                    10
                🧑‍🦲 😠 a > 1
                    20
                🧑‍🦲 😡
                    30
                🧑‍🦲"
            ),
            Ok((
                "",
                If {
                    cond: Expr::Operation {
                        lhs: Box::new(Expr::BindingUsage(BindingUsage {
                            name: "a".to_string()
                        })),
                        rhs: Box::new(Expr::Number(Number(0))),
                        op: Op::Greater
                    },
                    body: Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(10)))]
                    },
                    elifs: vec![(
                        Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string()
                            })),
                            rhs: Box::new(Expr::Number(Number(1))),
                            op: Op::Greater
                        },
                        Block {
                            stmts: vec![Stmt::Expr(Expr::Number(Number(20)))]
                        }
                    )],
                    body_else: Some(Block {
                        stmts: vec![Stmt::Expr(Expr::Number(Number(30)))]
                    }),
                },
            )),
        );
    }

    #[test]
    fn eval_if() {
        let (_, if_e) = If::new(
            "
            ❓ a > 0
                let b = 2 💪
                let c = 3 💪

                b + c
            🧑‍🦲
        ",
        )
        .unwrap();

        let mut env = Env::new();

        env.store_binding("a".to_string(), Val::Number(3));
        let res = if_e.eval(&mut env);
        assert_eq!(res, Ok(Val::Number(5)));

        env.store_binding("a".to_string(), Val::Number(0));
        let res = if_e.eval(&mut env);
        assert_eq!(res, Ok(Val::Unit));
    }

    #[test]
    fn eval_if_else() {
        let (_, if_e) = If::new(
            "
            ❓ a > 0
                let x = 2 💪

                x + a
            🧑‍🦲 😡
                let a = 1 💪

                a
            🧑‍🦲
        ",
        )
        .unwrap();

        let mut env = Env::new();

        env.store_binding("a".to_string(), Val::Number(3));
        let res = if_e.eval(&mut env);
        assert_eq!(res, Ok(Val::Number(5)));

        env.store_binding("a".to_string(), Val::Number(0));
        let res = if_e.eval(&mut env);
        assert_eq!(res, Ok(Val::Number(1)));
    }

    #[test]
    fn eval_if_elif_x3_else() {
        let (_, if_e) = If::new(
            "
            ❓ a > 0
                a
            🧑‍🦲 😠 a > 0-1
                0
            🧑‍🦲 😠 a > 0-2
                0-1
            🧑‍🦲 😠 a > 0-3
                0-2
            🧑‍🦲 😡
                0-999
            🧑‍🦲
        ",
        )
        .unwrap();

        let mut env = Env::new();

        let results = [
            (100, 100),
            (3, 3),
            (0, 0),
            (-1, -1),
            (-2, -2),
            (-3, -999),
            (-20, -999),
        ];

        for (if_in, if_out) in results {
            env.store_binding("a".to_string(), Val::Number(if_in));
            let res = if_e.eval(&mut env);
            assert_eq!(res, Ok(Val::Number(if_out)));
        }
    }
}
