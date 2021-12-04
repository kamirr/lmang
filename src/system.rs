use crate::error::RuntimeError;
use std::cell::RefCell;
use std::fmt::Write as _;
use std::io::Write as _;
use std::rc::Rc;

pub trait System {
    fn args(&self) -> Box<dyn Iterator<Item = String>>;
    fn print(&self) -> Box<dyn FnMut(String) -> Result<(), RuntimeError>>;
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
}

pub struct Test {
    args: Vec<String>,
    stdout: Rc<RefCell<String>>,
}

pub struct TestSystemOutput {
    pub stdout: Rc<RefCell<String>>,
}

impl Test {
    pub fn new(args: &[String]) -> (Self, TestSystemOutput) {
        let stdout = Rc::new(RefCell::new(String::new()));

        (
            Test {
                args: args.into(),
                stdout: stdout.clone(),
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
}
