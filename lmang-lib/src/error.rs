use crate::utils::kwords;
use crate::val::{Object, Val};
use std::fmt;
use strum::AsRefStr;
use thiserror::Error;

#[cfg(feature = "web")]
use wasm_bindgen::JsValue;

#[derive(Error, Clone, Debug, PartialEq, AsRefStr)]
pub enum ParseError {
    #[error("Expected digits")]
    ExpectedDigits,
    #[error("Expected whitespace")]
    ExpectedWhitespace,
    #[error("Expected identifier")]
    ExpectedIdent,
    #[error("Expected tag {0}")]
    ExpectedTag(&'static str),
    #[error("Expected boolean")]
    ExpectedBool,
    #[error("Expected {} or {}", kwords::LET, kwords::SET)]
    ExpectedBindingUpdate,
    #[error("Expected {}", kwords::INDEX)]
    ExpectedIndex,
    #[error("Expected expression")]
    ExpectedExpr,
    #[error("Variadic must be the last argument")]
    PrematureVariadic,
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Unexpected consequtive equality")]
    UnexpectedEquals,
}

impl Object for ParseError {
    fn member_names(&self) -> Vec<String> {
        ["type"]
            .into_iter()
            .chain({
                use ParseError::*;
                match self {
                    ExpectedTag(_) => vec!["expectedTag"],
                    _ => vec![],
                }
                .into_iter()
            })
            .map(|s| s.to_string())
            .collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        if name == "type" {
            Ok(Val::from_str(self.as_ref()))
        } else {
            use ParseError::*;
            match (name, self) {
                ("expectedTag", ExpectedTag(s)) => Ok(Val::from_str(s)),
                _ => Err(RuntimeError::NoKey(name.into())),
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn name(&self) -> &str {
        "ParseError"
    }
}

#[derive(Error, Clone, Debug, PartialEq, AsRefStr)]
pub enum RuntimeError {
    #[error("Binding {0} doesn't exist")]
    NoBinding(String),
    #[error("Timeout")]
    Timeout,
    #[error("Out of bounds access: {idx}, size is: {len}")]
    OutOfBounds { idx: i32, len: usize },
    #[error("Invalid number of arguments")]
    WrongArgsN,
    #[error("Can't cast from {from} to {to}")]
    CastError { from: String, to: String },
    #[error("Invalid operation {lhs} {op}, {rhs}")]
    InvalidOp {
        lhs: String,
        op: String,
        rhs: String,
    },
    #[error("Dangling weak pointer")]
    Dangling,
    #[error("Can't open file {file}, reason: {reason}")]
    IoError { file: String, reason: String },
    #[error("Invalid handle {0}")]
    NoHandle(i32),
    #[error("No key {0}")]
    NoKey(String),
    #[cfg(feature = "web")]
    #[error("Js error {:?}", .0)]
    JsError(JsValue),
}

impl Object for RuntimeError {
    fn member_names(&self) -> Vec<String> {
        ["type"]
            .into_iter()
            .chain({
                use RuntimeError::*;
                match self {
                    NoBinding(_) => vec!["binding"],
                    OutOfBounds { .. } => vec!["idx", "len"],
                    CastError { .. } => vec!["from", "to"],
                    InvalidOp { .. } => vec!["lhs", "op", "rhs"],
                    IoError { .. } => vec!["file", "reason"],
                    NoHandle(_) => vec!["handle"],
                    NoKey(_) => vec!["key"],
                    #[cfg(feature = "web")]
                    JsError(_) => vec!["jsError"],
                    _ => vec![],
                }
                .into_iter()
            })
            .map(|s| s.to_string())
            .collect()
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        if name == "type" {
            Ok(Val::from_str(self.as_ref()))
        } else {
            use RuntimeError::*;
            match (name, self) {
                ("binding", NoBinding(s)) => Ok(Val::from_str(&*s)),
                ("idx", OutOfBounds { idx, .. }) => Ok(Val::Number(*idx)),
                ("len", OutOfBounds { len, .. }) => Ok(Val::Number(*len as i32)),
                ("from", CastError { from, .. }) => Ok(Val::from_str(&*from)),
                ("to", CastError { to, .. }) => Ok(Val::from_str(&*to)),
                ("lhs", InvalidOp { lhs, .. }) => Ok(Val::from_str(&*lhs)),
                ("op", InvalidOp { op, .. }) => Ok(Val::from_str(&*op)),
                ("rhs", InvalidOp { rhs, .. }) => Ok(Val::from_str(&*rhs)),
                ("file", IoError { file, .. }) => Ok(Val::from_str(&*file)),
                ("reason", IoError { reason, .. }) => Ok(Val::from_str(&*reason)),
                ("handle", NoHandle(handle)) => Ok(Val::Number(*handle)),
                ("key", NoKey(key)) => Ok(Val::from_str(&*key)),
                #[cfg(feature = "web")]
                ("jsError", JsError(jv)) => Ok(Val::JsValue(jv.clone())),
                _ => Err(RuntimeError::NoKey(name.into())),
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn name(&self) -> &str {
        "RuntimeError"
    }
}

impl<'a> From<&'a RuntimeError> for RuntimeError {
    fn from(re: &'a RuntimeError) -> Self {
        re.clone()
    }
}

#[cfg(feature = "web")]
impl From<JsValue> for RuntimeError {
    fn from(jv: JsValue) -> Self {
        Self::JsError(jv)
    }
}

impl From<RuntimeError> for String {
    fn from(err: RuntimeError) -> String {
        format!("{}", err)
    }
}

#[derive(Error, Clone, Debug, PartialEq)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(ParseError),
    #[error("Runtime error: {0}")]
    Runtime(RuntimeError),
}

impl Object for Error {
    fn member_names(&self) -> Vec<String> {
        match self {
            Error::Parse(pe) => pe.member_names(),
            Error::Runtime(re) => re.member_names(),
        }
    }

    fn member(&self, name: &str) -> Result<Val, RuntimeError> {
        if name == "type" {
            match self {
                Error::Parse(_) => Ok(Val::from_str("Parse")),
                Error::Runtime(_) => Ok(Val::from_str("Runtime")),
            }
        } else {
            match self {
                Error::Parse(pe) => pe.member(name),
                Error::Runtime(re) => re.member(name),
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Object> {
        Box::new(self.clone())
    }

    fn dyn_debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }

    fn name(&self) -> &str {
        "Error"
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::Parse(err)
    }
}

impl From<RuntimeError> for Error {
    fn from(err: RuntimeError) -> Self {
        Self::Runtime(err)
    }
}
