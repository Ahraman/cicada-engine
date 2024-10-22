use std::path::PathBuf;

pub mod error;
pub mod load;

pub struct Settings {
    pub local_path: PathBuf,

    pub force_update: bool,
    pub registry_url: String,
}
