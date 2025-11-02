use std::string::FromUtf8Error;

pub type MzResult<T> = Result<T, MzError>;

#[derive(Debug)]
pub enum MzError {
    None,
    Io(std::io::Error),
    MissingHeader,
    FileTooSmall,
    Utf(FromUtf8Error),
}

impl std::error::Error for MzError{}

impl std::fmt::Display for MzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for MzError {
    fn from(t: std::io::Error) -> Self {
        MzError::Io(t)
    }
}

type T = FromUtf8Error;
impl From<T> for MzError {
    fn from(t: T) -> Self {
        MzError::Utf(t)
    }
}
