use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::{block::Block, Expr};
use crate::utils::{self, kwords};
use crate::val::Val;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct If {
    pub(crate) cond: Expr,
    pub(crate) body: Block,
    pub(crate) elifs: Vec<(Expr, Block)>,
    pub(crate) body_else: Option<Block>,
}

impl If {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::IF, s)?;

        let (s, _) = utils::extract_whitespace(s);
        let (s, cond) = Expr::new(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let (s, body) = Block::implicit(s)?;

        let mut s = s;
        let mut elifs = Vec::new();
        loop {
            let new_s = s;

            let (new_s, _) = utils::extract_whitespace(new_s);
            let new_s = match utils::tag(kwords::ELIF, new_s) {
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

        let (s, body_else) = || -> Result<(&str, Option<Block>), ParseError> {
            let old_s = s;

            let (s, _) = utils::extract_whitespace(s);
            let s = match utils::tag(kwords::ELSE, s) {
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
}

impl Eval for If {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        let cond_val = env.eval(&self.cond)?;
        if *cond_val.as_bool()? {
            env.eval(&self.body)
        } else {
            for (elif_cond, elif_body) in &self.elifs {
                if *env.eval(elif_cond)?.as_bool()? {
                    return env.eval(elif_body);
                }
            }
            match &self.body_else {
                Some(be) => env.eval(be),
                None => Ok(Cow::Owned(Val::Unit)),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Literal, Op};
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
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(0)))),
                        op: Op::Greater
                    },
                    body: Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(9)))]
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
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(0)))),
                        op: Op::Greater
                    },
                    body: Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(9)))]
                    },
                    elifs: Vec::new(),
                    body_else: Some(Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(0)))]
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
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(0)))),
                        op: Op::Greater
                    },
                    body: Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(10)))]
                    },
                    elifs: vec![(
                        Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string()
                            })),
                            rhs: Box::new(Expr::Literal(Literal(Val::Number(1)))),
                            op: Op::Greater
                        },
                        Block {
                            exprs: vec![Expr::Literal(Literal(Val::Number(20)))]
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
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(0)))),
                        op: Op::Greater
                    },
                    body: Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(10)))]
                    },
                    elifs: vec![(
                        Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string()
                            })),
                            rhs: Box::new(Expr::Literal(Literal(Val::Number(1)))),
                            op: Op::Greater
                        },
                        Block {
                            exprs: vec![Expr::Literal(Literal(Val::Number(20)))]
                        }
                    )],
                    body_else: Some(Block {
                        exprs: vec![Expr::Literal(Literal(Val::Number(30)))]
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
                👶 b = 2 💪
                👶 c = 3 💪

                b + c
            🧑‍🦲
        ",
        )
        .unwrap();

        let mut env = Env::test();

        env.store_binding("a".to_string(), Val::Number(3));
        let res = env.eval(&if_e);
        assert_eq!(res, Ok(Cow::Owned(Val::Number(5))));

        env.store_binding("a".to_string(), Val::Number(0));
        let res = env.eval(&if_e);
        assert_eq!(res, Ok(Cow::Owned(Val::Unit)));
    }

    #[test]
    fn eval_if_else() {
        let (_, if_e) = If::new(
            "
            ❓ a > 0
                👶 x = 2 💪

                x + a
            🧑‍🦲 😡
                👶 a = 1 💪

                a
            🧑‍🦲
        ",
        )
        .unwrap();

        let mut env = Env::test();

        env.store_binding("a".to_string(), Val::Number(3));
        let res = env.eval(&if_e);
        assert_eq!(res, Ok(Cow::Owned(Val::Number(5))));

        env.store_binding("a".to_string(), Val::Number(0));
        let res = env.eval(&if_e);
        assert_eq!(res, Ok(Cow::Borrowed(&Val::Number(1))));
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

        let mut env = Env::test();

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
            let res = env.eval(&if_e);
            if if_in > 0 {
                assert_eq!(res, Ok(Cow::Borrowed(&Val::Number(if_out))));
            } else {
                assert_eq!(res, Ok(Cow::Owned(Val::Number(if_out))));
            }
        }
    }
}
