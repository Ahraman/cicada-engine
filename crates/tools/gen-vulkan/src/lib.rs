use repr::Vulkan;

use crate::{emit::EmitSettings, error::Error, load::load_file, parse::ParseSettings};

pub mod emit;
pub mod error;
pub mod load;
pub mod parse;
pub mod repr;
pub mod trans;

#[derive(Default)]
pub struct Settings {
    pub parse: ParseSettings,
    pub emit: EmitSettings,
}

pub fn run(settings: &Settings) -> Result<(), Error> {
    Vulkan::init(&settings.parse)?.emit(&settings.emit)
}

impl Vulkan {
    pub fn init(settings: &ParseSettings) -> Result<Self, Error> {
        let reader = xml::EventReader::new(load_file("vk.xml", settings)?);
        Vulkan::from_xml(reader)
    }
}
