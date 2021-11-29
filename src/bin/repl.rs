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

        let maybe_res: Option<_> = if !line.trim().is_empty() {
            if !input.is_empty() {
                input.push_str(&line);

                match Expr::new(&input[..]) {
                    Ok((_, expr)) => {
                        let res = env.eval(&expr);
                        input.clear();

                        Some(res)
                    }
                    Err(_) => None,
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
                    input.clear();
                    Some(Err(e))
                }
            }
        };

        if let Some(res) = maybe_res {
            let res = res.map(|cow| cow.into_owned());
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
