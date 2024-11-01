use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{error::Error, Settings};

pub fn load_file(file: &str, settings: &Settings) -> Result<impl Read, Error> {
    let file = File::open(format!(
        "{}/{file}",
        settings.path.as_deref().unwrap_or("temp/vulkan")
    ))?;

    Ok(BufReader::new(file))
}

pub struct Url {
    pub scheme: String,
    pub host: String,
    pub path: String,
}
