use crate::env::{Env, Eval};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct BindingUpdate {
    pub name: String,
    pub val: Expr,
    pub set: bool,
}

impl BindingUpdate {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);

        let (s, set) = if let Ok(s) = utils::tag(kwords::LET, s) {
            (s, false)
        } else if let Ok(s) = utils::tag(kwords::SET, s) {
            (s, true)
        } else {
            return Err(format!("expected `{}` or `{}`", kwords::LET, kwords::SET));
        };

        let (s, _) = utils::extract_whitespace(s);
        let (s, name) = utils::extract_ident(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::UPDATE_SEP, s)?;

        // as of now two update separators concatenated without whitespace
        // should instead be parsed as equality check.
        if s.starts_with(kwords::UPDATE_SEP) {
            return Err(format!("unexpected {}", kwords::UPDATE_SEP));
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
}

impl Eval for BindingUpdate {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        let value = env.eval(&self.val)?.as_ref().to_owned();
        match value {
            Val::Break(_) => Ok(Cow::Owned(value)),
            _ => {
                if self.set {
                    env.set_binding(&self.name, value)?;
                } else {
                    env.store_binding(self.name.clone(), value);
                }

                Ok(Cow::Owned(Val::Unit))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Literal, Op};
    use crate::val::Val;

    #[test]
    fn parse_binding_set() {
        assert_eq!(
            BindingUpdate::new("set a = 10"),
            Ok((
                "",
                BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Literal(Literal(Val::Number(10))),
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
                        lhs: Box::new(Expr::Literal(Literal(Val::Number(10)))),
                        rhs: Box::new(Expr::Literal(Literal(Val::Number(2)))),
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
                    val: Expr::Literal(Literal(Val::Number(1))),
                    set: false,
                }
            )),
        );
    }

    #[test]
    fn eval_binding_def() {
        let (_s, bd) = BindingUpdate::new("👶 🍆💦 = 420 / 69").unwrap();
        let mut env = Env::test();

        env.eval(&bd).unwrap();

        assert_eq!(env.get_binding("🍆💦"), Ok(Cow::Borrowed(&Val::Number(6))));
    }
}
