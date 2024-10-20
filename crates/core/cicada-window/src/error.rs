use crate::backend::error::Error as BackendError;

#[derive(Debug, Clone)]
pub struct Error(BackendError);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<BackendError> for Error {
    fn from(value: BackendError) -> Self {
        Self(value)
    }
}
