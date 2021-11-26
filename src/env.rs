use crate::val::Val;
use std::collections::HashMap;
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
                *val = new_val;
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
}
