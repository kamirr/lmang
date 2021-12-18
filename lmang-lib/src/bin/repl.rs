use std::io::BufRead;
use std::io::Write;

use lmang_lib::{builtins::Builtins, env::Env, error::Error, expr::Expr, system, val::Val};

fn main() -> Result<(), String> {
    let mut env = Env::new();
    env.eval(&Builtins::new(system::Native::new(1))).unwrap();

    let mut prompt = "✅";
    let mut input = String::new();

    loop {
        let mut line = String::new();

        print!("{} ", prompt);
        std::io::stdout().lock().flush().unwrap();
        let bytes_read = std::io::stdin().lock().read_line(&mut line).unwrap();

        if bytes_read == 0 {
            // EOF
            break Ok(());
        }

        let maybe_res: Option<Result<_, Error>> = if !line.trim().is_empty() {
            if !input.is_empty() {
                input.push_str(&line);

                match Expr::new(&input[..]) {
                    Ok((_, expr)) => {
                        let res = env.eval(&expr);
                        input.clear();

                        Some(res.map_err(|err| err.into()))
                    }
                    Err(_) => None,
                }
            } else {
                match Expr::new(&line[..]) {
                    Ok((_, expr)) => Some(env.eval(&expr).map_err(|err| err.into())),
                    Err(_) => {
                        input.push_str(&line);

                        None
                    }
                }
            }
        } else {
            match Expr::new(&input[..]) {
                Ok((_, expr)) => {
                    let res = env.eval(&expr);
                    input.clear();

                    Some(res.map_err(|err| err.into()))
                }
                Err(e) => {
                    input.clear();
                    Some(Err(e.into()))
                }
            }
        };

        if let Some(res) = maybe_res {
            if let Ok(Val::Unit) = res {
                prompt = "✅";
            } else {
                match res {
                    Ok(v) => {
                        prompt = "✅";
                        println!("{}", v);
                    }
                    Err(e) => {
                        prompt = "❌";
                        println!("{}", e);
                    }
                }
            }
        } else {
            prompt = "😵‍💫";
        }
    }
}
