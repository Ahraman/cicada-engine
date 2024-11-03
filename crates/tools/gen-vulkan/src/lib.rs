use emit::EmitSettings;
use parse::ParseSettings;
use repr::Vulkan;

use crate::{error::Error, load::load_file};

pub mod emit;
pub mod error;
pub mod load;
pub mod parse;
pub mod repr;

pub fn run(parse: &ParseSettings, emit: &EmitSettings) -> Result<(), Error> {
    Vulkan::init(parse)?.emit(emit)
}

impl Vulkan {
    pub fn init(settings: &ParseSettings) -> Result<Vulkan, Error> {
        let reader = xml::EventReader::new(load_file("vk.xml", settings)?);
        Vulkan::from_xml(reader)
    }
}
