use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::block::{Block, FormatImplicit};
use crate::utils::{self, kwords};
use crate::val::{Callee, DynFunc, Val};
use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Arg {
    Single(String),
    Variadic(String),
}

impl Arg {
    pub fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let (s, variadic) = match utils::tag(kwords::VARIADIC, s) {
            Ok(s) => (s, true),
            _ => (s, false),
        };
        let (s, ident) = utils::extract_ident(s)?;

        let arg = if variadic {
            Arg::Variadic(ident.into())
        } else {
            Arg::Single(ident.into())
        };

        Ok((s, arg))
    }
}

impl crate::expr::Format for Arg {
    fn format(&self, w: &mut dyn std::fmt::Write, _depth: usize) -> std::fmt::Result {
        let name = match self {
            Self::Single(s) => s,
            Self::Variadic(s) => {
                write!(w, "{}", kwords::VARIADIC)?;
                s
            }
        };
        write!(w, "{}", name)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Func {
    pub(crate) args: Vec<Arg>,
    pub(crate) body: Block,
}

impl Func {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::FUNC, s)?;

        let mut s = s;
        let mut args = Vec::new();

        let (new_s, _) = utils::extract_whitespace(s);
        s = new_s;

        let mut variadic_found = false;
        while let Ok((new_s, arg)) = Arg::new(s) {
            if variadic_found {
                return Err(ParseError::PrematureVariadic);
            }

            s = new_s;
            variadic_found |= matches!(arg, Arg::Variadic(_));
            args.push(arg);
        }

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::FUNC_SEP, s)?;

        let (s, body) = Block::implicit(s)?;

        Ok((s, Func { args, body }))
    }
}

impl Eval for Func {
    fn eval(&self, _env: &mut Env) -> Result<Val, RuntimeError> {
        let funcval = FuncVal {
            args: self.args.clone(),
            body: self.body.clone(),
            parent: None,
        };
        Ok(Val::Func(DynFunc(Box::new(funcval))))
    }
}

impl crate::expr::Format for Func {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        write!(w, "{} ", kwords::FUNC)?;

        for arg in &self.args {
            arg.format(w, depth)?;
            write!(w, " ")?;
        }
        writeln!(w, "{}", kwords::FUNC_SEP)?;

        FormatImplicit(&self.body).format(w, depth)?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncVal {
    pub(crate) args: Vec<Arg>,
    pub(crate) body: Block,
    pub(crate) parent: Option<HashMap<String, Val, ahash::RandomState>>,
}

impl Callee for FuncVal {
    fn call(&self, args: &mut [Val], env: &mut Env) -> Result<Val, RuntimeError> {
        env.push();

        let mut idx = 0;
        for param in &self.args {
            match param {
                Arg::Single(name) => {
                    let val = args[idx].clone();
                    idx += 1;
                    env.store_binding(name.to_string(), val);
                }
                Arg::Variadic(name) => {
                    let mut dq = VecDeque::new();
                    while idx < args.len() {
                        dq.push_back(args[idx].clone());
                        idx += 1;
                    }

                    env.store_binding(name.to_string(), Val::Deque(Box::new(dq)));
                }
            }
        }

        if let Some(parent_vars) = &self.parent {
            for (k, v) in parent_vars {
                env.store_binding(k.to_string(), v.clone());
            }
        }

        let result = if idx == args.len() {
            env.eval(&self.body)
        } else {
            Err(RuntimeError::WrongArgsN)
        };

        env.pop();

        result
    }

    fn clone_box(&self) -> Box<dyn Callee> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }

    fn dyn_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "user-defined")
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_update::Mode, binding_usage::BindingUsage, BindingUpdate, Expr, Op};

    #[test]
    fn func_parse_id() {
        let func_e = Func::new("🧰 a ➡️ a 🧑‍🦲");
        let expected = Func {
            args: vec![Arg::Single("a".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn func_parse_variadic() {
        let func_e = Func::new("🧰 👨‍👨‍👦v ➡️ v 🧑‍🦲");
        let expected = Func {
            args: vec![Arg::Variadic("v".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "v".to_string(),
                })],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn error_on_premature_variadic() {
        let func_e = Func::new("🧰 👨‍👨‍👦v x ➡️ v 🧑‍🦲");
        let expected = ParseError::PrematureVariadic;

        assert_eq!(func_e, Err(expected));
    }

    #[test]
    fn func_parse_sum() {
        let func_e = Func::new("🧰 a b ➡️ a + b 🧑‍🦲");
        let expected = Func {
            args: vec![Arg::Single("a".to_string()), Arg::Single("b".to_string())],
            body: Block {
                exprs: vec![Expr::Operation {
                    lhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "a".to_string(),
                    })),
                    rhs: Box::new(Expr::BindingUsage(BindingUsage {
                        name: "b".to_string(),
                    })),
                    op: Op::Add,
                }],
            },
        };

        assert_eq!(func_e, Ok(("", expected)));
    }

    #[test]
    fn func_parse_id_in_block() {
        let expr = Expr::new("📦 👶 id = 🧰 a ➡️ a 🧑‍🦲 💪 id 🧑‍🦲");
        let expected = Expr::Block(Block {
            exprs: vec![
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "id".to_string(),
                    val: Expr::Func(Box::new(Func {
                        args: vec![Arg::Single("a".to_string())],
                        body: Block {
                            exprs: vec![Expr::BindingUsage(BindingUsage {
                                name: "a".to_string(),
                            })],
                        },
                    })),
                    mode: Mode::CreateLocal,
                })),
                Expr::BindingUsage(BindingUsage {
                    name: "id".to_string(),
                }),
            ],
        });

        assert_eq!(expr, Ok(("", expected)));
    }

    #[test]
    fn func_eval_id() {
        let (_, func_e) = Func::new("🧰 a ➡️ a 🧑‍🦲").unwrap();
        let expected = FuncVal {
            args: vec![Arg::Single("a".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })],
            },
            parent: None,
        };

        let mut env = Env::test();
        let result = env.eval(&func_e);

        assert_eq!(result, Ok(Val::Func(DynFunc(Box::new(expected)))));
    }

    #[test]
    fn func_eval_variadic_id() {
        let (_, func_e) = Func::new("🧰 👨‍👨‍👦a ➡️ a 🧑‍🦲").unwrap();
        let expected = FuncVal {
            args: vec![Arg::Variadic("a".to_string())],
            body: Block {
                exprs: vec![Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })],
            },
            parent: None,
        };

        let mut env = Env::test();
        let result = env.eval(&func_e);

        assert_eq!(result, Ok(Val::Func(DynFunc(Box::new(expected)))));
    }

    #[test]
    fn func_var_shadowing() {
        let (_, func_def) = Expr::new(
            "👶 f = 🧰 a ➡️
                👶 b = 2  💪
                ♻️ a = 1 💪
                ♻️ x = 1 💪
                ♻️ c = 1 💪
                ♻️ b = 4 💪
                
                a
            🧑‍🦲",
        )
        .unwrap();
        let (_, func_call) = Expr::new("📞 f x").unwrap();

        let mut env = Env::test();
        env.eval(&func_def).unwrap();

        env.store_binding("c".to_string(), Val::Number(0));
        env.store_binding("x".to_string(), Val::Number(8));
        let res = env.eval(&func_call);

        assert_eq!(res, Ok(Val::Number(1)));
        assert_eq!(env.get_binding("c"), Ok(Val::Number(1)));
        assert_eq!(env.get_binding("x"), Ok(Val::Number(1)));
        assert_eq!(
            env.get_binding("a"),
            Err(RuntimeError::NoBinding("a".into()))
        );
        assert_eq!(
            env.get_binding("b"),
            Err(RuntimeError::NoBinding("b".into()))
        );
    }

    #[test]
    fn format() {
        let (_, func_e) = Func::new("🧰x👨‍👨‍👦v➡️v🧑‍🦲").unwrap();
        assert_eq!(
            format!("{}", crate::expr::Display(&func_e)),
            "🧰 x 👨‍👨‍👦v ➡️\n    v\n🧑‍🦲"
        );
    }
}
