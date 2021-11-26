use crate::env::Env;
use crate::expr::Expr;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Call {
    pub func: Expr,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("📞", s)?;

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

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let func = self.func.eval(env)?.as_func()?;
        let args: Result<Vec<Val>, _> = self.args.iter().map(|arg| arg.eval(env)).collect();
        let args = args?;

        env.push();
        for (arg_name, arg_val) in func.args.iter().zip(args.iter()) {
            env.store_binding(arg_name.0.clone(), arg_val.clone());
        }

        let result = func.body.eval(env);
        env.pop();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{binding_usage::BindingUsage, Expr, Number};
    use crate::stmt::Stmt;

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
                Expr::Number(Number(1)),
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
                Expr::Number(Number(1)),
            ],
        };

        assert_eq!(call_e, Ok(("💪 👶 a = 2", expected)));
    }

    #[test]
    fn eval_call() {
        let (_, call_e) = Call::new("📞 🧰 🔍 ➡️ 🔍 + 1 🧑‍🦲 4").unwrap();
        let mut env = Env::new();
        let result = call_e.eval(&mut env);

        assert_eq!(result, Ok(Val::Number(5)));
    }

    #[test]
    fn eval_call_fib() {
        let (_, stmt_e) = Stmt::new(
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
        let mut env = Env::new();
        for (arg, fib) in cases.iter() {
            env.store_binding("💾".to_string(), Val::Number(*arg));
            let result = stmt_e.eval(&mut env);
            assert_eq!(result, Ok(Val::Number(*fib)));
        }
    }
}
