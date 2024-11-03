use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{error::Error, ParseSettings};

pub fn load_file(file: &str, settings: &ParseSettings) -> Result<impl Read, Error> {
    let file = File::open(format!(
        "{}/{file}",
        settings.path.as_deref().unwrap_or("temp/registry/vulkan")
    ))?;

    Ok(BufReader::new(file))
}

pub struct Url {
    pub scheme: String,
    pub host: String,
    pub path: String,
}
