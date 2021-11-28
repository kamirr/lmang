use super::{FnState, RustFn};
use crate::env::{Env, Eval};
use crate::val::Val;
use std::borrow::Cow;
use std::io::BufRead;

fn print(args: &[Val], _env: &mut Env, _: FnState) -> Result<Val, String> {
    if args.len() > 0 {
        for arg in &args[0..args.len() - 1] {
            print!("{} ", arg);
        }
        println!("{}", args.last().unwrap());
    } else {
        println!("");
    }

    Ok(Val::Unit)
}

fn read(_args: &[Val], _env: &mut Env, _: FnState) -> Result<Val, String> {
    let mut line = String::new();
    std::io::stdin().lock().read_line(&mut line).unwrap();

    let deque = line.chars().map(|c| Val::Char(c)).collect();
    Ok(Val::Deque(deque))
}

pub struct BuiltinFns;

impl Eval for BuiltinFns {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String> {
        env.store_binding("🗣️".to_string(), RustFn::new("print", print).into_val());
        env.store_binding("👂".to_string(), RustFn::new("read", read).into_val());

        Ok(Cow::Owned(Val::Unit))
    }
}
