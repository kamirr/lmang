use crate::env::Env;
use crate::expr::block::Block;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub struct Loop {
    body: Block,
}

impl Loop {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("loop", s)?;

        let (s, body) = Block::implicit(s)?;
        let loop_e = Loop { body };

        Ok((s, loop_e))
    }

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_def::BindingDef;
    use crate::expr::{if_expr::If, BindingUsage, Break, Expr, Number, Op};
    use crate::stmt::Stmt;

    #[test]
    fn parse_loop_empty() {
        let parse = Loop::new("loop 🧑‍🦲");
        let expected = Loop {
            body: Block { stmts: Vec::new() },
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_loop_break() {
        let parse = Loop::new("loop break 🧑‍🦲🧑‍🦲");
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
            "
            loop
                ❓ 0-a
                    break fact 🧑‍🦲
                🧑‍🦲 💪

                let fact = fact * a 💪
                let a = a - 1
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
                    Stmt::BindingDef(BindingDef {
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
                    }),
                    Stmt::BindingDef(BindingDef {
                        name: "a".to_string(),
                        val: Expr::Operation {
                            lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })),
                            rhs: Box::new(Expr::Number(Number(1))),
                            op: Op::Sub,
                        },
                    }),
                ],
            },
        };

        assert_eq!(loop_e, Ok(("", expected)));
    }

    #[test]
    fn parse_factorial_complete() {
        let loop_e = Block::explicit(
            "
📦
    let fact = 1 💪
    let a = 5 💪

    loop
        ❓ 0-a
            break fact 🧑‍🦲
        🧑‍🦲 💪

        let fact = fact * a 💪
        let a = a - 1
    🧑‍🦲
🧑‍🦲",
        );

        let expected = Block {
            stmts: vec![
                Stmt::BindingDef(BindingDef {
                    name: "fact".to_string(),
                    val: Expr::Number(Number(1)),
                }),
                Stmt::BindingDef(BindingDef {
                    name: "a".to_string(),
                    val: Expr::Number(Number(5)),
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
                            Stmt::BindingDef(BindingDef {
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
                            }),
                            Stmt::BindingDef(BindingDef {
                                name: "a".to_string(),
                                val: Expr::Operation {
                                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                                        name: "a".to_string(),
                                    })),
                                    rhs: Box::new(Expr::Number(Number(1))),
                                    op: Op::Sub,
                                },
                            }),
                        ],
                    },
                }))),
            ],
        };

        assert_eq!(loop_e, Ok(("", expected)));
    }
}
