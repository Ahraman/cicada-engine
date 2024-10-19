#[derive(Debug, Clone)]
pub struct BackendError(pub windows::core::Error);

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for BackendError {}

impl From<windows::core::Error> for BackendError {
    fn from(value: windows::core::Error) -> Self {
        Self(value)
    }
}
