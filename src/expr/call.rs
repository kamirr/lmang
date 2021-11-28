use crate::env::{Env, Eval};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func: Expr,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
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
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        let func = env.eval(&self.func)?.as_func()?;
        let args: Result<Vec<Val>, _> = self
            .args
            .iter()
            .map(|arg| env.eval(arg).map(|cow| cow.as_ref().to_owned()))
            .collect();
        let args = args?;

        func.0.call(args.as_slice(), env).map(|v| Cow::Owned(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Expr, Literal};

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

        assert_eq!(result, Ok(Cow::Owned(Val::Number(5))));
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
            assert_eq!(result, Ok(Cow::Owned(Val::Number(*fib))));
        }
    }
}
