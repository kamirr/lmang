use crate::val::JsObj;

// will be unused if built with feature web but for arch other than wasm32
#[allow(dead_code)]
pub fn make_web_builtin() -> JsObj {
    JsObj::new_eval("js", "globalThis").unwrap()
}
