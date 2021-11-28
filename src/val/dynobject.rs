use crate::val::Val;
use std::fmt;

pub trait Object {
    fn member_names(&self) -> Vec<String>;
    fn member(&self, name: &str) -> Result<Val, String>;
    fn clone_box(&self) -> Box<dyn Object>;
    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

pub struct DynObject(pub Box<dyn Object>);

impl Clone for DynObject {
    fn clone(&self) -> Self {
        DynObject(self.0.clone_box())
    }
}

impl PartialEq for DynObject {
    fn eq(&self, other: &DynObject) -> bool {
        use std::any::Any;
        if self.type_id() != other.type_id() {
            false
        } else {
            format!("{:?}", self) == format!("{:?}", other)
        }
    }
}

impl fmt::Debug for DynObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_debug(f)
    }
}

impl fmt::Display for DynObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn placeholder_object() -> DynObject {
    struct ImplDetail;

    impl Object for ImplDetail {
        fn member_names(&self) -> Vec<String> {
            unreachable!()
        }
        fn member(&self, _name: &str) -> Result<Val, String> {
            unreachable!()
        }
        fn clone_box(&self) -> Box<dyn Object> {
            unreachable!()
        }
        fn dyn_debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            unreachable!()
        }
    }

    DynObject(Box::new(ImplDetail))
}
