use crate::utils::kwords;
use thiserror::Error;

#[derive(Error, Clone, Debug, PartialEq)]
pub enum ParseError {
    #[error("Expected digits")]
    ExpectedDigits,
    #[error("Expected whitespace")]
    ExpectedWhitespace,
    #[error("Expected identifier")]
    ExpectedIdent,
    #[error("Expected tag {0}")]
    ExpectedTag(&'static str),
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Expected boolean")]
    ExpectedBool,
    #[error("Unexpected consequtive equality")]
    UnexpectedEquals,
    #[error("Expected {} or {}", kwords::LET, kwords::SET)]
    ExpectedBindingUpdate,
    #[error("Expected {}", kwords::INDEX)]
    ExpectedIndex,
    #[error("Expected expression")]
    ExpectedExpr,
}

#[derive(Error, Clone, Debug, PartialEq)]
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
}

impl<'a> From<&'a RuntimeError> for RuntimeError {
    fn from(re: &'a RuntimeError) -> Self {
        re.clone()
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
