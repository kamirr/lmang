use crate::val::Val;
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use ahash::RandomState;

#[derive(Debug, PartialEq, Default)]
pub struct Env {
    stack: Vec<HashMap<String, Val, RandomState>>,
    timeout: Option<Instant>,
    last_popped: Option<HashMap<String, Val, RandomState>>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            stack: vec![HashMap::default()],
            timeout: None,
            last_popped: None,
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Env {
            stack: vec![HashMap::default()],
            timeout: Some(Instant::now() + Duration::from_secs_f32(0.1)),
            last_popped: None,
        }
    }

    pub fn push(&mut self) {
        self.stack.push(HashMap::default());
    }

    pub fn pop(&mut self) {
        self.last_popped = Some(self.stack.pop().expect("can't pop last stack frame"));
    }

    pub fn take_last_popped(&mut self) -> Option<HashMap<String, Val, RandomState>> {
        self.last_popped.take()
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

    pub fn get_binding<'a, 'b>(&'a self, name: &'b str) -> Result<Cow<'a, Val>, String> {
        for frame in self.stack.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Ok(Cow::Borrowed(val));
            }
        }

        Err(format!("binding with name `{}` does not exist", name))
    }

    pub fn take_ref<'a, 'b>(&'a mut self, name: &'b str) -> Result<Cow<'a, Val>, String> {
        for frame in self.stack.iter_mut().rev() {
            if let Some(val) = frame.remove(name) {
                let rc = Rc::new(RefCell::new(val));
                let val = Val::Ref(rc);
                frame.insert(name.into(), val.clone());

                return Ok(Cow::Owned(val));
            }
        }

        Err(format!("binding with name `{}` does not exist", name))
    }

    pub fn set_timeout(&mut self, dur: Duration) {
        self.timeout = Some(Instant::now() + dur);
    }

    pub fn eval<'a, 'b>(&'a mut self, expr: &'b impl Eval) -> Result<Cow<'a, Val>, String> {
        if self.timeout.map(|t| Instant::now() > t).unwrap_or(false) {
            Err("timeout".to_string())
        } else {
            expr.eval(self)
        }
    }
}

pub trait Eval {
    fn eval<'a, 'b>(&'a self, env: &'b mut Env) -> Result<Cow<'b, Val>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_store_get() {
        let val = Val::Number(4);

        let mut env = Env::test();
        env.store_binding("a".to_string(), val.clone());
        assert_eq!(env.get_binding("a"), Ok(Cow::Borrowed(&val)));
    }

    #[test]
    fn env_child() {
        let mut env = Env::test();
        env.store_binding("a".to_string(), Val::Number(3));
        env.store_binding("c".to_string(), Val::Number(9));

        assert_eq!(env.get_binding("a"), Ok(Cow::Borrowed(&Val::Number(3))));
        assert_eq!(
            env.get_binding("b"),
            Err("binding with name `b` does not exist".to_string())
        );
        assert_eq!(env.get_binding("c"), Ok(Cow::Borrowed(&Val::Number(9))));

        env.push();
        env.store_binding("a".to_string(), Val::Bool(false));
        env.store_binding("b".to_string(), Val::Unit);

        assert_eq!(env.get_binding("c"), Ok(Cow::Borrowed(&Val::Number(9))));

        env.set_binding("c", Val::Number(7)).unwrap();

        assert_eq!(env.get_binding("a"), Ok(Cow::Borrowed(&Val::Bool(false))));
        assert_eq!(env.get_binding("b"), Ok(Cow::Borrowed(&Val::Unit)));
        assert_eq!(env.get_binding("c"), Ok(Cow::Borrowed(&Val::Number(7))));
        env.pop();

        assert_eq!(env.get_binding("a"), Ok(Cow::Borrowed(&Val::Number(3))));
        assert_eq!(
            env.get_binding("b"),
            Err("binding with name `b` does not exist".to_string())
        );
        assert_eq!(env.get_binding("c"), Ok(Cow::Borrowed(&Val::Number(7))));
    }

    #[test]
    fn env_ref() {
        let mut env = Env::test();
        env.store_binding("a".to_string(), Val::Number(0));

        let val_ref = env.take_ref("a").unwrap();
        if let Val::Ref(rc) = val_ref.as_ref() {
            assert_eq!(*rc.borrow(), Val::Number(0));
            *rc.borrow_mut() = Val::Number(4);
        } else {
            panic!("val not ref");
        }

        if let Val::Ref(rc) = env.get_binding("a").unwrap().as_ref() {
            assert_eq!(*rc.borrow(), Val::Number(4));
        } else {
            panic!("val not ref");
        }

        let expected = vec![{
            let mut tmp = HashMap::default();
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
}
