#[derive(Debug, Clone)]
pub struct Error(pub windows::core::Error);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for Error {}

impl From<windows::core::Error> for Error {
    fn from(value: windows::core::Error) -> Self {
        Self(value)
    }
}
