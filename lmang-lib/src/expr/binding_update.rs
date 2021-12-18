use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct BindingUpdate {
    pub(crate) name: String,
    pub(crate) val: Expr,
    pub(crate) set: bool,
}

impl BindingUpdate {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);

        let (s, set) = if let Ok(s) = utils::tag(kwords::LET, s) {
            (s, false)
        } else if let Ok(s) = utils::tag(kwords::SET, s) {
            (s, true)
        } else {
            return Err(ParseError::ExpectedBindingUpdate);
        };

        let (s, _) = utils::extract_whitespace(s);
        let (s, name) = utils::extract_ident(s)?;

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::UPDATE_SEP, s)?;

        // as of now two update separators concatenated without whitespace
        // should instead be parsed as equality check.
        if s.starts_with(kwords::UPDATE_SEP) {
            return Err(ParseError::UnexpectedEquals);
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
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Val, RuntimeError> {
        let value = env.eval(&self.val)?;
        match value {
            Val::Break(_) => Ok(value),
            _ => {
                if self.set {
                    env.set_binding(&self.name, value)?;
                } else {
                    env.store_binding(self.name.clone(), value);
                }

                Ok(Val::Unit)
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
            BindingUpdate::new("♻️ a = 10"),
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

        assert_eq!(env.get_binding("🍆💦"), Ok(Val::Number(6)));
    }
}
