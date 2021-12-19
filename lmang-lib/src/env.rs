use crate::error::RuntimeError;
use crate::val::Val;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use ahash::RandomState;

type StackFrame = HashMap<String, Val, RandomState>;

#[derive(Debug, PartialEq, Default)]
pub struct Env {
    last_popped: Option<HashMap<String, Val, RandomState>>,
    root: Rc<RefCell<StackFrame>>,
    stack: Vec<StackFrame>,
    timeout: Option<Instant>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            last_popped: None,
            root: Rc::new(RefCell::new(HashMap::default())),
            stack: Vec::new(),
            timeout: None,
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        let timeout = Instant::now() + Duration::from_secs_f32(0.1);

        Env {
            last_popped: None,
            root: Rc::new(RefCell::new(HashMap::default())),
            stack: Vec::new(),
            timeout: Some(timeout),
        }
    }

    pub fn shared(&self) -> Self {
        Env {
            last_popped: None,
            root: self.root.clone(),
            stack: Vec::new(),
            timeout: self.timeout,
        }
    }

    pub fn push(&mut self) {
        self.stack.push(HashMap::default());
    }

    pub fn pop(&mut self) {
        self.last_popped = Some(self.stack.pop().expect("stack empty"));
    }

    pub fn take_last_popped(&mut self) -> Option<HashMap<String, Val, RandomState>> {
        self.last_popped.take()
    }

    pub fn store_binding(&mut self, name: String, val: Val) {
        match self.stack.last_mut() {
            Some(frame) => {
                frame.insert(name, val);
            }
            None => {
                let mut borrow = self.root.borrow_mut();
                borrow.insert(name, val);
            }
        };
    }

    pub fn store_global(&mut self, name: String, val: Val) {
        let mut borrow = self.root.borrow_mut();
        borrow.insert(name, val);
    }

    pub fn set_binding(&mut self, name: &str, new_val: Val) -> Result<(), RuntimeError> {
        for frame in self.stack.iter_mut().rev() {
            if let Some(val) = frame.get_mut(name) {
                match val {
                    Val::Ref(rc) => *rc.borrow_mut() = new_val,
                    _ => *val = new_val,
                }

                return Ok(());
            }
        }

        let mut borrow = self.root.borrow_mut();
        if let Some(val) = borrow.get_mut(name) {
            match val {
                Val::Ref(rc) => *rc.borrow_mut() = new_val,
                _ => *val = new_val,
            }

            return Ok(());
        }

        Err(RuntimeError::NoBinding(name.into()))
    }

    pub fn get_binding(&self, name: &str) -> Result<Val, RuntimeError> {
        for frame in self.stack.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Ok(val.clone());
            }
        }

        let borrow = self.root.borrow();
        if let Some(val) = borrow.get(name) {
            return Ok(val.clone());
        }

        Err(RuntimeError::NoBinding(name.into()))
    }

    pub fn take_ref(&mut self, name: &str) -> Result<Val, RuntimeError> {
        for frame in self.stack.iter_mut().rev() {
            if let Some(val) = frame.get_mut(name) {
                let val_ref = val.make_ref();
                return Ok(val_ref);
            }
        }

        let mut borrow = self.root.borrow_mut();
        if let Some(val) = borrow.get_mut(name) {
            let val_ref = val.make_ref();
            return Ok(val_ref);
        }

        Err(RuntimeError::NoBinding(name.into()))
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = Some(Instant::now() + dur);
    }

    pub fn eval(&mut self, expr: &impl Eval) -> Result<Val, RuntimeError> {
        if self.timeout.map(|t| Instant::now() > t).unwrap_or(false) {
            Err(RuntimeError::Timeout)
        } else {
            expr.eval(self)
        }
    }
}

pub trait Eval {
    fn eval(&self, env: &mut Env) -> Result<Val, RuntimeError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn env_store_get() {
        let val = Val::Number(4);

        let mut env = Env::test();
        env.store_binding("a".to_string(), val.clone());
        assert_eq!(env.get_binding("a"), Ok(val));
    }

    #[test]
    fn env_child() {
        let mut env = Env::test();
        env.store_binding("a".to_string(), Val::Number(3));
        env.store_binding("c".to_string(), Val::Number(9));

        assert_eq!(env.get_binding("a"), Ok(Val::Number(3)));
        assert_eq!(
            env.get_binding("b"),
            Err(RuntimeError::NoBinding("b".into()))
        );
        assert_eq!(env.get_binding("c"), Ok(Val::Number(9)));

        env.push();
        env.store_binding("a".to_string(), Val::Bool(false));
        env.store_binding("b".to_string(), Val::Unit);

        assert_eq!(env.get_binding("c"), Ok(Val::Number(9)));

        env.set_binding("c", Val::Number(7)).unwrap();

        assert_eq!(env.get_binding("a"), Ok(Val::Bool(false)));
        assert_eq!(env.get_binding("b"), Ok(Val::Unit));
        assert_eq!(env.get_binding("c"), Ok(Val::Number(7)));
        env.pop();

        assert_eq!(env.get_binding("a"), Ok(Val::Number(3)));
        assert_eq!(
            env.get_binding("b"),
            Err(RuntimeError::NoBinding("b".into()))
        );
        assert_eq!(env.get_binding("c"), Ok(Val::Number(7)));
    }

    #[test]
    fn prevent_double_ref() {
        let mut env = Env::test();
        env.store_binding("a".to_string(), Val::Unit);
        let _ = env.take_ref("a").unwrap();
        let val_ref = env.take_ref("a");

        assert_eq!(val_ref, Ok(Val::Ref(Rc::new(RefCell::new(Val::Unit)))));
    }

    #[test]
    fn env_ref() {
        let mut env = Env::test();
        env.store_binding("a".to_string(), Val::Number(0));

        let val_ref = env.take_ref("a").unwrap();
        if let Val::Ref(rc) = val_ref {
            assert_eq!(*rc.borrow(), Val::Number(0));
            *rc.borrow_mut() = Val::Number(4);
        } else {
            panic!("val not ref");
        }

        if let Val::Ref(rc) = env.get_binding("a").unwrap() {
            assert_eq!(*rc.borrow(), Val::Number(4));
        } else {
            panic!("val not ref");
        }

        let expected = {
            let mut tmp = HashMap::default();
            tmp.insert(
                "a".to_string(),
                Val::Ref(Rc::new(RefCell::new(Val::Number(4)))),
            );

            tmp
        };

        assert_eq!(env.root, Rc::new(RefCell::new(expected)));
    }

    #[test]
    fn fn_edit_ref() {
        use crate::expr::{binding_update::BindingUpdate, call::Call};

        let (_, bu) = BindingUpdate::new("👶 become2 = 🧰 arg ➡️ ♻️ arg = 2 🧑‍🦲").unwrap();
        let (_, call) = Call::new("📞 become2 x").unwrap();

        let mut env = Env::test();
        env.eval(&bu).unwrap();

        // create a binding x and transform it into a refcounted one
        env.store_binding("x".to_string(), Val::Number(1));
        let _ = env.take_ref("x").unwrap();

        // given a ref as its argument, fn should modify it using ♻️ in
        // a globally visible manner.
        env.eval(&call).unwrap();

        let result = env
            .get_binding("x")
            .unwrap()
            .as_val_ref()
            .unwrap()
            .borrow()
            .clone();
        assert_eq!(result, Val::Number(2));
    }

    #[test]
    fn env_shared_root_only() {
        let mut env = Env::test();

        env.store_binding("x".to_string(), Val::Unit);
        env.push();
        env.store_binding("y".to_string(), Val::Unit);

        let env_other = env.shared();

        assert_eq!(env_other.get_binding("x"), Ok(Val::Unit));
        assert_eq!(
            env_other.get_binding("y"),
            Err(RuntimeError::NoBinding("y".to_string()))
        );
    }

    #[test]
    fn test_timeout() {
        let mut env = Env::test();

        std::thread::sleep(std::time::Duration::from_secs_f32(0.2));

        let (_, unit_e) = crate::expr::Expr::new("📦🧑‍🦲").unwrap();
        let res = env.eval(&unit_e);

        assert_eq!(res, Err(RuntimeError::Timeout));
    }
}
