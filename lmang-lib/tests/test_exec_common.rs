use lmang_lib::{builtins::Builtins, env::Env, expr::Expr, system, val::Val};

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub struct ExecResult {
    pub stdout: String,
    pub return_val: Result<Val, Val>,
}

pub fn test_exec(path: String, args: &[String], stdin: &[String]) -> ExecResult {
    let code = std::fs::read_to_string(path).unwrap();

    let (_, expr) = Expr::new(&code).unwrap();

    let mut env = Env::new();
    let (system, system_out) = system::Test::new(args, stdin);
    env.eval(&Builtins::new(system)).unwrap();
    let val = env.eval(&expr);

    let borrow = system_out.stdout.borrow();
    ExecResult {
        stdout: borrow.to_string(),
        return_val: val.map(|v| v),
    }
}
