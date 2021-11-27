use crate::expr::func::Callee;
use std::cmp::{Ordering, PartialEq, PartialOrd};
use std::collections::VecDeque;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

pub struct DynFunc(pub Box<dyn Callee>);

impl Clone for DynFunc {
    fn clone(&self) -> Self {
        DynFunc(self.0.clone_box())
    }
}

impl PartialEq for DynFunc {
    fn eq(&self, other: &DynFunc) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl fmt::Debug for DynFunc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.dyn_debug(f)
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
    Deque(VecDeque<Val>),
    // special
    Func(DynFunc),
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{}", n),
            Self::Char(c) => write!(f, "{}", c),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Unit => write!(f, "📦🧑‍🦲"),
            Self::Break(val) => write!(f, "💔: {}", val.as_ref()),
            Self::Deque(vals) => Ok(for v in vals {
                write!(f, "{}", v)?;
            }),
            Self::Func(df) => write!(f, "{:?}", df),
        }
    }
}

impl Val {
    fn try_match_type(&self, other: &Self) -> Result<Self, String> {
        let err = format!("can't convert type `{}` to `{}`", "?", "?");

        use Val::*;
        match self {
            Deque(d) => match other {
                Deque(_) => Ok(Deque(d.clone())),
                _ => Err(err),
            }
            Char(c) => match other {
                Char(_) => Ok(Char(*c)),
                Number(_) => Ok(Number(*c as i32)),
                _ => Err(err),
            },
            Number(n) => match other {
                Char(_) => char::from_u32(*n as u32).map(|c| Char(c)).ok_or(err),
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
        match self {
            Self::Func(f) => Ok(f.clone()),
            _ => Err(format!("can't convert type `{}` to `{}`", "?", "?")),
        }
    }

    pub fn try_gt(&self, other: &Val) -> Result<Self, String> {
        if let Some(_) = self.partial_cmp(other) {
            Ok(Self::Bool(self > other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", "?", "?"))
        }
    }

    pub fn try_ge(&self, other: &Val) -> Result<Self, String> {
        if let Some(_) = self.partial_cmp(other) {
            Ok(Self::Bool(self >= other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", "?", "?"))
        }
    }

    pub fn try_eq(&self, other: &Val) -> Result<Self, String> {
        if let Some(_) = self.partial_cmp(other) {
            Ok(Self::Bool(!(self > other) && !(self < other)))
        } else {
            Err(format!("can't compare types `{}` and `{}`", "?", "?"))
        }
    }

    pub fn try_lt(&self, other: &Val) -> Result<Self, String> {
        if let Some(_) = self.partial_cmp(other) {
            Ok(Self::Bool(self < other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", "?", "?"))
        }
    }

    pub fn try_le(&self, other: &Val) -> Result<Self, String> {
        if let Some(_) = self.partial_cmp(other) {
            Ok(Self::Bool(self <= other))
        } else {
            Err(format!("can't compare types `{}` and `{}`", "?", "?"))
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
