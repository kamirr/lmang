use crate::builtins::rustfn::RustFn;
use crate::error::RuntimeError;
use crate::val::{Object, Val};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct RustObj<T>
where
    T: std::clone::Clone + std::fmt::Debug,
{
    name: String,
    funcs: Vec<RustFn>,
    state: Option<Rc<RefCell<T>>>,
}

impl<T> RustObj<T>
where
    T: 'static + std::clone::Clone + std::fmt::Debug,
{
    pub fn boxed(
        name: impl Into<String>,
        funcs: Vec<RustFn>,
        state: Option<Rc<RefCell<T>>>,
    ) -> Box<Self> {
        let name = name.into();
        Box::new(RustObj { name, funcs, state })
    }
}

impl<T> Object for RustObj<T>
where
    T: 'static + std::clone::Clone + std::fmt::Debug,
{
    fn member_names(&self) -> Vec<String> {
        self.funcs.iter().map(|f| f.name.clone()).collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        for func in self.funcs.iter() {
            if func.name == name {
                return Ok(func.clone().into_val());
            }
        }

        Err(RuntimeError::NoKey(name.into()))
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
