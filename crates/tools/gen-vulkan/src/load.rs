use std::{
    fs::{self, File},
    io::{BufReader, Read},
};

use crate::{error::Error, Settings};

pub fn load_xml_registry(settings: &Settings) -> Result<impl Read, Error> {
    if settings.force_update || !fs::exists(&settings.local_path)? {
        let response = ureq::get(&settings.registry_url).call()?;
        std::fs::write(&settings.local_path, response.into_string()?)?;
    }

    Ok(BufReader::new(File::open(&settings.local_path)?))
}
