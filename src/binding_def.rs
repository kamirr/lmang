use crate::env::Env;
use crate::expr::Expr;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
pub struct BindingDef {
    pub name: String,
    pub val: Expr,
}

impl BindingDef {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag("👶", s)?;
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
            },
        ))
    }

    pub(crate) fn eval(&self, env: &mut Env) -> Result<Val, String> {
        let value = self.val.eval(env)?;
        env.store_binding(self.name.clone(), value);

        Ok(Val::Unit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Number, Op};
    use crate::val::Val;

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            BindingDef::new("👶 a = 10 / 2"),
            Ok((
                "",
                BindingDef {
                    name: "a".to_string(),
                    val: Expr::Operation {
                        lhs: Box::new(Expr::Number(Number(10))),
                        rhs: Box::new(Expr::Number(Number(2))),
                        op: Op::Div,
                    },
                },
            )),
        );
    }

    #[test]
    fn can_parse_binding_without_space() {
        assert_eq!(
            BindingDef::new("👶aaa=1"),
            Ok((
                "",
                BindingDef {
                    name: "aaa".to_string(),
                    val: Expr::Number(Number(1)),
                }
            )),
        );
    }

    #[test]
    fn eval_binding_def() {
        let (_s, bd) = BindingDef::new("👶 🍆💦 = 420 / 69").unwrap();
        let mut env = Env::new();

        bd.eval(&mut env).unwrap();

        assert_eq!(env.get_binding("🍆💦"), Ok(Val::Number(6)));
    }
}
