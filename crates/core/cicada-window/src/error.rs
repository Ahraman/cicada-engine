#[derive(Debug, Clone)]
pub struct WindowError(windows::core::Error);

impl std::error::Error for WindowError {}

impl std::fmt::Display for WindowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<windows::core::Error> for WindowError {
    fn from(value: windows::core::Error) -> Self {
        Self(value)
    }
}
