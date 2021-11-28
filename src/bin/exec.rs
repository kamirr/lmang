use lmang_lib::{builtins::Builtins, env::Env, expr::Expr, val::Val};

fn main() {
    let path = std::env::args().nth(1).expect("no file given");
    let code = std::fs::read_to_string(path).unwrap();

    let (_, expr) = Expr::new(&code).unwrap();

    let mut env = Env::new();
    env.eval(&Builtins).unwrap();
    let val = env.eval(&expr).unwrap();

    if val != Val::Unit {
        println!("{}", val);
    }
}