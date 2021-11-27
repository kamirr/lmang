use std::io::BufRead;
use std::io::Write;

use lmang_lib::{builtins::Builtins, env::Env, expr::Expr, val::Val};

fn main() -> Result<(), String> {
    let mut env = Env::new();
    env.eval(&Builtins)?;

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

        let maybe_res: Option<_> = if line.trim().len() != 0 {
            if input.len() > 0 {
                match Expr::new(&input[..]) {
                    Ok((_, expr)) => {
                        let res = env.eval(&expr);
                        input.clear();

                        Some(res)
                    }
                    Err(_) => {
                        input.push_str(&line);

                        None
                    }
                }
            } else {
                match Expr::new(&line[..]) {
                    Ok((_, expr)) => Some(env.eval(&expr)),
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

                    Some(res)
                }
                Err(e) => {
                    println!("{}", e);
                    input.clear();
                    prompt = "❌";

                    None
                }
            }
        };

        if let Some(res) = maybe_res {
            if let Ok(Val::Unit) = res {
            } else {
                if let Ok(_) = &res {
                    prompt = "✅";
                } else {
                    prompt = "❌";
                }
                println!("{:?}", res);
            }
        } else {
            prompt = "😵‍💫";
        }
    }
}
