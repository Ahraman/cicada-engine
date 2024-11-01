use repr::Vulkan;

use crate::{error::Error, load::load_file};

pub mod emit;
pub mod error;
pub mod load;
pub mod parse;
pub mod repr;

#[derive(Default)]
pub struct Settings {
    pub path: Option<String>,
    pub no_video: bool,
}

pub fn run(settings: &Settings) -> Result<(), Error> {
    init_vulkan(settings)?;
    if !settings.no_video {
        init_video(settings)?;
    }

    Ok(())
}

pub fn init_vulkan(settings: &Settings) -> Result<(), Error> {
    let reader = xml::EventReader::new(load_file("vk.xml", settings)?);
    let _vk = Vulkan::default().apply_xml(reader)?;

    Ok(())
}

pub fn init_video(settings: &Settings) -> Result<(), Error> {
    let _reader = load_file("video.xml", settings)?;

    Ok(())
}
