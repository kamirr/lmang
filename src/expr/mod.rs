pub mod binding_update;
pub mod binding_usage;
pub mod block;
pub mod break_expr;
pub mod call;
pub mod class;
pub mod func;
pub mod if_expr;
pub mod index;
pub mod literal;
pub mod loop_expr;
pub mod ref_expr;

use crate::env::{Env, Eval};
use crate::utils::{self, kwords};
use crate::val::Val;
use binding_update::BindingUpdate;
use binding_usage::BindingUsage;
use block::Block;
use break_expr::Break;
use call::Call;
use class::Class;
use func::Func;
use if_expr::If;
use index::Index;
use literal::Literal;
use loop_expr::Loop;
use ref_expr::Ref;

use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
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
        utils::tag(kwords::ADD, s)
            .map(|s| (s, Self::Add))
            .or_else(|_| utils::tag(kwords::SUB, s).map(|s| (s, Self::Sub)))
            .or_else(|_| utils::tag(kwords::MUL, s).map(|s| (s, Self::Mul)))
            .or_else(|_| utils::tag(kwords::DIV, s).map(|s| (s, Self::Div)))
            .or_else(|_| utils::tag(kwords::GE, s).map(|s| (s, Self::GreaterEq)))
            .or_else(|_| utils::tag(kwords::GT, s).map(|s| (s, Self::Greater)))
            .or_else(|_| utils::tag(kwords::LE, s).map(|s| (s, Self::LessEq)))
            .or_else(|_| utils::tag(kwords::LT, s).map(|s| (s, Self::Less)))
            .or_else(|_| utils::tag(kwords::EQ, s).map(|s| (s, Self::Eq)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Operation {
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        op: Op,
    },
    Literal(Literal),
    BindingUpdate(Box<BindingUpdate>),
    BindingUsage(BindingUsage),
    Block(Block),
    Class(Box<Class>),
    If(Box<If>),
    Index(Box<Index>),
    Break(Box<Break>),
    Loop(Box<Loop>),
    Func(Box<Func>),
    Call(Box<Call>),
    Ref(Ref),
}

impl Expr {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);

        BindingUpdate::new(s)
            .map(|(s, update)| (s, Self::BindingUpdate(Box::new(update))))
            .or_else(|_| Self::new_operation(s))
            .or_else(|_| Block::explicit(s).map(|(s, block)| (s, Self::Block(block))))
            .or_else(|_| Call::new(s).map(|(s, call_e)| (s, Self::Call(Box::new(call_e)))))
            .or_else(|_| Class::new(s).map(|(s, class_e)| (s, Self::Class(Box::new(class_e)))))
            .or_else(|_| If::new(s).map(|(s, if_e)| (s, Self::If(Box::new(if_e)))))
            .or_else(|_| Index::new(s).map(|(s, index_e)| (s, Self::Index(Box::new(index_e)))))
            .or_else(|_| Break::new(s).map(|(s, break_e)| (s, Self::Break(Box::new(break_e)))))
            .or_else(|_| Loop::new(s).map(|(s, loop_e)| (s, Self::Loop(Box::new(loop_e)))))
            .or_else(|_| Func::new(s).map(|(s, func_e)| (s, Self::Func(Box::new(func_e)))))
            .or_else(|_| Literal::new(s).map(|(s, literal)| (s, Self::Literal(literal))))
            .or_else(|_| BindingUsage::new(s).map(|(s, usage)| (s, Self::BindingUsage(usage))))
            .or_else(|_| Ref::new(s).map(|(s, ref_expr)| (s, Self::Ref(ref_expr))))
    }

    fn new_operation(s: &str) -> Result<(&str, Self), String> {
        let mut op_b_idx = 0;
        let mut op_c_idx = 0;

        loop {
            let sub = &s[op_b_idx..];
            if Op::new(sub).is_ok() {
                break;
            }

            let c = s
                .chars()
                .nth(op_c_idx)
                .ok_or_else(|| "unexpected eof".to_string())?;

            op_b_idx += c.len_utf8();
            op_c_idx += 1;
        }

        let (sub, lhs) = Expr::new(&s[0..op_b_idx])?;
        let (sub, _) = utils::extract_whitespace(sub);

        if !sub.is_empty() {
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
}

impl Eval for Expr {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        let result = match self {
            Self::Operation { lhs, rhs, op } => {
                let lhs = env.eval(lhs.as_ref())?.as_ref().clone();
                let rhs = env.eval(rhs.as_ref())?;

                Ok(Cow::Owned(match op {
                    Op::Add => (&lhs + rhs.as_ref())?,
                    Op::Sub => (&lhs - rhs.as_ref())?,
                    Op::Mul => (&lhs * rhs.as_ref())?,
                    Op::Div => (&lhs / rhs.as_ref())?,
                    Op::Greater => lhs.try_gt(rhs.as_ref())?,
                    Op::GreaterEq => lhs.try_ge(rhs.as_ref())?,
                    Op::Eq => lhs.try_eq(rhs.as_ref())?,
                    Op::LessEq => lhs.try_le(rhs.as_ref())?,
                    Op::Less => lhs.try_lt(rhs.as_ref())?,
                }))
            }
            Self::BindingUpdate(bu) => env.eval(bu.as_ref()),
            Self::BindingUsage(bu) => env.eval(bu),
            Self::Block(block) => env.eval(block),
            Self::Class(class) => env.eval(class.as_ref()),
            Self::If(if_e) => env.eval(if_e.as_ref()),
            Self::Index(index_e) => env.eval(index_e.as_ref()),
            Self::Break(break_e) => env.eval(break_e.as_ref()),
            Self::Loop(loop_e) => env.eval(loop_e.as_ref()),
            Self::Func(func_e) => env.eval(func_e.as_ref()),
            Self::Call(call_e) => env.eval(call_e.as_ref()),
            Self::Literal(val) => Ok(Cow::Owned(val.0.clone())),
            Self::Ref(ref_expr) => env.eval(ref_expr),
        };

        let weak_err = "dangling ref expired".to_string();
        match result {
            Ok(Cow::Owned(Val::Weak(ref wk))) => {
                Ok(Cow::Owned(Val::Ref(wk.upgrade().ok_or(weak_err)?)))
            }
            Ok(Cow::Borrowed(Val::Weak(wk))) => {
                Ok(Cow::Owned(Val::Ref(wk.upgrade().ok_or(weak_err)?)))
            }
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                    lhs: Box::new(Expr::Literal(Literal(Val::Number(1)))),
                    rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
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
                    lhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
                    rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
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
                        lhs: Box::new(Expr::Literal(Literal(Val::Number(11)))),
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
                        op: case.1
                    }
                ))
            );
        }
    }

    #[test]
    fn parse_number_as_expr() {
        assert_eq!(
            Expr::new("456"),
            Ok(("", Expr::Literal(Literal(Val::Number(456)))))
        );
    }

    #[test]
    fn eval_add() {
        let mut env = Env::test();

        assert_eq!(
            env.eval(&Expr::Operation {
                lhs: Box::new(Expr::Literal(Literal(Val::Number(10)))),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(10)))),
                op: Op::Add,
            }),
            Ok(Cow::Owned(Val::Number(20))),
        );
    }

    #[test]
    fn eval_sub() {
        let mut env = Env::test();

        assert_eq!(
            env.eval(&Expr::Operation {
                lhs: Box::new(Expr::Literal(Literal(Val::Number(1)))),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(5)))),
                op: Op::Sub,
            }),
            Ok(Cow::Owned(Val::Number(-4))),
        );
    }

    #[test]
    fn eval_mul() {
        let mut env = Env::test();

        assert_eq!(
            env.eval(&Expr::Operation {
                lhs: Box::new(Expr::Literal(Literal(Val::Number(5)))),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(6)))),
                op: Op::Mul,
            }),
            Ok(Cow::Owned(Val::Number(30))),
        );
    }

    #[test]
    fn eval_div() {
        let mut env = Env::test();

        assert_eq!(
            env.eval(&Expr::Operation {
                lhs: Box::new(Expr::Literal(Literal(Val::Number(200)))),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(20)))),
                op: Op::Div,
            }),
            Ok(Cow::Owned(Val::Number(10))),
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
                    exprs: vec![Expr::Literal(Literal(Val::Number(200)))],
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

                    let expected = Ok(Cow::Owned(Val::Bool(op.1(*n1, *n2))));

                    let mut env = Env::test();
                    let result = env.eval(&expr);

                    assert_eq!(result, expected);
                }
            }
        }
    }
}
