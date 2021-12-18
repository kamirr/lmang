use crate::error::ParseError;
use crate::utils;
use crate::val::Val;
use std::collections::VecDeque;

#[derive(Debug, PartialEq)]
struct Number(i32);

impl Number {
    fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let (s, number) = utils::extract_digits(s)?;
        Ok((s, Self(number.parse().unwrap())))
    }
}

#[derive(Debug, PartialEq)]
struct Char(char);

impl Char {
    fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(utils::kwords::CHAR_LIT, s)?;
        let c = s.chars().next().ok_or_else(|| ParseError::UnexpectedEof)?;
        let s = &s[c.len_utf8()..];
        let s = utils::tag(utils::kwords::CHAR_LIT, s)?;

        Ok((s, Self(c)))
    }
}

#[derive(Debug, PartialEq)]
struct StringLiteral(VecDeque<Val>);

impl StringLiteral {
    fn new(s: &str) -> Result<(&str, Self), ParseError> {
        use utils::kwords::STR_LIT;

        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(STR_LIT, s)?;
        let (s, lit) = utils::take_while(|c| !STR_LIT.starts_with(c), s);
        let s = utils::tag(STR_LIT, s)?;

        let deque = lit.chars().map(Val::Char).collect();

        Ok((s, Self(deque)))
    }
}

#[derive(Debug, PartialEq)]
struct Bool(bool);

impl Bool {
    fn new(s: &str) -> Result<(&str, Self), ParseError> {
        use utils::kwords::{FALSE, TRUE};

        let (s, _) = utils::extract_whitespace(s);
        if let Ok(s) = utils::tag(TRUE, s) {
            Ok((s, Self(true)))
        } else if let Ok(s) = utils::tag(FALSE, s) {
            Ok((s, Self(false)))
        } else {
            Err(ParseError::ExpectedBool)
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal(pub(crate) Val);

impl Literal {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        Number::new(s)
            .map(|(s, number)| (s, Self(Val::Number(number.0))))
            .or_else(|_| Char::new(s).map(|(s, char_lit)| (s, Self(Val::Char(char_lit.0)))))
            .or_else(|_| Bool::new(s).map(|(s, bool_lit)| (s, Self(Val::Bool(bool_lit.0)))))
            .or_else(|_| {
                StringLiteral::new(s).map(|(s, str_lit)| (s, Self(Val::Deque(Box::new(str_lit.0)))))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env::Env;
    use crate::expr::{BindingUpdate, BindingUsage, Block, Call, Expr, Func};

    #[test]
    fn parse_number() {
        assert_eq!(Number::new("123"), Ok(("", Number(123))));
    }

    #[test]
    fn parse_char() {
        assert_eq!(Char::new("🔡x🔡"), Ok(("", Char('x'))));
        assert_eq!(Char::new("🔡📞🔡"), Ok(("", Char('📞'))));
        assert_eq!(Char::new("🔡💈🔡"), Ok(("", Char('💈'))));
        assert_eq!(
            Char::new("🔡💈y🔡"),
            Err(ParseError::ExpectedTag("🔡".into()))
        );
    }

    #[test]
    fn parse_string() {
        let cases = [
            ("🧵🧵", ""),
            ("🧵x🧵", "x"),
            ("🧵⛷️Ⓜ️🧵", "⛷️Ⓜ️"),
            ("🧵Hello World🧵", "Hello World"),
        ];

        for (input, expected) in cases {
            let str_lit = StringLiteral::new(input);
            let expected = StringLiteral(expected.chars().map(|c| Val::Char(c)).collect());

            assert_eq!(str_lit, Ok(("", expected)))
        }
    }

    #[test]
    fn parse_literal_number() {
        assert_eq!(Literal::new("67892"), Ok(("", Literal(Val::Number(67892)))));
    }

    #[test]
    fn parse_literal_char() {
        assert_eq!(Literal::new("🔡💈🔡"), Ok(("", Literal(Val::Char('💈')))));
    }

    #[test]
    fn parse_literal_true() {
        assert_eq!(Literal::new("🙆‍♀️"), Ok(("", Literal(Val::Bool(true)))));
    }

    #[test]
    fn parse_literal_false() {
        assert_eq!(Literal::new("🙅‍♀️"), Ok(("", Literal(Val::Bool(false)))));
    }

    #[test]
    fn parse_hello_world() {
        let expr_e = Expr::new(
            "📦
            👶 hello = 🧰 ➡️
                🧵Hello World🧵
            🧑‍🦲 💪

            📞 hello
        🧑‍🦲",
        );
        let expected = Expr::Block(Block {
            exprs: vec![
                Expr::BindingUpdate(Box::new(BindingUpdate {
                    name: "hello".to_string(),
                    val: Expr::Func(Box::new(Func {
                        args: Vec::new(),
                        body: Block {
                            exprs: vec![Expr::Literal(Literal(Val::Deque(Box::new(
                                "Hello World".chars().map(|c| Val::Char(c)).collect(),
                            ))))],
                        },
                    })),
                    set: false,
                })),
                Expr::Call(Box::new(Call {
                    func: Expr::BindingUsage(BindingUsage {
                        name: "hello".to_string(),
                    }),
                    args: Vec::new(),
                })),
            ],
        });

        assert_eq!(expr_e, Ok(("", expected)));
    }

    #[test]
    fn eval_hello_world() {
        let (_, expr_e) = Expr::new(
            "📦
            👶 hello = 🧰 ➡️
                🧵Hello World🧵
            🧑‍🦲 💪

            📞 hello
        🧑‍🦲",
        )
        .unwrap();
        let expected = "Hello World".chars().map(|c| Val::Char(c)).collect();

        let mut env = Env::test();
        let result = env.eval(&expr_e);

        assert_eq!(result, Ok(Val::Deque(Box::new(expected))));
    }
}
