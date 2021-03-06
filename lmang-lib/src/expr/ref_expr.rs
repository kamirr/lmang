use crate::env::{Env, Eval};
use crate::error::ParseError;
use crate::utils;
use crate::val::Val;

#[derive(Clone, Debug, PartialEq)]
pub struct Ref {
    pub(crate) ident: String,
}

impl Ref {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("🔖", s)?;

        let (s, _) = utils::extract_whitespace(s);
        let (s, ident) = utils::extract_ident(s)?;

        Ok((
            s,
            Ref {
                ident: ident.to_string(),
            },
        ))
    }
}

impl Eval for Ref {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        Ok(env.take_ref(&self.ident)?)
    }
}

impl crate::expr::Format for Ref {
    fn format(&self, w: &mut dyn std::fmt::Write, _depth: usize) -> std::fmt::Result {
        write!(w, "🔖{}", self.ident)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ref() {
        let ref_e = Ref::new("🔖x");
        let expected = Ref {
            ident: "x".to_string(),
        };

        assert_eq!(ref_e, Ok(("", expected)))
    }

    #[test]
    fn parse_ref_emoji() {
        let ref_e = Ref::new("🔖🔥🔥");
        let expected = Ref {
            ident: "🔥🔥".to_string(),
        };

        assert_eq!(ref_e, Ok(("", expected)))
    }

    #[test]
    fn eval_ref() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let (_, ref_e) = Ref::new("🔖 var").unwrap();

        let mut env = Env::new();
        env.store_binding("var".to_string(), Val::Number(0));

        let result = env.eval(&ref_e).unwrap();
        *result.as_val_ref().unwrap().borrow_mut() = Val::Number(42);

        let var_val = env.get_binding("var").unwrap();

        assert_eq!(
            var_val.as_val_ref(),
            Ok(&Rc::new(RefCell::new(Val::Number(42))))
        );
    }

    #[test]
    fn format() {
        let (_, ref_e) = Ref::new("🔖  var").unwrap();
        let d = crate::expr::Display(&ref_e);

        assert_eq!(format!("{}", d), "🔖var");
    }
}
