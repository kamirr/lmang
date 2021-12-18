use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::Val;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub(crate) struct SysState {
    args: Vec<VecDeque<Val>>,
}

fn get_args(_args: &[Val], _env: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let mut borrow = state.0.borrow_mut();
    let sys: &SysState = borrow.downcast_mut::<SysState>().unwrap();

    let res_deque = sys
        .args
        .iter()
        .cloned()
        .map(Box::new)
        .map(Val::Deque)
        .collect();
    Ok(Val::Deque(Box::new(res_deque)))
}

pub(crate) fn make_sys_builtin(args: impl Iterator<Item = String>) -> Box<RustObj> {
    let args = args.map(|s| s.chars().map(Val::Char).collect()).collect();
    let state = Rc::new(RefCell::new(SysState { args }));

    RustObj::boxed("sys", vec![RustFn::stateful("args", get_args, &state)])
}
