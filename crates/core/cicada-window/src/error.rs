use crate::backend;

#[derive(Debug)]
pub struct Error(backend::Error);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<backend::Error> for Error {
    fn from(value: backend::Error) -> Self {
        Self(value)
    }
}
