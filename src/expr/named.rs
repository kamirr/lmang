use crate::env::{Env, Eval};
use crate::expr::Expr;
use crate::utils::{self, kwords};
use crate::val::Val;
use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq)]
pub struct Named {
    name: String,
    expr: Expr,
}

impl Named {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let (s, name) = utils::extract_ident(s)?;
        let s = utils::tag(kwords::NAMED, s)?;
        let (s, expr) = Expr::new(s)?;

        Ok((
            s,
            Named {
                name: name.to_string(),
                expr,
            },
        ))
    }
}

impl Eval for Named {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        let val = Val::Named((
            self.name.clone(),
            Box::new(self.expr.eval(env)?.into_owned()),
        ));
        Ok(Cow::Owned(val))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{BindingUsage, Literal, Op};

    #[test]
    fn parse_named_binding_usage() {
        let named_e = Named::new("arg:value");
        let expected = Named {
            name: "arg".to_string(),
            expr: Expr::BindingUsage(BindingUsage {
                name: "value".to_string(),
            }),
        };

        assert_eq!(named_e, Ok(("", expected)));
    }

    #[test]
    fn parse_named_expr() {
        let named_e = Named::new("arg:a*3");
        let expected = Named {
            name: "arg".to_string(),
            expr: Expr::Operation {
                lhs: Box::new(Expr::BindingUsage(BindingUsage {
                    name: "a".to_string(),
                })),
                rhs: Box::new(Expr::Literal(Literal(Val::Number(3)))),
                op: Op::Mul,
            },
        };

        assert_eq!(named_e, Ok(("", expected)));
    }

    #[test]
    fn eval_named() {
        let (_, named_e) = Named::new("arg:3*3").unwrap();
        let mut env = Env::test();
        let result = env.eval(&named_e);

        assert_eq!(
            result,
            Ok(Cow::Owned(Val::Named((
                "arg".to_string(),
                Box::new(Val::Number(9))
            )))),
        );
    }

    #[test]
    fn eval_named_binding() {
        let (_, expr_e) = Expr::new("👶 x = arg:3*3").unwrap();
        let mut env = Env::test();
        env.eval(&expr_e).unwrap();
        let result = env.get_binding("x");

        assert_eq!(
            result,
            Ok(Cow::Borrowed(&Val::Named((
                "arg".to_string(),
                Box::new(Val::Number(9))
            )))),
        );
    }
}
