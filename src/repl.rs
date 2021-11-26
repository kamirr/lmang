use std::io::BufRead;

use lmang_lib::{env::Env, stmt::Stmt};

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
                match Stmt::new(&input[..]) {
                    Ok((_, stmt)) => {
                        let res = env.eval(&stmt)?;
                        println!("{:?}", res);
                        input.clear();
                    }
                    Err(_) => continue,
                }
            } else {
                match Stmt::new(&line[..]) {
                    Ok((_, stmt)) => {
                        let res = env.eval(&stmt);
                        println!("{:?}", res);
                    }
                    Err(_) => {
                        input.push_str(&line);
                    }
                }
            }
        } else {
            let stmt = match Stmt::new(&input[..]) {
                Ok((_, stmt)) => stmt,
                Err(e) => {
                    println!("{}", e);
                    continue;
                }
            };
            let res = env.eval(&stmt)?;
            println!("{:?}", res);

            input.clear();
        }
    }
}
