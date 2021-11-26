use std::io::BufRead;

use lmang_lib::{env::Env, expr::Expr};

fn main() -> Result<(), String> {
    let mut env = Env::new();

    let mut input = String::new();
    let stdin = std::io::stdin();
    loop {
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();

        if line.trim().len() != 0 {
            if input.len() > 0 {
                input.push_str(&line);
                match Expr::new(&input[..]) {
                    Ok((_, expr)) => {
                        let res = env.eval(&expr)?;
                        println!("{:?}", res);
                        input.clear();
                    }
                    Err(_) => continue,
                }
            } else {
                match Expr::new(&line[..]) {
                    Ok((_, expr)) => {
                        let res = env.eval(&expr);
                        println!("{:?}", res);
                    }
                    Err(_) => {
                        input.push_str(&line);
                    }
                }
            }
        } else {
            let expr = match Expr::new(&input[..]) {
                Ok((_, expr)) => expr,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };
            let res = env.eval(&expr)?;
            println!("{:?}", res);

            input.clear();
        }
    }
}
