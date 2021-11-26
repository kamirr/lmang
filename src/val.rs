use crate::expr::func::Func;
use std::cmp::{Ordering, PartialOrd};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Debug, PartialEq, Clone)]
pub enum Val {
    Number(i32),
    Bool(bool),
    Break(Box<Val>),
    Func(Func),
    Unit,
}

impl Val {
    fn try_match_type(&self, other: &Self) -> Result<Self, String> {
        let err = Err(format!("can't convert type `{}` to `{}`", "?", "?"));

        use Val::*;
        match self {
            Number(n) => match other {
                Number(_) => Ok(Number(*n)),
                _ => err,
            },
            Bool(b) => match other {
                Bool(_) => Ok(Bool(*b)),
                _ => err,
            },
            Unit => match other {
                Unit => Ok(Val::Unit),
                _ => err,
            },
            _ => err,
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
