use crate::env::Env;
use crate::error::RuntimeError;
use crate::val::{Callee, Object, Val};
use js_sys::{Function as JsFunction, Object as JsObject};
use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use wasm_bindgen::JsValue;

fn dq_to_jv(dq: &VecDeque<Val>, env: &Env) -> JsValue {
    let all_chars = dq.iter().all(|v| v.as_char().is_ok());
    if all_chars {
        let s: String = dq.iter().map(|v| *v.as_char().unwrap()).collect();
        JsValue::from_str(&s)
    } else {
        let arr = js_sys::Array::new();
        for v in dq {
            arr.push(&val_to_jv(v, env));
        }

        arr.value_of().into()
    }
}

fn val_to_jv(val: &Val, env: &Env) -> JsValue {
    match val {
        Val::Bool(b) => JsValue::from(*b),
        Val::Number(n) => JsValue::from(*n),
        Val::Deque(dq) => dq_to_jv(dq.as_ref(), env),
        Val::Func(f) => {
            let mut env_own = env.shared();
            let f_own = f.clone();
            let cl = move || -> JsValue {
                let res = f_own.0.call(&mut [], &mut env_own);
                match res {
                    Ok(val) => val_to_jv(&val, &env_own),
                    Err(e) => format!("{}", e).into(),
                }
            };
            let cl = Box::new(cl) as Box<dyn FnMut() -> JsValue>;
            let cl = wasm_bindgen::closure::Closure::wrap(cl);
            cl.into_js_value()
        }
        Val::JsValue(jv) => jv.clone(),
        _ => JsValue::UNDEFINED,
    }
}

#[derive(Clone)]
pub struct JsFunc {
    name: String,
    val: JsFunction,
}

impl JsFunc {
    pub fn new(name: impl Into<String>, val: JsFunction) -> Self {
        Self {
            name: name.into(),
            val,
        }
    }
}

impl fmt::Debug for JsFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JsFunc")
            .field("name", &self.name)
            .field("val", &"[...]")
            .finish()
    }
}

impl Callee for JsFunc {
    fn call(&self, args: &mut [Val], env: &mut Env) -> Result<Val, RuntimeError> {
        let arr = js_sys::Array::new();

        for arg in args {
            arr.push(&val_to_jv(arg, env));
        }

        let jv = self.val.apply(&js_sys::eval("globalThis")?, &arr)?;

        Ok(if jv.is_function() {
            Val::convert_from_jv(jv)
        } else {
            Val::JsValue(jv)
        })
    }

    fn clone_box(&self) -> Box<dyn Callee> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::fmt::Debug;
        self.fmt(f)
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn dyn_display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "jsfunc")
    }
}

#[derive(Clone)]
pub struct JsObj {
    name: String,
    val: JsValue,
}

impl JsObj {
    pub fn new_eval(name: impl Into<String>, expr: &str) -> Result<Self, RuntimeError> {
        let name = name.into();
        let val = js_sys::eval(expr)?;

        Ok(JsObj { name, val })
    }

    pub fn new_jv(name: impl Into<String>, val: JsValue) -> Self {
        JsObj {
            name: name.into(),
            val,
        }
    }

    fn member_ffi(&self, name: &str) -> Result<Val, RuntimeError> {
        js_sys::Reflect::set(
            &js_sys::global().into(),
            &"__js_tmp".into(),
            &(&self.val).into(),
        )?;

        let js = format!("globalThis.__js_tmp.{}", name);
        let jv = js_sys::eval(&js)?;
        let jv = if jv.is_function() {
            js_sys::Function::from(jv).bind(&self.val).into()
        } else {
            jv
        };

        Ok(if jv.is_function() {
            Val::convert_from_jv(jv)
        } else {
            Val::JsValue(jv)
        })
    }
}

impl Object for JsObj {
    fn member_names(&self) -> Vec<String> {
        let self_proto: JsObject =
            js_sys::Reflect::get_prototype_of(&self.val).unwrap_or(js_sys::Object::new());
        JsObject::keys(&self_proto)
            .iter()
            .filter_map(|jv| jv.as_string())
            .chain(["set"].map(ToString::to_string).into_iter())
            .collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        if name == "set" {
            let jv = js_sys::eval(
                "(self, prop_name, new_val) => { Reflect.set(self, prop_name, new_val); }",
            )?;
            let func = js_sys::Function::from(jv).bind1(&JsValue::UNDEFINED, &self.val);
            Ok(Val::convert_from_jv(func.into()))
        } else {
            self.member_ffi(name)
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.val)
    }

    fn name(&self) -> &str {
        &self.name
    }
}
