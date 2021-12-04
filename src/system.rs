use crate::error::RuntimeError;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Write as _;
use std::io::BufRead as _;
use std::io::Write as _;
use std::rc::Rc;

pub trait System {
    fn args(&self) -> Box<dyn Iterator<Item = String>>;
    fn print(&self) -> Box<dyn FnMut(String) -> Result<(), RuntimeError>>;
    fn read(&self) -> Box<dyn FnMut() -> Result<String, RuntimeError>>;
}

pub struct Native {
    skip_n_args: usize,
}

impl Native {
    pub fn new(skip_n_args: usize) -> Self {
        Native { skip_n_args }
    }
}

impl System for Native {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(std::env::args().skip(self.skip_n_args))
    }

    fn print(&self) -> Box<dyn FnMut(String) -> Result<(), RuntimeError>> {
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
        })
    }

    fn read(&self) -> Box<dyn FnMut() -> Result<String, RuntimeError>> {
        Box::new(|| -> Result<String, RuntimeError> {
            let mut line = String::new();
            let stdin = std::io::stdin();
            stdin
                .lock()
                .read_line(&mut line)
                .map_err(|e| RuntimeError::IoError {
                    file: "stdin".into(),
                    reason: e.to_string(),
                })?;

            Ok(line)
        })
    }
}

pub struct Test {
    args: Vec<String>,
    stdout: Rc<RefCell<String>>,
    stdin: Rc<RefCell<VecDeque<String>>>,
}

pub struct TestSystemOutput {
    pub stdout: Rc<RefCell<String>>,
}

impl Test {
    pub fn new(args: &[String], stdin: &[String]) -> (Self, TestSystemOutput) {
        let stdout = Rc::new(RefCell::new(String::new()));

        (
            Test {
                args: args.into(),
                stdout: stdout.clone(),
                stdin: Rc::new(RefCell::new(stdin.iter().cloned().collect())),
            },
            TestSystemOutput { stdout },
        )
    }
}

impl System for Test {
    fn args(&self) -> Box<dyn Iterator<Item = String>> {
        Box::new(self.args.clone().into_iter())
    }

    fn print(&self) -> Box<dyn FnMut(String) -> Result<(), RuntimeError>> {
        let buf = self.stdout.clone();

        Box::new(move |s| -> Result<(), RuntimeError> {
            write!(&mut buf.borrow_mut(), "{}", s).map_err(|e| RuntimeError::IoError {
                file: "stdout".into(),
                reason: e.to_string(),
            })?;

            Ok(())
        })
    }

    fn read(&self) -> Box<dyn FnMut() -> Result<String, RuntimeError>> {
        let stdin = self.stdin.clone();
        Box::new(move || -> Result<String, RuntimeError> {
            let mut borrow = stdin.borrow_mut();
            let line = borrow.pop_front().ok_or_else(|| RuntimeError::IoError {
                file: "stdin".into(),
                reason: "no more lines".to_string(),
            })?;

            Ok(line)
        })
    }
}
