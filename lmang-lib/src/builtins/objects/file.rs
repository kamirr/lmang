use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::view::{self, test_consumed, view1};
use crate::val::Val;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FileState {
    files: Rc<RefCell<HashMap<i32, File>>>,
    cnt: i32,
}

fn open(args: &mut [Val], _env: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let (fname, tail) = view1::<view::AnyRef<view::Deque>, _, _>(args, |dq| {
        let string = dq
            .iter()
            .map(|v| v.as_char().map(|c| *c))
            .collect::<Result<String, _>>()?;
        Ok(string)
    })?;
    test_consumed(tail)?;

    let file = File::open(&fname).map_err(|e| RuntimeError::IoError {
        file: fname.clone(),
        reason: e.to_string(),
    })?;

    let mut borrow = state.0.borrow_mut();
    let fstate: &mut FileState = borrow.downcast_mut::<FileState>().unwrap();

    let id = fstate.cnt;
    fstate.cnt += 1;

    let mut files_borrow = fstate.files.borrow_mut();
    files_borrow.insert(id, file);

    Ok(Val::Number(id))
}

fn read(args: &mut [Val], _env: &mut Env, state: FnState) -> Result<Val, RuntimeError> {
    let (id, tail) = view1::<view::Number, _, _>(args, |n| Ok(*n))?;
    test_consumed(tail)?;

    let mut borrow = state.0.borrow_mut();
    let fstate: &mut FileState = borrow.downcast_mut::<FileState>().unwrap();

    let files = fstate.files.borrow_mut();
    let mut file = files.get(&id).ok_or_else(|| RuntimeError::NoHandle(id))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .map_err(|e| RuntimeError::IoError {
            file: format!("handle({})", id),
            reason: e.to_string(),
        })?;

    let deque = buf.chars().map(Val::Char).collect();

    Ok(Val::Deque(Box::new(deque)))
}

pub(crate) fn make_file_builtin() -> Box<RustObj> {
    let files = Rc::new(RefCell::new(Default::default()));
    let state = Rc::new(RefCell::new(FileState { files, cnt: 0 }));

    RustObj::boxed(
        "file",
        vec![
            RustFn::stateful("open", open, &state),
            RustFn::stateful("read", read, &state),
        ],
    )
}
