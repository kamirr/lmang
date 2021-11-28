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
    error_msg: String,
) -> Result<(&str, &str), String> {
    let (remainder, extracted) = take_while(accept, s);

    if extracted.is_empty() {
        Err(error_msg)
    } else {
        Ok((remainder, extracted))
    }
}

pub(crate) fn extract_digits(s: &str) -> Result<(&str, &str), String> {
    take_while1(|c| c.is_ascii_digit(), s, "expected digits".to_string())
}

fn is_whitespace(c: char) -> bool {
    c.is_ascii_whitespace() || c == '\n'
}

pub(crate) fn extract_whitespace(s: &str) -> (&str, &str) {
    take_while(|c| is_whitespace(c), s)
}

#[allow(unused)]
pub(crate) fn extract_whitespace1(s: &str) -> Result<(&str, &str), String> {
    take_while1(|c| is_whitespace(c), s, "expected a space".to_string())
}

pub(crate) fn extract_ident(s: &str) -> Result<(&str, &str), String> {
    let reserved = [
        "📦",
        "🧑‍🦲",
        "💪",
        "❓",
        "😠",
        "😡",
        "💔",
        "🔁",
        "👶",
        "set",
        "=",
        "+",
        "-",
        "*",
        "/",
        ">",
        ">=",
        "==",
        "<=",
        "<",
        "!=",
    ];

    let not_ident_err = Err("expected identifier".to_string());
    if s.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(true) {
        return not_ident_err;
    }
    for r in reserved {
        if s.starts_with(r) {
            return not_ident_err;
        }
    }

    let mut bytes = 0;
    let mut chars = 0;
    loop {
        let c = match s.chars().skip(chars).next() {
            Some(c) => c,
            None => break,
        };

        if c.is_whitespace() {
            break;
        }

        let mut stop_ident_read = false;
        for r in reserved {
            if s[bytes..].starts_with(r) {
                stop_ident_read = true;
            }
        }
        if stop_ident_read {
            break;
        }

        bytes += c.len_utf8();
        chars += 1;
    }

    Ok((&s[bytes..], &s[0..bytes]))
}

pub(crate) fn tag<'a, 'b>(starting_text: &'a str, s: &'b str) -> Result<&'b str, String> {
    if s.starts_with(starting_text) {
        Ok(&s[starting_text.len()..])
    } else {
        Err(format!("expected {}", starting_text))
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
        assert_eq!(extract_digits("abcd"), Err("expected digits".to_string()));
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
            Err("expected a space".to_string()),
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
    fn extract_fireworks_ident() {
        assert_eq!(extract_ident("a🎆c stop"), Ok((" stop", "a🎆c")));
    }

    #[test]
    fn cannot_extract_ident_beginning_with_number() {
        assert_eq!(
            extract_ident("123abc"),
            Err("expected identifier".to_string()),
        );
    }

    #[test]
    fn tag_word() {
        assert_eq!(tag("👶", "👶 a"), Ok(" a"));
    }
}
