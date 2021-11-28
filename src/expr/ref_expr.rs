use crate::env::{Env, Eval};
use crate::utils;
use crate::val::Val;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct Ref {
    ident: String,
}

impl Ref {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
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
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.take_ref(&self.ident)
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
        let (_, ref_e) = Ref::new("🔖 var").unwrap();

        let mut env = Env::new();
        env.store_binding("var".to_string(), Val::Number(0));

        let result = env.eval(&ref_e).unwrap();
        *result.as_val_ref().unwrap().borrow_mut() = Val::Number(42);

        let var_val = env.get_binding("var").unwrap();

        assert_eq!(*var_val.as_val_ref().unwrap().borrow(), Val::Number(42));
    }
}
