use crate::error::ParseError;

pub mod kwords {
    pub const BLOCK_OPEN: &str = "๐ฆ";
    pub const BLOCK_CLOSE: &str = "๐งโ๐ฆฒ";
    pub const EXPR_SEP: &str = "๐ช";

    pub const IF: &str = "โ";
    pub const ELIF: &str = "๐ ";
    pub const ELSE: &str = "๐ก";

    pub const BREAK: &str = "๐";
    pub const LOOP: &str = "๐";

    pub const LET: &str = "๐ถ";
    pub const SET: &str = "โป๏ธ";
    pub const GLOB: &str = "๐";
    pub const UPDATE_SEP: &str = "=";

    pub const TRUE: &str = "๐โโ๏ธ";
    pub const FALSE: &str = "๐โโ๏ธ";

    pub const FUNC: &str = "๐งฐ";
    pub const VARIADIC: &str = "๐จโ๐จโ๐ฆ";
    pub const FUNC_SEP: &str = "โก๏ธ";
    pub const CALL: &str = "๐";

    pub const CHAR_LIT: &str = "๐ก";
    pub const STR_LIT: &str = "๐งต";

    pub const INDEX: &str = "๐ช";
    pub const REF: &str = "๐";

    pub const CLASS: &str = "๐งโ๐ซ";

    pub const NAMED: &str = ":";

    pub const TRY: &str = "๐ฉโ๐";
    pub const EXCEPT: &str = "๐คก";

    pub const ADD: &str = "+";
    pub const SUB: &str = "-";
    pub const MUL: &str = "*";
    pub const DIV: &str = "/";

    pub const GT: &str = ">";
    pub const GE: &str = ">=";
    pub const EQ: &str = "==";
    pub const LE: &str = "<=";
    pub const LT: &str = "<";
    pub const NE: &str = "!=";
    pub const FE: &str = "~=";

    pub const ALL: [&str; 37] = [
        BLOCK_OPEN,
        BLOCK_CLOSE,
        EXPR_SEP,
        IF,
        ELIF,
        ELSE,
        BREAK,
        LOOP,
        LET,
        SET,
        GLOB,
        UPDATE_SEP,
        TRUE,
        FALSE,
        FUNC,
        VARIADIC,
        FUNC_SEP,
        CALL,
        CHAR_LIT,
        STR_LIT,
        INDEX,
        REF,
        CLASS,
        NAMED,
        TRY,
        EXCEPT,
        ADD,
        SUB,
        MUL,
        DIV,
        GT,
        GE,
        EQ,
        LE,
        LT,
        NE,
        FE,
    ];
}

pub(crate) fn take_while(accept: impl Fn(char) -> bool, s: &str) -> (&str, &str) {
    let extracted_end = s
        .char_indices()
        .find_map(|(idx, c)| if accept(c) { None } else { Some(idx) })
        .unwrap_or_else(|| s.len());

    let extracted = &s[..extracted_end];
    let remainder = &s[extracted_end..];
    (remainder, extracted)
}

#[allow(unused)]
pub(crate) fn take_while1(
    accept: impl Fn(char) -> bool,
    s: &str,
    error: ParseError,
) -> Result<(&str, &str), ParseError> {
    let (remainder, extracted) = take_while(accept, s);

    if extracted.is_empty() {
        Err(error)
    } else {
        Ok((remainder, extracted))
    }
}

pub(crate) fn extract_digits(s: &str) -> Result<(&str, &str), ParseError> {
    take_while1(|c| c.is_ascii_digit(), s, ParseError::ExpectedDigits)
}

fn is_whitespace(c: char) -> bool {
    c.is_ascii_whitespace() || c == '\n'
}

pub(crate) fn extract_whitespace(s: &str) -> (&str, &str) {
    take_while(is_whitespace, s)
}

#[allow(unused)]
pub(crate) fn extract_whitespace1(s: &str) -> Result<(&str, &str), ParseError> {
    take_while1(is_whitespace, s, ParseError::ExpectedWhitespace)
}

pub(crate) fn extract_ident(s: &str) -> Result<(&str, &str), ParseError> {
    let not_ident_err = Err(ParseError::ExpectedIdent);
    if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true) {
        return not_ident_err;
    }
    for reserved in kwords::ALL {
        if s.starts_with(reserved) {
            return not_ident_err;
        }
    }

    let mut bytes = 0;
    for c in s.chars() {
        if c.is_whitespace() {
            break;
        }

        let mut stop_ident_read = false;
        for reserved in kwords::ALL {
            if s[bytes..].starts_with(reserved) {
                stop_ident_read = true;
            }
        }
        if stop_ident_read {
            break;
        }

        bytes += c.len_utf8();
    }

    if bytes == 0 {
        not_ident_err
    } else {
        Ok((&s[bytes..], &s[0..bytes]))
    }
}

pub(crate) fn tag<'a>(starting_text: &'static str, s: &'a str) -> Result<&'a str, ParseError> {
    if let Some(stripped) = s.strip_prefix(starting_text) {
        Ok(stripped)
    } else {
        Err(ParseError::ExpectedTag(starting_text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_one_digit() {
        assert_eq!(extract_digits("1+2"), Ok(("+2", "1")));
    }

    #[test]
    fn extract_multiple_digits() {
        assert_eq!(extract_digits("10-20"), Ok(("-20", "10")));
    }

    #[test]
    fn do_not_extract_digits_when_input_is_invalid() {
        assert_eq!(extract_digits("abcd"), Err(ParseError::ExpectedDigits));
    }

    #[test]
    fn extract_digits_with_no_remainder() {
        assert_eq!(extract_digits("100"), Ok(("", "100")));
    }

    #[test]
    fn extract_spaces() {
        assert_eq!(extract_whitespace("    1"), ("1", "    "));
    }

    #[test]
    fn do_not_extract_spaces1_when_input_does_not_start_with_them() {
        assert_eq!(
            extract_whitespace1("blah"),
            Err(ParseError::ExpectedWhitespace),
        );
    }

    #[test]
    fn extract_alphabetic_ident() {
        assert_eq!(extract_ident("abcdEFG stop"), Ok((" stop", "abcdEFG")));
    }

    #[test]
    fn extract_alphanumeric_ident() {
        assert_eq!(extract_ident("abc123 stop"), Ok((" stop", "abc123")));
    }

    #[test]
    fn extract_reject_empty_ident() {
        assert_eq!(extract_ident("  "), Err(ParseError::ExpectedIdent));
        assert_eq!(extract_ident(""), Err(ParseError::ExpectedIdent));
    }

    #[test]
    fn extract_fireworks_ident() {
        assert_eq!(extract_ident("a๐c stop"), Ok((" stop", "a๐c")));
    }

    #[test]
    fn cannot_extract_ident_beginning_with_number() {
        assert_eq!(extract_ident("123abc"), Err(ParseError::ExpectedIdent),);
    }

    #[test]
    fn tag_word() {
        assert_eq!(tag("๐ถ", "๐ถ a"), Ok(" a"));
    }
}
