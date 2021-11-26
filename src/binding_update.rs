use crate::env::Env;
use crate::expr::Expr;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct BindingUpdate {
    pub name: String,
    pub val: Expr,
    pub set: bool,
}

impl BindingUpdate {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);

        let (s, set) = if let Ok(s) = utils::tag("👶", s) {
            (s, false)
        } else if let Ok(s) = utils::tag("set", s) {
            (s, true)
        } else {
            return Err("expected `👶` or `set`".to_string());
        };

        let (s, _) = utils::extract_whitespace(s);
        let (s, name) = utils::extract_ident(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("=", s)?;

        if s.starts_with("=") {
            return Err("unexpected =".into());
        }

        let (s, _) = utils::extract_whitespace(s);
        let (s, val) = Expr::new(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
                val,
                set,
            },
        ))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let value = self.val.eval(env)?;
        if self.set {
            env.set_binding(&self.name, value)?;
        } else {
            env.store_binding(self.name.clone(), value);
        }

        Ok(Val::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Number, Op};
    use crate::val::Val;

    #[test]
    fn parse_binding_set() {
        assert_eq!(
            BindingUpdate::new("set a = 10"),
            Ok((
                "",
                BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Number(Number(10)),
                    set: true,
                },
            )),
        );
    }

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            BindingUpdate::new("👶 a = 10 / 2"),
            Ok((
                "",
                BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Operation {
                        lhs: Box::new(Expr::Number(Number(10))),
                        rhs: Box::new(Expr::Number(Number(2))),
                        op: Op::Div,
                    },
                    set: false,
                },
            )),
        );
    }

    #[test]
    fn can_parse_binding_without_space() {
        assert_eq!(
            BindingUpdate::new("👶aaa=1"),
            Ok((
                "",
                BindingUpdate {
                    name: "aaa".to_string(),
                    val: Expr::Number(Number(1)),
                    set: false,
                }
            )),
        );
    }

    #[test]
    fn eval_binding_def() {
        let (_s, bd) = BindingUpdate::new("👶 🍆💦 = 420 / 69").unwrap();
        let mut env = Env::new();

        bd.eval(&mut env).unwrap();

        assert_eq!(env.get_binding("🍆💦"), Ok(Val::Number(6)));
    }
}
