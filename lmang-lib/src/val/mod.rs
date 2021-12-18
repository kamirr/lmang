mod dynfunc;
mod dynobject;

pub use dynfunc::{placeholder_func, Callee, DynFunc};
pub use dynobject::{placeholder_object, DynObject, Object};

use crate::error::RuntimeError;
use crate::utils::kwords;
use std::cell::RefCell;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::VecDeque;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use std::rc::{Rc, Weak};

#[derive(Debug, Clone)]
pub struct WeakWrapper(pub Weak<RefCell<Val>>);

impl WeakWrapper {
    pub fn upgrade(&self) -> Option<Rc<RefCell<Val>>> {
        self.0.upgrade()
    }
}

impl PartialEq for WeakWrapper {
    fn eq(&self, other: &Self) -> bool {
        match (self.0.upgrade(), other.0.upgrade()) {
            (Some(rc_self), Some(rc_other)) => rc_self.borrow().eq(&*rc_other.borrow()),
            (None, None) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Val {
    // base types
    Number(i32),
    Char(char),
    Bool(bool),
    Unit,
    // collections
    Break(Box<Val>),
    Deque(Box<VecDeque<Val>>),
    // special
    Func(DynFunc),
    Object(DynObject),
    Ref(Rc<RefCell<Val>>),
    Weak(WeakWrapper),
    Named((String, Box<Val>)),
}

fn pretty_print_deque(dq: &VecDeque<Val>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let all_chars = dq.iter().map(|v| v.as_char()).all(|v| matches!(v, Ok(_)));

    if all_chars {
        for v in dq.iter() {
            write!(f, "{}", v)?;
        }
    } else {
        write!(f, "🧵")?;
        for v in dq.iter() {
            write!(f, "{},", v)?;
        }
        write!(f, "🧵")?;
    }

    Ok(())
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Char(c) => write!(f, "{}", c),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Unit => write!(f, "📦🧑‍🦲"),
            Self::Break(val) => write!(f, "💔{}", val.as_ref()),
            Self::Deque(vals) => pretty_print_deque(vals, f),
            Self::Func(df) => write!(f, "{}", df),
            Self::Object(obj) => write!(f, "{}", obj),
            Self::Ref(rc) => write!(f, "🔖{}", rc.borrow()),
            Self::Weak(wk) => {
                write!(f, "{}", self.variant_name())?;
                match wk.upgrade() {
                    Some(rc) => write!(f, "{}", rc.borrow().variant_name()),
                    None => write!(f, "🥵"),
                }
            }
            Self::Named((name, val)) => write!(f, "{}{}{}", name, kwords::NAMED, &val),
        }
    }
}

impl Val {
    pub fn variant_name(&self) -> &'static str {
        use Val::*;

        match self {
            Number(_) => "🔢",
            Char(_) => "🔡",
            Bool(_) => "🤨",
            Unit => "📦🧑‍🦲",
            Break(_) => "💔",
            Deque(_) => "😵‍💫😵‍💫",
            Func(_) => "🧰",
            Object(_) => "🧑‍🏫",
            Ref(_) => "🔖",
            Weak(_) => "🦽",
            Named(_) => ":",
        }
    }

    pub fn as_number(&self) -> Result<&i32, RuntimeError> {
        match self {
            Val::Number(n) => Ok(n),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔢".into(),
            }),
        }
    }

    pub fn as_char(&self) -> Result<&char, RuntimeError> {
        match self {
            Val::Char(c) => Ok(c),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔡".into(),
            }),
        }
    }

    pub fn as_bool(&self) -> Result<&bool, RuntimeError> {
        match self {
            Val::Bool(b) => Ok(b),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "😵‍💫".into(),
            }),
        }
    }

    pub fn as_unit(&self) -> Result<&(), RuntimeError> {
        match self {
            Val::Unit => Ok(&()),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "📦🧑‍🦲".into(),
            }),
        }
    }

    pub fn as_break(&self) -> Result<&Val, RuntimeError> {
        match self {
            Val::Break(b) => Ok(b.as_ref()),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "💔".into(),
            }),
        }
    }

    pub fn as_deque(&self) -> Result<&VecDeque<Val>, RuntimeError> {
        match self {
            Self::Deque(obj) => Ok(obj.as_ref()),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "😵‍💫😵‍💫".into(),
            }),
        }
    }

    pub fn as_func(&self) -> Result<&DynFunc, RuntimeError> {
        match self {
            Val::Func(f) => Ok(f),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🧰".into(),
            }),
        }
    }

    pub fn as_object(&self) -> Result<&DynObject, RuntimeError> {
        match self {
            Self::Object(obj) => Ok(obj),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🧑‍🏫".into(),
            }),
        }
    }

    pub fn as_val_ref(&self) -> Result<&Rc<RefCell<Val>>, RuntimeError> {
        match self {
            Self::Ref(rc) => Ok(rc),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔖".into(),
            }),
        }
    }

    pub fn apply_to_root<T, F>(&self, f: F) -> Result<T, RuntimeError>
    where
        F: FnOnce(&Val) -> T,
    {
        let wk_err = RuntimeError::Dangling;

        match self {
            Val::Ref(rc) => rc.borrow().apply_to_root(f),
            Val::Weak(wk) => match wk.upgrade() {
                Some(rc) => rc.borrow().apply_to_root(f),
                _ => Err(wk_err),
            },
            root => Ok(f(root)),
        }
    }

    pub fn as_number_mut(&mut self) -> Result<&mut i32, RuntimeError> {
        match self {
            Val::Number(n) => Ok(n),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔢".into(),
            }),
        }
    }

    pub fn as_char_mut(&mut self) -> Result<&mut char, RuntimeError> {
        match self {
            Val::Char(c) => Ok(c),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔡".into(),
            }),
        }
    }

    pub fn as_bool_mut(&mut self) -> Result<&mut bool, RuntimeError> {
        match self {
            Val::Bool(b) => Ok(b),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "😵‍💫".into(),
            }),
        }
    }

    pub fn as_break_mut(&mut self) -> Result<&mut Val, RuntimeError> {
        match self {
            Val::Break(b) => Ok(b.as_mut()),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "💔".into(),
            }),
        }
    }

    pub fn as_deque_mut(&mut self) -> Result<&mut VecDeque<Val>, RuntimeError> {
        match self {
            Self::Deque(obj) => Ok(obj.as_mut()),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "😵‍💫😵‍💫".into(),
            }),
        }
    }

    pub fn as_func_mut(&mut self) -> Result<&mut DynFunc, RuntimeError> {
        match self {
            Val::Func(f) => Ok(f),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🧰".into(),
            }),
        }
    }

    pub fn as_object_mut(&mut self) -> Result<&mut DynObject, RuntimeError> {
        match self {
            Self::Object(obj) => Ok(obj),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🧑‍🏫".into(),
            }),
        }
    }

    pub fn as_val_ref_mut(&mut self) -> Result<&mut Rc<RefCell<Val>>, RuntimeError> {
        match self {
            Self::Ref(rc) => Ok(rc),
            _ => Err(RuntimeError::CastError {
                from: self.variant_name().to_string(),
                to: "🔖".into(),
            }),
        }
    }

    pub fn apply_to_root_mut<T, F>(&mut self, f: F) -> Result<T, RuntimeError>
    where
        F: FnOnce(&mut Val) -> T,
    {
        let wk_err = RuntimeError::Dangling;

        match self {
            Val::Ref(rc) => rc.borrow_mut().apply_to_root_mut(f),
            Val::Weak(wk) => match wk.upgrade() {
                Some(rc) => rc.borrow_mut().apply_to_root_mut(f),
                _ => Err(wk_err),
            },
            root => Ok(f(root)),
        }
    }

    pub fn make_ref(&mut self) -> Val {
        match self {
            Val::Ref(rc) => Val::Ref(rc.clone()),
            _ => {
                let self_own = std::mem::replace(self, Val::Unit);
                let rc = Rc::new(RefCell::new(self_own));
                *self = Val::Ref(rc.clone());

                Val::Ref(rc)
            }
        }
    }

    pub fn try_gt(&self, other: &Val) -> Result<Self, RuntimeError> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self > other))
        } else {
            Err(RuntimeError::InvalidOp {
                lhs: self.variant_name().into(),
                rhs: other.variant_name().into(),
                op: kwords::LT.into(),
            })
        }
    }

    pub fn try_ge(&self, other: &Val) -> Result<Self, RuntimeError> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self >= other))
        } else {
            Err(RuntimeError::InvalidOp {
                lhs: self.variant_name().into(),
                rhs: other.variant_name().into(),
                op: kwords::GE.into(),
            })
        }
    }

    pub fn try_lt(&self, other: &Val) -> Result<Self, RuntimeError> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self < other))
        } else {
            Err(RuntimeError::InvalidOp {
                lhs: self.variant_name().into(),
                rhs: other.variant_name().into(),
                op: kwords::LT.into(),
            })
        }
    }

    pub fn try_le(&self, other: &Val) -> Result<Self, RuntimeError> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self <= other))
        } else {
            Err(RuntimeError::InvalidOp {
                lhs: self.variant_name().into(),
                rhs: other.variant_name().into(),
                op: kwords::LE.into(),
            })
        }
    }
}

impl<'a, 'b> Add<&'b Val> for &'a Val {
    type Output = Result<Val, RuntimeError>;

    fn add(self, other: &'b Val) -> Self::Output {
        let n = self.apply_to_root(|v1| {
            other.apply_to_root::<Result<_, RuntimeError>, _>(|v2| {
                Ok(v1.as_number()? + v2.as_number()?)
            })
        })???;
        Ok(Val::Number(n))
    }
}

impl Add for Val {
    type Output = Result<Val, RuntimeError>;

    fn add(self, other: Self) -> Self::Output {
        &self + &other
    }
}

impl<'a, 'b> Sub<&'b Val> for &'a Val {
    type Output = Result<Val, RuntimeError>;

    fn sub(self, other: &'b Val) -> Self::Output {
        let n = self.apply_to_root(|v1| {
            other.apply_to_root::<Result<_, RuntimeError>, _>(|v2| {
                Ok(v1.as_number()? - v2.as_number()?)
            })
        })???;
        Ok(Val::Number(n))
    }
}

impl Sub for Val {
    type Output = Result<Val, RuntimeError>;

    fn sub(self, other: Self) -> Self::Output {
        &self - &other
    }
}

impl<'a, 'b> Mul<&'b Val> for &'a Val {
    type Output = Result<Val, RuntimeError>;

    fn mul(self, other: &'b Val) -> Self::Output {
        let n = self.apply_to_root(|v1| {
            other.apply_to_root::<Result<_, RuntimeError>, _>(|v2| {
                Ok(v1.as_number()? * v2.as_number()?)
            })
        })???;
        Ok(Val::Number(n))
    }
}

impl Mul for Val {
    type Output = Result<Val, RuntimeError>;

    fn mul(self, other: Self) -> Self::Output {
        &self * &other
    }
}

impl<'a, 'b> Div<&'b Val> for &'a Val {
    type Output = Result<Val, RuntimeError>;

    fn div(self, other: &'b Val) -> Self::Output {
        let n = self.apply_to_root(|v1| {
            other.apply_to_root::<Result<_, RuntimeError>, _>(|v2| {
                Ok(v1.as_number()? / v2.as_number()?)
            })
        })???;
        Ok(Val::Number(n))
    }
}

impl Div for Val {
    type Output = Result<Val, RuntimeError>;

    fn div(self, other: Self) -> Self::Output {
        &self / &other
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
        match self {
            Self::Number(n1) => {
                let n2 = other.apply_to_root(|v| Some(*v.as_number().ok()?)).ok()??;
                Some(n1.cmp(&n2))
            }
            _ => None,
        }
    }
}
