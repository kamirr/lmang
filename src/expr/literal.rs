use crate::utils;
use crate::val::Val;

#[derive(Debug, PartialEq)]
struct Number(i32);

impl Number {
    fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let (s, number) = utils::extract_digits(s)?;
        Ok((s, Self(number.parse().unwrap())))
    }
}

#[derive(Debug, PartialEq)]
struct Char(char);

impl Char {
    fn new(s: &str) -> Result<(&str, Self), String> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag("🔡", s)?;
        let c = s.chars().next().ok_or("unexpected eof".to_string())?;
        let s = &s[c.len_utf8()..];
        let s = utils::tag("🔡", s)?;

        Ok((s, Self(c)))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Literal(pub Val);

impl Literal {
    pub fn new(s: &str) -> Result<(&str, Self), String> {
        Number::new(s)
            .map(|(s, number)| (s, Self(Val::Number(number.0))))
            .or_else(|_| Char::new(s).map(|(s, char_lit)| (s, Self(Val::Char(char_lit.0)))))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_number() {
        assert_eq!(Number::new("123"), Ok(("", Number(123))));
    }

    #[test]
    fn parse_char() {
        assert_eq!(Char::new("🔡x🔡"), Ok(("", Char('x'))));
        assert_eq!(Char::new("🔡📞🔡"), Ok(("", Char('📞'))));
        assert_eq!(Char::new("🔡💈🔡"), Ok(("", Char('💈'))));
        assert_eq!(Char::new("🔡💈y🔡"), Err("expected 🔡".to_string()));
    }

    #[test]
    fn parse_literal_number() {
        assert_eq!(Literal::new("67892"), Ok(("", Literal(Val::Number(67892)))));
    }

    #[test]
    fn parse_literal_char() {
        assert_eq!(Literal::new("🔡💈🔡"), Ok(("", Literal(Val::Char('💈')))));
    }
}
