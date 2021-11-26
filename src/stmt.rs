use crate::binding_update::BindingUpdate;
use crate::env::Env;
use crate::expr::Expr;
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    BindingUpdate(BindingUpdate),
    Expr(Expr),
}

impl Stmt {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        BindingUpdate::new(s)
            .map(|(s, binding_def)| (s, Self::BindingUpdate(binding_def)))
            .or_else(|_| Expr::new(s).map(|(s, expr)| (s, Self::Expr(expr))))
    }

    pub fn eval(&self, env: &mut Env) -> Result<Val, String> {
        match self {
            Self::BindingUpdate(bd) => bd.eval(env),
            Self::Expr(e) => e.eval(env),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{Number, Op};

    #[test]
    fn parse_binding_def() {
        assert_eq!(
            Stmt::new("👶 a = 10"),
            Ok((
                "",
                Stmt::BindingUpdate(BindingUpdate {
                    name: "a".to_string(),
                    val: Expr::Number(Number(10)),
                    set: false,
                }),
            )),
        );
    }

    #[test]
    fn parse_expr() {
        assert_eq!(
            Stmt::new("1+1"),
            Ok((
                "",
                Stmt::Expr(Expr::Operation {
                    lhs: Box::new(Expr::Number(Number(1))),
                    rhs: Box::new(Expr::Number(Number(1))),
                    op: Op::Add,
                }),
            )),
        );
    }
}
