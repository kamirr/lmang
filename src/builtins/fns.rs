use super::{FnState, RustFn};
use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::val::Val;
use std::borrow::Cow;
use std::io::{BufRead, Write};

fn print(args: &[Val], _env: &mut Env, _: FnState) -> Result<Val, RuntimeError> {
    if !args.is_empty() {
        let mut args = &args[..];
        let mut sep = None;
        let mut end = None;
        while let Ok(Some((name, val))) = args[0].apply_to_root(|v| {
            if let Val::Named((s, inner)) = v {
                Some((s.clone(), inner.clone()))
            } else {
                None
            }
        }) {
            args = &args[1..];
            if name == "sep" {
                sep = Some(format!("{}", val));
            } else if name == "end" {
                end = Some(format!("{}", val));
            }
        }

        let args = args;
        let sep = sep.unwrap_or_else(|| " ".to_string());
        let end = end.unwrap_or_else(|| "\n".to_string());

        for arg in &args[0..args.len() - 1] {
            print!("{}{}", arg, sep);
        }
        print!("{}{}", args.last().unwrap(), end);
    } else {
        println!("");
    }
    std::io::stdout()
        .lock()
        .flush()
        .map_err(|e| RuntimeError::IoError {
            file: "stdout".into(),
            reason: e.to_string(),
        })?;

    Ok(Val::Unit)
}

fn read(_args: &[Val], _env: &mut Env, _: FnState) -> Result<Val, RuntimeError> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).unwrap();

    let deque = line
        .strip_suffix('\n')
        .unwrap()
        .chars()
        .map(Val::Char)
        .collect();
    Ok(Val::Deque(Box::new(deque)))
}

pub struct BuiltinFns;

impl Eval for BuiltinFns {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, RuntimeError> {
        env.store_binding("🗣️".to_string(), RustFn::new("print", print).into_val());
        env.store_binding("👂".to_string(), RustFn::new("read", read).into_val());

        Ok(Cow::Owned(Val::Unit))
    }
}
