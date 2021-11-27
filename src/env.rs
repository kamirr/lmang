use crate::val::Val;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Default)]
pub struct Env {
    stack: Vec<HashMap<String, Val>>,
    timeout: Option<Instant>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            stack: vec![HashMap::new()],
            timeout: None,
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Env {
            stack: vec![HashMap::new()],
            timeout: Some(Instant::now() + Duration::from_secs_f32(0.1)),
        }
    }

    pub fn push(&mut self) {
        self.stack.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.stack.pop().expect("can't pop last stack frame");
    }

    pub fn store_binding(&mut self, name: String, val: Val) {
        self.stack.last_mut().map(|frame| frame.insert(name, val));
    }

    pub fn set_binding(&mut self, name: &str, new_val: Val) -> Result<(), String> {
        for frame in self.stack.iter_mut().rev() {
            if let Some(val) = frame.get_mut(name) {
                match val {
                    Val::Ref(rc) => *rc.borrow_mut() = new_val,
                    _ => *val = new_val,
                }

                return Ok(());
            }
        }

        Err(format!("binding with name `{}` does not exist", name))
    }

    pub fn get_binding(&self, name: &str) -> Result<Val, String> {
        for frame in self.stack.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Ok(val.clone());
            }
        }

        Err(format!("binding with name `{}` does not exist", name))
    }

    pub fn take_ref(&mut self, name: &str) -> Result<Val, String> {
        for frame in self.stack.iter_mut().rev() {
            if let Some(val) = frame.remove(name) {
                let rc = Rc::new(RefCell::new(val));
                let val = Val::Ref(rc);
                frame.insert(name.into(), val.clone());

                return Ok(val);
            }
        }

        Err(format!("binding with name `{}` does not exist", name))
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = Some(Instant::now() + dur);
    }

    pub fn eval(&mut self, expr: &impl Eval) -> Result<Val, String> {
        if self.timeout.map(|t| Instant::now() > t).unwrap_or(false) {
            Err("timeout".to_string())
        } else {
            expr.eval(self)
        }
    }
}

pub trait Eval {
    fn eval(&self, env: &mut Env) -> Result<Val, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Err("binding with name `b` does not exist".to_string())
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
            Err("binding with name `b` does not exist".to_string())
        );
        assert_eq!(env.get_binding("c"), Ok(Val::Number(7)));
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

        let expected = vec![{
            let mut tmp = HashMap::new();
            tmp.insert(
                "a".to_string(),
                Val::Ref(Rc::new(RefCell::new(Val::Number(4)))),
            );

            tmp
        }];

        assert_eq!(env.stack, expected);
    }

    #[test]
    fn fn_edit_ref() {
        use crate::expr::{binding_update::BindingUpdate, call::Call};

        let (_, bu) = BindingUpdate::new("👶 become2 = 🧰 arg ➡️ set arg = 2 🧑‍🦲").unwrap();
        let (_, call) = Call::new("📞 become2 x").unwrap();

        let mut env = Env::test();
        env.eval(&bu).unwrap();

        // create a binding x and transform it into a refcounted one
        env.store_binding("x".to_string(), Val::Number(1));
        let _ = env.take_ref("x").unwrap();

        // given a ref as its argument, fn should modify it using set in
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
}
