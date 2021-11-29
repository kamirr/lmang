mod dynfunc;
mod dynobject;

pub use dynfunc::{placeholder_func, Callee, DynFunc};
pub use dynobject::{placeholder_object, DynObject, Object};

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
    // collectionsv
    Break(Box<Val>),
    Deque(Box<VecDeque<Val>>),
    // special
    Func(DynFunc),
    Object(DynObject),
    Ref(Rc<RefCell<Val>>),
    Weak(WeakWrapper),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Char(c) => write!(f, "{}", c),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Unit => write!(f, "📦🧑‍🦲"),
            Self::Break(val) => write!(f, "💔{}", val.as_ref()),
            Self::Deque(vals) => {
                for v in vals.as_ref() {
                    write!(f, "{}", v)?;
                }

                Ok(())
            },
            Self::Func(df) => write!(f, "{:?}", df),
            Self::Object(obj) => write!(f, "{}", obj),
            Self::Ref(rc) => write!(f, "🔖{}", rc.borrow()),
            Self::Weak(wk) => {
                write!(f, "{}", self.variant_name())?;
                match wk.upgrade() {
                    Some(rc) => write!(f, "{}", rc.borrow().variant_name()),
                    None => write!(f, "🥵"),
                }
            }
        }
    }
}

impl Val {
    pub fn variant_name(&self) -> &'static str {
        use Val::*;

        match self {
            Number(_) => "🔢",
            Char(_) => "🔡",
            Bool(_) => "😵‍💫",
            Unit => "📦🧑‍🦲",
            Break(_) => "💔",
            Deque(_) => "😵‍💫😵‍💫",
            Func(_) => "🧰",
            Object(_) => "🧑‍🏫",
            Ref(_) => "🔖",
            Weak(_) => "🦽",
        }
    }

    pub fn try_match_type(&self, other: &Self) -> Result<Self, String> {
        let err = format!("can't convert type `{}` to `{}`", self.variant_name(), other.variant_name());

        use Val::*;
        match self {
            Ref(rc) => rc.borrow().try_match_type(other),
            Weak(wk) => match wk.upgrade() {
                Some(rc) => rc.borrow().try_match_type(other),
                _ => Err(err),
            }
            Func(f) => match other {
                Func(_) => Ok(Func(f.clone())),
                _ => Err(err),
            },
            Object(obj) => match other {
                Object(_) => Ok(Object(obj.clone())),
                _ => Err(err),
            },
            Deque(d) => match other {
                Deque(_) => Ok(Deque(d.clone())),
                _ => Err(err),
            },
            Char(c) => match other {
                Char(_) => Ok(Char(*c)),
                Number(_) => Ok(Number(*c as i32)),
                _ => Err(err),
            },
            Number(n) => match other {
                Char(_) => char::from_u32(*n as u32).map(Char).ok_or(err),
                Number(_) => Ok(Number(*n)),
                _ => Err(err),
            },
            Bool(b) => match other {
                Bool(_) => Ok(Bool(*b)),
                _ => Err(err),
            },
            Unit => match other {
                Unit => Ok(Val::Unit),
                _ => Err(err),
            },
            _ => Err(err),
        }
    }

    pub fn as_number(&self) -> Result<i32, String> {
        use Val::*;

        let self_number = self.try_match_type(&Number(0))?;

        match self_number {
            Val::Number(n) => Ok(n),
            _ => unreachable!(),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        use Val::*;

        let self_number = self.try_match_type(&Bool(false))?;

        match self_number {
            Val::Bool(b) => Ok(b),
            _ => unreachable!(),
        }
    }

    pub fn as_func(&self) -> Result<DynFunc, String> {
        use Val::*;

        let self_func = self.try_match_type(&Func(placeholder_func()))?;

        match self_func {
            Self::Func(f) => Ok(f),
            _ => Err(format!("can't convert type `{}` to `{}`", self.variant_name(), Val::Func(placeholder_func()).variant_name())),
        }
    }

    pub fn as_val_ref(&self) -> Result<Rc<RefCell<Val>>, String> {
        match self {
            Self::Ref(rc) => Ok(rc.clone()),
            _ => Err(format!("can't convert type `{}` to `{}`", self.variant_name(), Val::Ref(Rc::new(RefCell::new(Val::Unit))).variant_name())),
        }
    }

    pub fn as_object(&self) -> Result<DynObject, String> {
        match self {
            Self::Object(obj) => Ok(obj.clone()),
            _ => Err(format!("can't convert type `{}` to `{}`", self.variant_name(), Val::Object(placeholder_object()).variant_name())),
        }
    }

    pub fn try_gt(&self, other: &Val) -> Result<Self, String> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self > other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", self.variant_name(), other.variant_name()))
        }
    }

    pub fn try_ge(&self, other: &Val) -> Result<Self, String> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self >= other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", self.variant_name(), other.variant_name()))
        }
    }

    pub fn try_eq(&self, other: &Val) -> Result<Self, String> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self == other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", self.variant_name(), other.variant_name()))
        }
    }

    pub fn try_lt(&self, other: &Val) -> Result<Self, String> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self < other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", self.variant_name(), other.variant_name()))
        }
    }

    pub fn try_le(&self, other: &Val) -> Result<Self, String> {
        if self.partial_cmp(other).is_some() {
            Ok(Self::Bool(self <= other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", self.variant_name(), other.variant_name()))
        }
    }
}

impl<'a, 'b> Add<&'b Val> for &'a Val {
    type Output = Result<Val, String>;

    fn add(self, other: &'b Val) -> Self::Output {
        let err = Err(format!(
            "can't add bindings of types `{}` and `{}`",
            "?", "?"
        ));

        use Val::*;
        match self {
            Number(n) => Ok(Number(n + other.as_number()?)),
            _ => err,
        }
    }
}

impl Add for Val {
    type Output = Result<Val, String>;

    fn add(self, other: Self) -> Self::Output {
        &self + &other
    }
}

impl<'a, 'b> Sub<&'b Val> for &'a Val {
    type Output = Result<Val, String>;

    fn sub(self, other: &'b Val) -> Self::Output {
        let err = Err(format!(
            "can't sub bindings of types `{}` and `{}`",
            "?", "?"
        ));

        use Val::*;
        match self {
            Number(n) => Ok(Number(n - other.as_number()?)),
            _ => err,
        }
    }
}

impl Sub for Val {
    type Output = Result<Val, String>;

    fn sub(self, other: Self) -> Self::Output {
        &self - &other
    }
}

impl<'a, 'b> Mul<&'b Val> for &'a Val {
    type Output = Result<Val, String>;

    fn mul(self, other: &'b Val) -> Self::Output {
        let err = Err(format!(
            "can't mul bindings of types `{}` and `{}`",
            "?", "?"
        ));

        use Val::*;
        match self {
            Number(n) => Ok(Number(n * other.as_number()?)),
            _ => err,
        }
    }
}

impl Mul for Val {
    type Output = Result<Val, String>;

    fn mul(self, other: Self) -> Self::Output {
        &self * &other
    }
}

impl<'a, 'b> Div<&'b Val> for &'a Val {
    type Output = Result<Val, String>;

    fn div(self, other: &'b Val) -> Self::Output {
        let err = Err(format!(
            "can't div bindings of types `{}` and `{}`",
            "?", "?"
        ));

        use Val::*;
        match self {
            Number(n) => Ok(Number(n / other.as_number()?)),
            _ => err,
        }
    }
}

impl Div for Val {
    type Output = Result<Val, String>;

    fn div(self, other: Self) -> Self::Output {
        &self / &other
    }
}

impl PartialOrd for Val {
    fn partial_cmp(&self, other: &Val) -> Option<Ordering> {
        match self {
            Self::Number(n1) => other.as_number().map(|n2| n1.cmp(&n2)).ok(),
            _ => None,
        }
    }
}
