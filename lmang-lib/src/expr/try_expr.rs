use crate::env::{Env, Eval};
use crate::error::ParseError;
use crate::expr::block::{Block, FormatImplicit};
use crate::utils::{self, kwords};
use crate::val::Val;

#[derive(Debug, PartialEq, Clone)]
pub struct Try {
    try_block: Block,
    except_blocks: Vec<(String, Block)>,
    except_any_block: Option<Block>,
}

impl Try {
    pub(crate) fn new(s: &str) -> Result<(&str, Self), ParseError> {
        let (s, _) = utils::extract_whitespace(s);
        let s = utils::tag(kwords::TRY, s)?;

        let (s, body) = Block::implicit(s)?;

        let mut s = s;
        let mut except_blocks = Vec::default();
        let mut except_any_block = None;

        loop {
            let (new_s, _) = utils::extract_whitespace(s);
            let new_s = if let Ok(new_s) = utils::tag(kwords::EXCEPT, new_s) {
                new_s
            } else {
                s = new_s;
                break;
            };

            let (new_s, _) = utils::extract_whitespace(new_s);
            let (new_s, error_type) = match utils::extract_ident(new_s) {
                Ok((new_s, error_type)) => (new_s, error_type),
                Err(_) => {
                    let (new_s, except_block) = Block::implicit(new_s)?;
                    except_any_block = Some(except_block);
                    s = new_s;
                    break;
                }
            };

            let (new_s, except_block) = Block::implicit(new_s)?;

            s = new_s;

            except_blocks.push((error_type.into(), except_block));
        }

        let try_e = Try {
            try_block: body,
            except_blocks,
            except_any_block,
        };

        Ok((s, try_e))
    }
}

impl Eval for Try {
    fn eval(&self, env: &mut Env) -> Result<Val, Val> {
        match env.eval(&self.try_block) {
            Ok(val) => Ok(val),
            Err(err) => {
                for excepts in self.except_blocks.iter() {
                    if excepts.0 == err.as_object()?.0.member("type")?.to_string() {
                        return env.eval(&excepts.1);
                    }
                }
                if let Some(catch_all) = &self.except_any_block {
                    return env.eval(catch_all);
                }

                Err(err)
            }
        }
    }
}

impl crate::expr::Format for Try {
    fn format(&self, w: &mut dyn std::fmt::Write, depth: usize) -> std::fmt::Result {
        writeln!(w, "{}", kwords::TRY)?;
        FormatImplicit(&self.try_block).format(w, depth)?;

        for (exception, body) in &self.except_blocks {
            writeln!(w, " {} {}", kwords::EXCEPT, exception)?;
            FormatImplicit(body).format(w, depth)?;
        }

        if let Some(body_else) = &self.except_any_block {
            writeln!(w, " {}", kwords::EXCEPT)?;
            FormatImplicit(body_else).format(w, depth)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_try_nop() {
        let parse = Try::new("👩‍🚒 🧑‍🦲");
        let expected = Try {
            try_block: Block { exprs: Vec::new() },
            except_blocks: Vec::default(),
            except_any_block: None,
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_except_1() {
        let parse = Try::new("👩‍🚒 🧑‍🦲 🤡 OutOfBounds 🧑‍🦲");
        let blocks = {
            let mut tmp = Vec::default();
            tmp.push(("OutOfBounds".into(), Block { exprs: Vec::new() }));

            tmp
        };
        let expected = Try {
            try_block: Block { exprs: Vec::new() },
            except_blocks: blocks,
            except_any_block: None,
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_except_2() {
        let parse = Try::new("👩‍🚒 🧑‍🦲 🤡 Timeout 🧑‍🦲 🤡 CastError 🧑‍🦲");
        let blocks = {
            let mut tmp = Vec::default();
            tmp.push(("Timeout".into(), Block { exprs: Vec::new() }));
            tmp.push(("CastError".into(), Block { exprs: Vec::new() }));

            tmp
        };
        let expected = Try {
            try_block: Block { exprs: Vec::new() },
            except_blocks: blocks,
            except_any_block: None,
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn parse_except_2_all() {
        let parse = Try::new("👩‍🚒 🧑‍🦲 🤡 Timeout 🧑‍🦲 🤡 CastError 🧑‍🦲 🤡 🧑‍🦲");
        let blocks = {
            let mut tmp = Vec::default();
            tmp.push(("Timeout".into(), Block { exprs: Vec::new() }));
            tmp.push(("CastError".into(), Block { exprs: Vec::new() }));

            tmp
        };
        let expected = Try {
            try_block: Block { exprs: Vec::new() },
            except_blocks: blocks,
            except_any_block: Some(Block { exprs: Vec::new() }),
        };

        assert_eq!(parse, Ok(("", expected)));
    }

    #[test]
    fn reject_default_in_middle() {
        let parse = Try::new("👩‍🚒 🧑‍🦲 🤡 NoKey 🧑‍🦲 🤡 🧑‍🦲 🤡 CastError 🧑‍🦲");

        assert!(!matches!(parse, Err(_)));
    }

    #[test]
    fn test_ok() {
        let mut env = Env::test();
        let (_, parse) =
            Try::new("👩‍🚒 10 🧑‍🦲 🤡 WrongArgsN 🧑‍🦲 🤡 IoError 🧑‍🦲 🤡 🧑‍🦲").unwrap();

        assert_eq!(env.eval(&parse), Ok(Val::Number(10)));
    }

    #[test]
    fn test_invalid_binding() {
        let mut env = Env::test();
        let (_, parse) = Try::new("👩‍🚒 x 🧑‍🦲 🤡 NoBinding 12 🧑‍🦲").unwrap();

        assert_eq!(env.eval(&parse), Ok(Val::Number(12)));
    }

    #[test]
    fn format() {
        let (_, parse) = Try::new("👩‍🚒 x 🧑‍🦲 🤡 NoBinding 12 🧑‍🦲").unwrap();
        let expected = "👩‍🚒\n    x\n🧑‍🦲 🤡 NoBinding\n    12\n🧑‍🦲";

        assert_eq!(format!("{}", crate::expr::Display(&parse)), expected);
    }
}
