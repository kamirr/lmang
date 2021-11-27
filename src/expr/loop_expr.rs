use crate::env::{Env, Eval};
use crate::expr::block::Block;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Loop {
    body: Block,
}

impl Loop {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::LOOP, s)?;

        let (s, body) = Block::implicit(s)?;
        let loop_e = Loop { body };

        Ok((s, loop_e))
    }
}

impl Eval for Loop {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        Ok(loop {
            match env.eval(&self.body)? {
                Val::Break(inner_box) => break *inner_box,
                _ => continue,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{if_expr::If, BindingUpdate, BindingUsage, Break, Expr, Op};

    #[test]
    fn parse_loop_empty() {
        let parse = Loop::new("🔁 🧑‍🦲");
        let expected = Loop {
            body: Block { exprs: Vec::new() },
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_loop_break() {
        let parse = Loop::new("🔁 💔 🧑‍🦲🧑‍🦲");
        let expected = Loop {
            body: Block {
                exprs: vec![Expr::Break(Box::new(Break {
                    body: Block { exprs: Vec::new() },
                }))],
            },
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_factorial_loop() {
        let loop_e = Loop::new(
            "🔁
                ❓ 0-a
                    💔 fact 🧑‍🦲
                🧑‍🦲 💪

                👶 fact = fact * a 💪
                👶 a = a - 1
            🧑‍🦲",
        );

        let expected = Loop {
            body: Block {
                exprs: vec![
                    Expr::If(Box::new(If {
                        cond: Expr::Operation {
                            lhs: Box::new(Expr::Literal(Val::Number(0))),
                            rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            op: Op::Sub,
                        },
                        body: Block {
                            exprs: vec![Expr::Break(Box::new(Break {
                                body: Block {
                                    exprs: vec![Expr::BindingUsage(BindingUsage {
                                        name: "fact".to_string(),
                                    })],
                                },
                            }))],
                        },
                        elifs: Vec::new(),
                        body_else: None,
                    })),
                    Expr::BindingUpdate(Box::new(BindingUpdate {
                        name: "fact".to_string(),
                        val: Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "fact".to_string(),
                            })),
                            rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            op: Op::Mul,
                        },
                        set: false,
                    })),
                    Expr::BindingUpdate(Box::new(BindingUpdate {
                        name: "a".to_string(),
                        val: Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            rhs: Box::new(Expr::Literal(Val::Number(1))),
                            op: Op::Sub,
                        },
                        set: false,
                    })),
                ],
            },
        };

        assert_eq!(loop_e, Ok(("", expected)));
    }

    #[test]
    fn parse_factorial_complete() {
        let loop_e = Block::explicit(
            "📦
                👶 fact = 1 💪
                👶 a = 5 💪

                🔁
                    ❓ 0-a
                        💔 fact 🧑‍🦲
                    🧑‍🦲 💪

                    set fact = fact * a 💪
                    set a = a - 1
                🧑‍🦲
            🧑‍🦲",
        );

        let expected = Block {
            exprs: vec![
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "fact".to_string(),
                    val: Expr::Literal(Val::Number(1)),
                    set: false,
                })),
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Literal(Val::Number(5)),
                    set: false,
                })),
                Expr::Loop(Box::new(Loop {
                    body: Block {
                        exprs: vec![
                            Expr::If(Box::new(If {
                                cond: Expr::Operation {
                                    lhs: Box::new(Expr::Literal(Val::Number(0))),
                                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    op: Op::Sub,
                                },
                                body: Block {
                                    exprs: vec![Expr::Break(Box::new(Break {
                                        body: Block {
                                            exprs: vec![Expr::BindingUsage(BindingUsage {
                                                name: "fact".to_string(),
                                            })],
                                        },
                                    }))],
                                },
                                elifs: Vec::new(),
                                body_else: None,
                            })),
                            Expr::BindingUpdate(Box::new(BindingUpdate {
                                name: "fact".to_string(),
                                val: Expr::Operation {
                                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "fact".to_string(),
                                    })),
                                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    op: Op::Mul,
                                },
                                set: true,
                            })),
                            Expr::BindingUpdate(Box::new(BindingUpdate {
                                name: "a".to_string(),
                                val: Expr::Operation {
                                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    rhs: Box::new(Expr::Literal(Val::Number(1))),
                                    op: Op::Sub,
                                },
                                set: true,
                            })),
                        ],
                    },
                })),
            ],
        };

        assert_eq!(loop_e, Ok(("", expected)));
    }

    #[test]
    fn eval_loop_simple() {
        let (_, loop_e) = Loop::new("🔁 💔 2 🧑‍🦲 🧑‍🦲").unwrap();
        let expected = Val::Number(2);

        let mut env = Env::test();
        let res = env.eval(&loop_e);

        assert_eq!(res, Ok(expected));
    }

    #[test]
    fn eval_loop_factorial() {
        let (_, loop_e) = Block::explicit(
            "📦
                👶 fact = 1 💪

                🔁
                    ❓ a < 1
                        💔 fact 🧑‍🦲
                    🧑‍🦲 💪

                    👶 fact = fact * a 💪
                    👶 a = a - 1
                🧑‍🦲💪

                fact
            🧑‍🦲",
        )
        .unwrap();

        let mut env = Env::test();

        let cases = [(0, 1), (1, 1), (2, 2), (3, 6), (4, 24), (5, 120)];
        for case in cases {
            env.store_binding("a".to_string(), Val::Number(case.0));
            let result = env.eval(&loop_e);

            assert_eq!(result, Ok(Val::Number(case.1)));
        }
    }

    #[test]
    fn eval_loop_infinite() {
        let (_, loop_e) = Loop::new("🔁🧑‍🦲").unwrap();
        let mut env = Env::test();

        assert_eq!(env.eval(&loop_e), Err("timeout".to_string()))
    }
}
