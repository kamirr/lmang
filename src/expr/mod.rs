pub mod binding_usage;
pub mod block;
pub mod break_expr;
pub mod if_expr;
pub mod loop_expr;

use crate::env::Env;
use crate::utils;
use crate::val::Val;
use binding_usage::BindingUsage;
use block::Block;
use break_expr::Break;
use if_expr::If;
use loop_expr::Loop;

#[derive(Debug, PartialEq)]
pub struct Number(pub i32);

impl Number {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, number) = utils::extract_digits(s)?;
        Ok((s, Self(number.parse().unwrap())))
    }
}

#[derive(Debug, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Greater,
    GreaterEq,
    Eq,
    LessEq,
    Less,
}

impl Op {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        utils::tag("+", s)
            .map(|s| (s, Self::Add))
            .or_else(|_| utils::tag("-", s).map(|s| (s, Self::Sub)))
            .or_else(|_| utils::tag("*", s).map(|s| (s, Self::Mul)))
            .or_else(|_| utils::tag("/", s).map(|s| (s, Self::Div)))
            .or_else(|_| utils::tag(">=", s).map(|s| (s, Self::GreaterEq)))
            .or_else(|_| utils::tag(">", s).map(|s| (s, Self::Greater)))
            .or_else(|_| utils::tag("<=", s).map(|s| (s, Self::LessEq)))
            .or_else(|_| utils::tag("<", s).map(|s| (s, Self::Less)))
            .or_else(|_| utils::tag("==", s).map(|s| (s, Self::Eq)))
    }
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Operation {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Op,
    },
    Number(Number),
    BindingUsage(BindingUsage),
    Block(Block),
    If(Box<If>),
    Break(Box<Break>),
    Loop(Box<Loop>),
}

impl Expr {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);

        Self::new_operation(s)
            .or_else(|_| Self::new_number(s))
            .or_else(|_| {
                BindingUsage::new(s)
                    .map(|(s, binding_usage)| (s, Self::BindingUsage(binding_usage)))
            })
            .or_else(|_| Block::explicit(s).map(|(s, block)| (s, Self::Block(block))))
            .or_else(|_| If::new(s).map(|(s, if_e)| (s, Self::If(Box::new(if_e)))))
            .or_else(|_| Break::new(s).map(|(s, break_e)| (s, Self::Break(Box::new(break_e)))))
            .or_else(|_| Loop::new(s).map(|(s, loop_e)| (s, Self::Loop(Box::new(loop_e)))))
    }

    fn new_operation(s: &str) -> Result<(&str, Self), String> {
        let mut op_b_idx = 0;
        let mut op_c_idx = 0;

        loop {
            let sub = &s[op_b_idx..];
            if Op::new(sub).is_ok() {
                break;
            }

            for expr_breaker in ["💪", "📦"] {
                if sub.starts_with(expr_breaker) {
                    break;
                }
            }

            let c = s
                .chars()
                .skip(op_c_idx)
                .next()
                .ok_or("unexpected eof".to_string())?;

            op_b_idx += c.len_utf8();
            op_c_idx += 1;
        }

        let (sub, lhs) = Expr::new(&s[0..op_b_idx])?;
        let (sub, _) = utils::extract_whitespace(sub);

        if sub.len() > 0 {
            return Err("malformed operation".to_string());
        }

        let s = &s[op_b_idx..];

        let (s, op) = Op::new(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let (s, rhs) = Expr::new(s)?;

        Ok((
            s,
            Self::Operation {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                op,
            },
        ))
    }

    fn new_number(s: &str) -> Result<(&str, Self), String> {
        Number::new(s).map(|(s, number)| (s, Self::Number(number)))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        match self {
            Self::Number(Number(n)) => Ok(Val::Number(*n)),
            Self::Operation { lhs, rhs, op } => {
                let lhs = lhs.eval(env)?;
                let rhs = rhs.eval(env)?;

                match op {
                    Op::Add => lhs + rhs,
                    Op::Sub => lhs - rhs,
                    Op::Mul => lhs * rhs,
                    Op::Div => lhs / rhs,
                    Op::Greater => lhs.try_gt(&rhs),
                    Op::GreaterEq => lhs.try_ge(&rhs),
                    Op::Eq => lhs.try_eq(&rhs),
                    Op::LessEq => lhs.try_le(&rhs),
                    Op::Less => lhs.try_lt(&rhs),
                }
            }
            Self::BindingUsage(bu) => bu.eval(env),
            Self::Block(block) => block.eval(env),
            Self::If(if_e) => if_e.eval(env),
            Self::Break(break_e) => break_e.eval(env),
            Self::Loop(loop_e) => loop_e.eval(env),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stmt::Stmt;

    #[test]
    fn parse_number() {
        assert_eq!(Number::new("123"), Ok(("", Number(123))));
    }

    #[test]
    fn parse_add_op() {
        assert_eq!(Op::new("+"), Ok(("", Op::Add)));
    }

    #[test]
    fn parse_sub_op() {
        assert_eq!(Op::new("-"), Ok(("", Op::Sub)));
    }

    #[test]
    fn parse_mul_op() {
        assert_eq!(Op::new("*"), Ok(("", Op::Mul)));
    }

    #[test]
    fn parse_div_op() {
        assert_eq!(Op::new("/"), Ok(("", Op::Div)));
    }

    #[test]
    fn parse_one_plus_two() {
        assert_eq!(
            Expr::new("1+2"),
            Ok((
                "",
                Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(1))),
                    rhs: Box::new(Expr::Number(Number(2))),
                    op: Op::Add,
                },
            )),
        );
    }

    #[test]
    fn parse_expr_with_whitespace() {
        assert_eq!(
            Expr::new("2 * 2"),
            Ok((
                "",
                Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(2))),
                    rhs: Box::new(Expr::Number(Number(2))),
                    op: Op::Mul,
                },
            )),
        );
    }

    #[test]
    fn parse_cmp() {
        let cases = [
            ("11 > 2", Op::Greater),
            ("11 >= 2", Op::GreaterEq),
            ("11 == 2", Op::Eq),
            ("11 <= 2", Op::LessEq),
            ("11 < 2", Op::Less),
        ];

        for case in cases {
            assert_eq!(
                Expr::new(case.0),
                Ok((
                    "",
                    Expr::Operation {
                        lhs: Box::new(Expr::Number(Number(11))),
                        rhs: Box::new(Expr::Number(Number(2))),
                        op: case.1
                    }
                ))
            );
        }
    }

    #[test]
    fn parse_number_as_expr() {
        assert_eq!(Expr::new("456"), Ok(("", Expr::Number(Number(456)))));
    }

    #[test]
    fn eval_add() {
        let mut env = Env::new();

        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(10))),
                rhs: Box::new(Expr::Number(Number(10))),
                op: Op::Add,
            }
            .eval(&mut env),
            Ok(Val::Number(20)),
        );
    }

    #[test]
    fn eval_sub() {
        let mut env = Env::new();

        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(1))),
                rhs: Box::new(Expr::Number(Number(5))),
                op: Op::Sub,
            }
            .eval(&mut env),
            Ok(Val::Number(-4)),
        );
    }

    #[test]
    fn eval_mul() {
        let mut env = Env::new();

        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(5))),
                rhs: Box::new(Expr::Number(Number(6))),
                op: Op::Mul,
            }
            .eval(&mut env),
            Ok(Val::Number(30)),
        );
    }

    #[test]
    fn eval_div() {
        let mut env = Env::new();

        assert_eq!(
            Expr::Operation {
                lhs: Box::new(Expr::Number(Number(200))),
                rhs: Box::new(Expr::Number(Number(20))),
                op: Op::Div,
            }
            .eval(&mut env),
            Ok(Val::Number(10)),
        );
    }

    #[test]
    fn parse_binding_usage() {
        assert_eq!(
            Expr::new("bar"),
            Ok((
                "",
                Expr::BindingUsage(BindingUsage {
                    name: "bar".to_string(),
                }),
            )),
        );
    }

    #[test]
    fn parse_block() {
        assert_eq!(
            Expr::new("📦 200 🧑‍🦲"),
            Ok((
                "",
                Expr::Block(Block {
                    stmts: vec![Stmt::Expr(Expr::Number(Number(200)))],
                }),
            )),
        );
    }

    #[test]
    fn eval_cmp() {
        let nums: Vec<_> = (0..10).collect();
        for n1 in &nums {
            for n2 in &nums {
                let ops: [(&'static str, Box<dyn Fn(i32, i32) -> bool>); 5] = [
                    (">", Box::new(|a, b| a > b)),
                    (">=", Box::new(|a, b| a >= b)),
                    ("==", Box::new(|a, b| a == b)),
                    ("<=", Box::new(|a, b| a <= b)),
                    ("<", Box::new(|a, b| a < b)),
                ];
                for op in ops.iter() {
                    let expr_s = format!("{} {} {}", n1, op.0, n2);
                    let (_, expr) = Expr::new(&expr_s).unwrap();

                    let expected = Ok(Val::Bool(op.1(*n1, *n2)));

                    let mut env = Env::new();
                    let result = expr.eval(&mut env);

                    assert_eq!(result, expected);
                }
            }
        }
    }
}
