use crate::env::Env;
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

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        Ok(loop {
            match self.body.eval(env)? {
                Val::Break(inner_box) => break *inner_box,
                _ => continue,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_update::BindingUpdate;
    use crate::expr::{if_expr::If, BindingUsage, Break, Expr, Number, Op};
    use crate::stmt::Stmt;

    #[test]
    fn parse_loop_empty() {
        let parse = Loop::new("🔁 🧑‍🦲");
        let expected = Loop {
            body: Block { stmts: Vec::new() },
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_loop_break() {
        let parse = Loop::new("🔁 💔 🧑‍🦲🧑‍🦲");
        let expected = Loop {
            body: Block {
                stmts: vec![Stmt::Expr(Expr::Break(Box::new(Break {
                    body: Block { stmts: Vec::new() },
                })))],
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
                stmts: vec![
                    Stmt::Expr(Expr::If(Box::new(If {
                        cond: Expr::Operation {
                            lhs: Box::new(Expr::Number(Number(0))),
                            rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            op: Op::Sub,
                        },
                        body: Block {
                            stmts: vec![Stmt::Expr(Expr::Break(Box::new(Break {
                                body: Block {
                                    stmts: vec![Stmt::Expr(Expr::BindingUsage(BindingUsage {
                                        name: "fact".to_string(),
                                    }))],
                                },
                            })))],
                        },
                        elifs: Vec::new(),
                        body_else: None,
                    }))),
                    Stmt::BindingUpdate(BindingUpdate {
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
                    }),
                    Stmt::BindingUpdate(BindingUpdate {
                        name: "a".to_string(),
                        val: Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            rhs: Box::new(Expr::Number(Number(1))),
                            op: Op::Sub,
                        },
                        set: false,
                    }),
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
            stmts: vec![
                Stmt::BindingUpdate(BindingUpdate {
                    name: "fact".to_string(),
                    val: Expr::Number(Number(1)),
                    set: false,
                }),
                Stmt::BindingUpdate(BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Number(Number(5)),
                    set: false,
                }),
                Stmt::Expr(Expr::Loop(Box::new(Loop {
                    body: Block {
                        stmts: vec![
                            Stmt::Expr(Expr::If(Box::new(If {
                                cond: Expr::Operation {
                                    lhs: Box::new(Expr::Number(Number(0))),
                                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    op: Op::Sub,
                                },
                                body: Block {
                                    stmts: vec![Stmt::Expr(Expr::Break(Box::new(Break {
                                        body: Block {
                                            stmts: vec![Stmt::Expr(Expr::BindingUsage(
                                                BindingUsage {
                                                    name: "fact".to_string(),
                                                },
                                            ))],
                                        },
                                    })))],
                                },
                                elifs: Vec::new(),
                                body_else: None,
                            }))),
                            Stmt::BindingUpdate(BindingUpdate {
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
                            }),
                            Stmt::BindingUpdate(BindingUpdate {
                                name: "a".to_string(),
                                val: Expr::Operation {
                                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    rhs: Box::new(Expr::Number(Number(1))),
                                    op: Op::Sub,
                                },
                                set: true,
                            }),
                        ],
                    },
                }))),
            ],
        };

        assert_eq!(loop_e, Ok(("", expected)));
    }

    #[test]
    fn eval_loop_simple() {
        let (_, loop_e) = Loop::new("🔁 💔 2 🧑‍🦲 🧑‍🦲").unwrap();
        let expected = Val::Number(2);

        let mut env = Env::new();
        let res = loop_e.eval(&mut env);

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

        let mut env = Env::new();

        let cases = [(0, 1), (1, 1), (2, 2), (3, 6), (4, 24), (5, 120)];
        for case in cases {
            env.store_binding("a".to_string(), Val::Number(case.0));
            let result = loop_e.eval(&mut env);

            assert_eq!(result, Ok(Val::Number(case.1)));
        }
    }
}
