use crate::env::{Env, Eval};
use crate::error::{ParseError, RuntimeError};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct Index {
    root: Expr,
    idents: Vec<String>,
}

impl Index {
    pub fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);

        let idx = s.find(kwords::INDEX).ok_or(ParseError::ExpectedIndex)?;
        let expr_s = &s[0..idx];
        let rest_s = &s[idx..];

        let (expr_rem, root) = Expr::new(expr_s)?;
        let (maybe_empty, _) = utils::extract_whitespace(expr_rem);
        if !maybe_empty.is_empty() {
            return Err(ParseError::ExpectedExpr);
        }

        let s = utils::tag(kwords::INDEX, rest_s)?;
        let (s, id0) = utils::extract_ident(s)?;

        let mut idents = vec![id0.to_string()];
        let mut s = s;
        loop {
            if let Ok(new_s) = utils::tag(kwords::INDEX, s) {
                s = new_s;
            } else {
                break;
            }

            if let Ok((new_s, ident)) = utils::extract_ident(s) {
                s = new_s;
                idents.push(ident.to_string());
            } else {
                break;
            }
        }

        Ok((s, Self { root, idents }))
    }
}

impl Eval for Index {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        let mut val = env.eval(&self.root)?.as_ref().to_owned();
        for ident in &self.idents {
            let obj = val.as_object()?;
            val = obj.0.member(ident.as_ref())?;
        }

        Ok(Cow::Owned(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Call};
    use crate::system;

    #[test]
    fn index2_parse() {
        let idx_e = Index::new("a🪆b");
        let expected = Index {
            root: Expr::BindingUsage(BindingUsage {
                name: "a".to_string(),
            }),
            idents: vec!["b".to_string()],
        };

        assert_eq!(idx_e, Ok(("", expected)));
    }

    #[test]
    fn index7_parse() {
        let idx_e = Index::new("a🪆b🪆c🪆d🪆d🪆🍇🪆🍇");
        let expected = Index {
            root: Expr::BindingUsage(BindingUsage {
                name: "a".to_string(),
            }),
            idents: vec![
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
                "d".to_string(),
                "🍇".to_string(),
                "🍇".to_string(),
            ],
        };

        assert_eq!(idx_e, Ok(("", expected)));
    }

    #[test]
    fn index0_error() {
        let idx_e = Index::new("a");
        let expected = Err(ParseError::ExpectedIndex);

        assert_eq!(idx_e, expected);
    }

    #[test]
    fn parse_call_indexed() {
        let call_e = Expr::new("📞rng🪆next");
        let expected = Expr::Call(Box::new(Call {
            func: Expr::Index(Box::new(Index {
                root: Expr::BindingUsage(BindingUsage {
                    name: "rng".to_string(),
                }),
                idents: vec!["next".to_string()],
            })),
            args: Vec::new(),
        }));

        assert_eq!(call_e, Ok(("", expected)));
    }

    #[test]
    fn eval_call_indexed() {
        let (_, call_e) = Expr::new("📞rng🪆next").unwrap();
        let mut env = Env::test();

        let (test_sys, _) = system::Test::new(&[]);
        let _ = env.eval(&crate::builtins::Builtins::new(test_sys)).unwrap();
        let result = env.eval(&call_e);

        assert!(matches!(result, Ok(Cow::Owned(Val::Number(_)))))
    }
}
