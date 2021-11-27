use crate::env::{Env, Eval};
use crate::val::Val;

use super::{FnState, RustFn};

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

pub struct BuiltinFns;

impl Eval for BuiltinFns {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        env.store_binding("🗣️".to_string(), RustFn::new("🗣️__print", print).into_val());

        Ok(Val::Unit)
    }
}
