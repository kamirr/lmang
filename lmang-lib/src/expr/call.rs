use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub(crate) func: Expr,
    pub(crate) args: Vec<Expr>,
}

impl Call {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::CALL, s)?;

        let (s, func) = Expr::new(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let mut s = s;
        let mut args = Vec::new();

        while let Ok((new_s, arg_val_e)) = Expr::new(s) {
            let (new_s, _) = utils::extract_whitespace(new_s);
            s = new_s;
            args.push(arg_val_e);
        }

        Ok((s, Call { func, args }))
    }
}

impl Eval for Call {
    fn eval(&self, env: &mut Env) -> Result<Val, RuntimeError> {
        let args: Result<Vec<Val>, _> = self.args.iter().map(|arg| env.eval(arg)).collect();
        let mut args = args?;

        let func_owned = env.eval(&self.func)?;
        func_owned.apply_to_root(|val| -> Result<_, RuntimeError> {
            #[cfg(feature = "web")]
            if let Val::JsValue(ref jv) = val {
                Val::convert_from_jv(jv.clone())
                    .as_func()?
                    .0
                    .call(args.as_mut_slice(), env)
            } else {
                val.as_func()?.0.call(args.as_mut_slice(), env)
            }
            #[cfg(not(feature = "web"))]
            {
                val.as_func()?.0.call(args.as_mut_slice(), env)
            }
        })?
    }
}

impl crate::expr::Format for Call {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        write!(w, "{} ", kwords::CALL)?;
        self.func.format(w, depth)?;

        if !self.args.is_empty() {
            write!(w, " ")?;
        }

        for (k, e) in self.args.iter().enumerate() {
            e.format(w, depth)?;
            if k != self.args.len() - 1 {
                write!(w, " ")?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Expr, Literal};
    use std::collections::VecDeque;

    #[test]
    fn parse_call() {
        let call_e = Call::new("📞 add a 1");
        let expected = Call {
            func: Expr::BindingUsage(BindingUsage {
                name: "add".to_string(),
            }),
            args: vec![
                Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                }),
                Expr::Literal(Literal(Val::Number(1))),
            ],
        };

        assert_eq!(call_e, Ok(("", expected)));
    }

    #[test]
    fn parse_call_with_remainder() {
        let call_e = Call::new("📞 add a 1 💪 👶 a = 2");
        let expected = Call {
            func: Expr::BindingUsage(BindingUsage {
                name: "add".to_string(),
            }),
            args: vec![
                Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                }),
                Expr::Literal(Literal(Val::Number(1))),
            ],
        };

        assert_eq!(call_e, Ok(("💪 👶 a = 2", expected)));
    }

    #[test]
    fn eval_call() {
        let (_, call_e) = Call::new("📞 🧰 🔍 ➡️ 🔍 + 1 🧑‍🦲 4").unwrap();
        let mut env = Env::test();
        let result = env.eval(&call_e);

        assert_eq!(result, Ok(Val::Number(5)));
    }

    #[test]
    fn eval_variadic() {
        let (_, call_e) = Call::new("📞 🧰 👨‍👨‍👦🔍 ➡️ 🔍 🧑‍🦲 2 3 10").unwrap();
        let mut env = Env::test();
        let result = env.eval(&call_e);

        let mut dq = VecDeque::new();
        dq.push_back(Val::Number(2));
        dq.push_back(Val::Number(3));
        dq.push_back(Val::Number(10));
        let dq_val = Val::Deque(Box::new(dq));
        assert_eq!(result, Ok(dq_val));
    }

    #[test]
    fn eval_call_fib() {
        let (_, expr_e) = Expr::new(
            "📦
                👶 🖩 = 🧰 #️⃣ ➡️
                    ❓ #️⃣ < 2
                        1
                    🧑‍🦲 😡
                        👶 🅰️ = 📞 🖩 📦 #️⃣ - 1 🧑‍🦲 💪
                        👶 🅱️ = 📞 🖩 📦 #️⃣ - 2 🧑‍🦲 💪

                        🅰️ + 🅱️
                    🧑‍🦲
                🧑‍🦲 💪

                📞 🖩 💾
            🧑‍🦲",
        )
        .unwrap();

        let cases = [
            (0, 1),
            (1, 1),
            (2, 2),
            (3, 3),
            (4, 5),
            (5, 8),
            (6, 13),
            (7, 21),
        ];
        let mut env = Env::test();
        for (arg, fib) in cases.iter() {
            env.store_binding("💾".to_string(), Val::Number(*arg));
            let result = env.eval(&expr_e);
            assert_eq!(result, Ok(Val::Number(*fib)));
        }
    }

    #[test]
    fn format() {
        let (_, call_e) = Call::new("📞add a    1").unwrap();
        assert_eq!(format!("{}", crate::expr::Display(&call_e)), "📞 add a 1");
    }
}
