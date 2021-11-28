use crate::env::{Env, Eval};
use crate::utils;
use crate::val::Val;
use std::borrow::Cow;

#[derive(Debug, PartialEq, Clone)]
pub struct BindingUsage {
    pub name: String,
}

impl BindingUsage {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
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
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.get_binding(&self.name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Err("binding with name `i_dont_exist` does not exist".to_string()),
        );
    }
}
