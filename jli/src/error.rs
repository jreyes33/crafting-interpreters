use crate::object::Object;
use std::fmt;
use std::rc::Rc;

type BoxedError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub enum Error {
    DynError(BoxedError),
    Return(Rc<dyn Object>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DynError(e) => write!(f, "{}", e),
            Self::Return(r) => write!(f, "<return {}>", r),
        }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::DynError(value.into())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::DynError(value.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::DynError(error.into())
    }
}
