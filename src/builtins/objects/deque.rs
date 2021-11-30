use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::{Object, Val};
use std::borrow::Borrow;
use std::fmt;

#[derive(Clone, Debug)]
pub struct DequeBuiltin {
    fns: Vec<RustFn>,
}

impl DequeBuiltin {
    pub fn boxed() -> Box<Self> {
        let fns = vec![
            RustFn::new("len", DequeBuiltin::len),
            RustFn::new("append", DequeBuiltin::append),
            RustFn::new("at", DequeBuiltin::at),
            RustFn::new("mut", DequeBuiltin::at_mut),
        ];

        Box::new(DequeBuiltin { fns })
    }

    fn len(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, String> {
        let len = args[0].apply_to_root(|v| -> Result<_, String> { Ok(v.as_deque()?.len()) })??;

        Ok(Val::Number(len as i32))
    }

    fn append(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("invalid number of arguments".to_string());
        }

        let val = args[1].clone();
        args[0]
            .as_val_ref()
            .borrow()
            .as_ref()
            .unwrap()
            .borrow_mut()
            .apply_to_root_mut(|v| -> Result<_, String> {
                v.as_deque_mut()?.push_back(val);
                Ok(())
            })??;

        Ok(Val::Unit)
    }

    fn at(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("invalid number of arguments".to_string());
        }

        let idx = args[1].apply_to_root(|v| v.as_number().map(|n| *n))??;
        let result = args[0].apply_to_root(|v| -> Result<_, String> {
            Ok(v.as_deque()?[idx as usize].clone())
        })??;

        Ok(result)
    }

    fn at_mut(args: &[Val], _env: &mut Env, _state: FnState) -> Result<Val, String> {
        if args.len() != 2 {
            return Err("invalid number of arguments".to_string());
        }

        let idx = args[1].apply_to_root(|v| v.as_number().map(|n| *n))??;
        let result = match args[0].as_val_ref().borrow().as_ref() {
            Ok(vr) => vr
                .borrow_mut()
                .apply_to_root_mut(|v| -> Result<_, String> {
                    let val_ref = v.as_deque_mut()?[idx as usize].make_ref();

                    Ok(val_ref)
                })??,
            _ => args[0].as_deque()?[idx as usize].clone(),
        };

        Ok(result)
    }
}

impl Object for DequeBuiltin {
    fn member_names(&self) -> Vec<String> {
        self.fns.iter().map(|f| f.name.clone()).collect()
    }

    fn member(&self, name: &str) -> Result<Val, String> {
        for func in self.fns.iter() {
            if func.name == name {
                return Ok(func.clone().into_val());
            }
        }

        Err(format!("no member {}", name))
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }

    fn name(&self) -> &str {
        "deque"
    }
}
