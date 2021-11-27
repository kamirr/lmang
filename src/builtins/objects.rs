use crate::env::{Env, Eval};
use crate::val::{DynObject, Object, Val};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::{FnState, RustFn};

use rand::{RngCore as _, SeedableRng};
use rand::rngs::SmallRng;

#[derive(Clone, Debug)]
struct Rng {
    rng: Rc<RefCell<SmallRng>>,
}

impl Rng {
    pub fn boxed() -> Box<Self> {
        Box::new(Rng {
            rng: Rc::new(RefCell::new(SmallRng::seed_from_u64(0))),
        })
    }

    pub fn next<'a, 'b>(_: &'a [Val], _: &'b mut Env, state: FnState) -> Result<Val, String> {
        let mut borrow = state.0.borrow_mut();
        let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();

        Ok(Val::Number(rng.next_u32() as i32))
    }

    pub fn seed<'a, 'b>(vals: &'a [Val], _: &'b mut Env, state: FnState) -> Result<Val, String> {
        let mut borrow = state.0.borrow_mut();
        let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();

        let new_seed = vals[0].as_number()?;
        *rng = SmallRng::seed_from_u64(new_seed as u64);

        Ok(Val::Unit)
    }
}

impl Object for Rng {
    fn member_names(&self) -> Vec<String> {
        vec!["next".to_string(), "state".to_string()]
    }

    fn member(&self, name: &str) -> Result<Val, String> {
        match name {
            "next" => {
                let func = RustFn::stateful("next", Rng::next, &self.rng).into_val();
                Ok(func)
            },
            "seed" => {
                let func = RustFn::stateful("seed", Rng::seed, &self.rng).into_val();
                Ok(func)
            }
            _ => Err(format!("no member {}", name)),
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct BuiltinObjects;

impl Eval for BuiltinObjects {
    fn eval(&self, env: &mut Env) -> Result<Val, String> {
        env.store_binding("rng".to_string(), Val::Object(DynObject(Rng::boxed())));

        Ok(Val::Unit)
    }
}
