use super::{FnState, RustFn};
use crate::env::{Env, Eval};
use crate::error::RuntimeError;
use crate::val::{
    view::{self, test_consumed, view1},
    Val,
};
use std::cell::RefCell;
use std::fmt::Write as _;
use std::rc::Rc;

pub(crate) type PrintImpl = Box<dyn FnMut(String) -> Result<(), RuntimeError>>;
pub(crate) type ReadImpl = Box<dyn FnMut() -> Result<String, RuntimeError>>;

fn print(args: &mut [Val], _env: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let mut borrow = state.0.borrow_mut();
    let print_impl: &mut PrintImpl = borrow.downcast_mut().unwrap();

    let mut buf = String::new();
    if !args.is_empty() {
        let mut args = &args[..];
        let mut sep = None;
        let mut end = None;
        while let Ok(Some((name, val))) = args[0].apply_to_root(|v| {
            if let Val::Named((s, inner)) = v {
                Some((s.clone(), inner.clone()))
            } else {
                None
            }
        }) {
            args = &args[1..];
            if name == "sep" {
                sep = Some(format!("{}", val));
            } else if name == "end" {
                end = Some(format!("{}", val));
            }
        }

        let args = args;
        let sep = sep.unwrap_or_else(|| " ".to_string());
        let end = end.unwrap_or_else(|| "\n".to_string());

        for arg in &args[0..args.len() - 1] {
            write!(&mut buf, "{}{}", arg, sep).map_err(|e| RuntimeError::IoError {
                file: "stdout".into(),
                reason: e.to_string(),
            })?;
        }
        write!(&mut buf, "{}{}", args.last().unwrap(), end).map_err(|e| RuntimeError::IoError {
            file: "stdout".into(),
            reason: e.to_string(),
        })?;
    } else {
        write!(&mut buf, "").map_err(|e| RuntimeError::IoError {
            file: "stdout".into(),
            reason: e.to_string(),
        })?;
    }

    (*print_impl)(buf)?;

    Ok(Val::Unit)
}

fn read(_args: &mut [Val], _env: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let mut borrow = state.0.borrow_mut();
    let read_impl: &mut ReadImpl = borrow.downcast_mut().unwrap();

    let line = read_impl()?;
    let deque = line
        .strip_suffix('\n')
        .unwrap_or(&line[..])
        .chars()
        .map(Val::Char)
        .collect();
    Ok(Val::Deque(Box::new(deque)))
}

fn eval(args: &mut [Val], env: &mut Env, _state: FnState) -> Result<Val, RuntimeError> {
    let (code, tail) = view1::<view::String, _, _>(args, |s| Ok(s.clone()))?;
    test_consumed(tail)?;

    let (_, expr) = crate::expr::Expr::new(&code).map_err(|_e| RuntimeError::CastError {
        from: "string".to_string(),
        to: "code".to_string(),
    })?;

    env.eval(&expr)
}

pub(crate) struct BuiltinFns {
    print_impl: Rc<RefCell<PrintImpl>>,
    read_impl: Rc<RefCell<ReadImpl>>,
}

impl BuiltinFns {
    pub(crate) fn new(print_impl: PrintImpl, read_impl: ReadImpl) -> Self {
        BuiltinFns {
            print_impl: Rc::new(RefCell::new(print_impl)),
            read_impl: Rc::new(RefCell::new(read_impl)),
        }
    }
}

impl Eval for BuiltinFns {
    fn eval(&self, env: &mut Env) -> Result<Val, RuntimeError> {
        env.store_binding(
            "🗣️".to_string(),
            RustFn::stateful("print", print, &self.print_impl).into_val(),
        );
        env.store_binding(
            "👂".to_string(),
            RustFn::stateful("read", read, &self.read_impl).into_val(),
        );
        env.store_binding("🪞".to_string(), RustFn::new("eval", eval).into_val());

        Ok(Val::Unit)
    }
}
