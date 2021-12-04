use lmang_lib::{builtins::Builtins, env::Env, error::RuntimeError, expr::Expr, val::Val};
use std::io::Write;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let path = std::env::args().nth(1).expect("no file given");
    let code = std::fs::read_to_string(path).unwrap();

    let (_, expr) = Expr::new(&code).unwrap();

    let mut env = Env::new();
    env.eval(&Builtins::new(
        std::env::args().skip(2),
        Box::new(|s| -> Result<(), RuntimeError> {
            print!("{}", s);

            std::io::stdout()
                .lock()
                .flush()
                .map_err(|e| RuntimeError::IoError {
                    file: "stdout".into(),
                    reason: e.to_string(),
                })?;

            Ok(())
        }),
    ))
    .unwrap();
    let val = env.eval(&expr).unwrap();

    if *val.as_ref() != Val::Unit {
        println!("{}", val.as_ref());
    }
}
