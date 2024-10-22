use std::path::PathBuf;

pub mod error;
pub mod load;
pub mod parse;

pub struct Settings {
    pub local_path: PathBuf,

    pub force_update: bool,
    pub registry_url: String,
}
