use crate::env::{Env, Eval};
use crate::error::ParseError;
use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct BindingUsage {
    pub(crate) name: String,
}

impl BindingUsage {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, name) = utils::extract_ident(s)?;

        Ok((
            s,
            Self {
                name: name.to_string(),
            },
        ))
    }
}

impl Eval for BindingUsage {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        Ok(env.get_binding(&self.name)?)
    }
}

impl crate::expr::Format for BindingUsage {
    fn format(&self, w: &mut dyn std::fmt::Write, _depth: usize) -> std::fmt::Result {
        write!(w, "{}", self.name)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RuntimeError;

    #[test]
    fn parse_binding_usage() {
        assert_eq!(
            BindingUsage::new("abc"),
            Ok((
                "",
                BindingUsage {
                    name: "abc".to_string(),
                },
            )),
        );
    }

    #[test]
    fn eval_non_existent_binding_usage() {
        let mut empty_env = Env::default();

        assert_eq!(
            empty_env.eval(&BindingUsage {
                name: "i_dont_exist".to_string(),
            }),
            Err(RuntimeError::NoBinding("i_dont_exist".into()).into())
        );
    }

    #[test]
    fn format() {
        let (_, bu_e) = BindingUsage::new("abc").unwrap();

        assert_eq!(format!("{}", crate::expr::Display(&bu_e)), "abc");
    }
}
