use crate::builtins::objects::rustobj::RustObj;
use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::Val;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct FileState {
    files: Rc<RefCell<HashMap<i32, File>>>,
    cnt: i32,
}

fn open(args: &[Val], _env: &mut Env, state: FnState) -> Result<Val, String> {
    let fname_dq: VecDeque<_> =
        args[0].apply_to_root(|v| -> Result<_, String> { Ok(v.as_deque()?.clone()) })??;
    let fname_res: Result<String, _> = fname_dq
        .into_iter()
        .map(|v| v.as_char().map(|c| *c))
        .collect();
    let fname = fname_res?;
    let file = File::open(&fname).map_err(|_| "cannot open file".to_string())?;

    let mut borrow = state.0.borrow_mut();
    let fstate: &mut FileState = borrow.downcast_mut::<FileState>().unwrap();

    let id = fstate.cnt;
    fstate.cnt += 1;

    let mut files_borrow = fstate.files.borrow_mut();
    files_borrow.insert(id, file);

    Ok(Val::Number(id))
}

fn read(args: &[Val], _env: &mut Env, state: FnState) -> Result<Val, String> {
    let mut borrow = state.0.borrow_mut();
    let fstate: &mut FileState = borrow.downcast_mut::<FileState>().unwrap();

    let id = args[0].apply_to_root(|v| v.as_number().map(|n| *n))??;
    let files = fstate.files.borrow_mut();
    let mut file = files.get(&id).ok_or_else(|| "no such file".to_string())?;

    let mut buf = String::new();
    file.read_to_string(&mut buf);

    let deque = buf.chars().map(Val::Char).collect();

    Ok(Val::Deque(Box::new(deque)))
}

pub fn make_file_builtin() -> Box<RustObj<FileState>> {
    let files = Rc::new(RefCell::new(Default::default()));
    let state = Rc::new(RefCell::new(FileState { files, cnt: 0 }));

    RustObj::boxed(
        "file",
        vec![
            RustFn::stateful("open", open, &state),
            RustFn::stateful("read", read, &state),
        ],
        Some(state),
    )
}
