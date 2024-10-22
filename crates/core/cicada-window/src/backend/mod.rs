#[cfg(target_os = "windows")]
#[path ="windows/mod.rs"]
mod inner;

#[cfg(target_os = "windows")]
pub use inner::*;
