use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Display, Formatter};
use joker::track::{Span};

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    InvalidAssignTarget(Option<Span>),
    InvalidPropPatt(Option<Span>)
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::InvalidAssignTarget(_) => "invalid assignment pattern",
            Error::InvalidPropPatt(_) => "invalid object property in assignment pattern",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        None
    }
}

