use crate::error::RuntimeError;
use crate::utils::kwords;
use crate::val::Val;
use std::fmt;

pub trait Object {
    fn member_names(&self) -> Vec<String>;
    fn member(&self, name: &str) -> Result<Val, RuntimeError>;
    fn clone_box(&self) -> Box<dyn Object>;
    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
    fn name(&self) -> &str;
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
        write!(f, "{}{}", kwords::CLASS, self.0.name())?;
        write!(f, " (")?;
        for m in &self.0.member_names() {
            write!(f, "{}:{}, ", m, self.0.member(m).unwrap())?;
        }
        writeln!(f, ")")?;
        Ok(())
    }
}

pub fn placeholder_object() -> DynObject {
    struct ImplDetail;

    impl Object for ImplDetail {
        fn member_names(&self) -> Vec<String> {
            unreachable!()
        }
        fn member(&self, _name: &str) -> Result<Val, RuntimeError> {
            unreachable!()
        }
        fn clone_box(&self) -> Box<dyn Object> {
            unreachable!()
        }
        fn dyn_debug(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
            unreachable!()
        }
        fn name(&self) -> &str {
            todo!()
        }
    }

    DynObject(Box::new(ImplDetail))
}
