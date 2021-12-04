use lmang_lib::{builtins::Builtins, env::Env, error::RuntimeError, expr::Expr, val::Val};
use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub struct ExecResult {
    pub stdout: String,
    pub return_val: Val,
}

pub fn test_exec(path: String, args: impl Iterator<Item = String> + 'static) -> ExecResult {
    let code = std::fs::read_to_string(path).unwrap();

    let (_, expr) = Expr::new(&code).unwrap();

    let buf = Rc::new(RefCell::new(String::new()));
    let buf_inner = buf.clone();

    let mut env = Env::new();
    env.eval(&Builtins::new(
        args,
        Box::new(move |s| -> Result<(), RuntimeError> {
            write!(&mut buf_inner.borrow_mut(), "{}", s).map_err(|e| RuntimeError::IoError {
                file: "stdout".into(),
                reason: e.to_string(),
            })?;

            Ok(())
        }),
    ))
    .unwrap();
    let val = env.eval(&expr).unwrap();

    let borrow = buf.borrow();
    ExecResult {
        stdout: borrow.to_string(),
        return_val: val.as_ref().clone(),
    }
}
