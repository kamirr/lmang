use crate::error::RuntimeError;
use crate::val::Val;
use std::any::Any;
use std::fmt;

pub trait Callee {
    fn call(&self, args: &[Val], env: &mut crate::env::Env) -> Result<Val, RuntimeError>;
    fn clone_box(&self) -> Box<dyn Callee>;
    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn dyn_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct DynFunc(pub Box<dyn Callee>);

impl Clone for DynFunc {
    fn clone(&self) -> Self {
        DynFunc(self.0.clone_box())
    }
}

impl PartialEq for DynFunc {
    fn eq(&self, other: &DynFunc) -> bool {
        if self.type_id() != other.type_id() {
            false
        } else {
            format!("{:?}", self) == format!("{:?}", other)
        }
    }
}

impl fmt::Debug for DynFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_debug(f)
    }
}

impl fmt::Display for DynFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_display(f)
    }
}

pub fn placeholder_func() -> DynFunc {
    struct ImplDetail;

    impl Callee for ImplDetail {
        fn call(&self, _args: &[Val], _env: &mut crate::env::Env) -> Result<Val, RuntimeError> {
            unreachable!()
        }
        fn clone_box(&self) -> Box<dyn Callee> {
            unreachable!()
        }
        fn dyn_debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            unreachable!()
        }
        fn dyn_display(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            unreachable!()
        }
        fn as_any_mut(&mut self) -> &mut dyn Any {
            unreachable!()
        }
    }

    DynFunc(Box::new(ImplDetail))
}
