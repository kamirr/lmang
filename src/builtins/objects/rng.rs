use crate::builtins::rustfn::{FnState, RustFn};
use crate::env::Env;
use crate::val::{Object, Val};
use rand::rngs::SmallRng;
use rand::{RngCore as _, SeedableRng};
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct RngBuiltin {
    rng: Rc<RefCell<SmallRng>>,
    fns: Vec<RustFn>,
}

impl RngBuiltin {
    pub fn boxed() -> Box<Self> {
        let rng = Rc::new(RefCell::new(SmallRng::seed_from_u64(0)));
        let fns = vec![
            RustFn::stateful("next", RngBuiltin::next, &rng),
            RustFn::stateful("seed", RngBuiltin::seed, &rng),
        ];

        Box::new(RngBuiltin { rng, fns })
    }

    fn next(_: &[Val], _: &mut Env, state: FnState) -> Result<Val, String> {
        let mut borrow = state.0.borrow_mut();
        let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();

        Ok(Val::Number(rng.next_u32() as i32))
    }

    fn seed(vals: &[Val], _: &mut Env, state: FnState) -> Result<Val, String> {
        let mut borrow = state.0.borrow_mut();
        let rng: &mut SmallRng = borrow.downcast_mut::<SmallRng>().unwrap();

        let new_seed = vals[0].apply_to_root(|v| v.as_number().map(|n| *n))??;
        *rng = SmallRng::seed_from_u64(new_seed as u64);

        Ok(Val::Unit)
    }
}

impl Object for RngBuiltin {
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
        "rng"
    }
}
