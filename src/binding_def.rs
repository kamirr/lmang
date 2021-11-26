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

        let s = utils::tag("let", s)?;
        let (s, _) = utils::extract_whitespace1(s)?;

        let (s, name) = utils::extract_ident(s)?;
        let (s, _) = utils::extract_whitespace(s);

        let s = utils::tag("=", s)?;
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
            BindingDef::new("let a = 10 / 2"),
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
    fn cannot_parse_binding_def_without_space_after_let() {
        assert_eq!(
            BindingDef::new("letaaa=1+2"),
            Err("expected a space".to_string()),
        );
    }

    #[test]
    fn eval_binding_def() {
        let (_s, bd) = BindingDef::new("let 🍆💦 = 420 / 69").unwrap();
        let mut env = Env::new();

        bd.eval(&mut env).unwrap();

        assert_eq!(env.get_binding("🍆💦"), Ok(Val::Number(6)));
    }
}
