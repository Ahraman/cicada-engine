use std::{io::Read, path::PathBuf};

use cicada_vkgen::{error::Error, loader::load_xml_registry, Settings};

fn main() -> Result<(), Error> {
    let settings = read_command_line();
    let mut reader = load_xml_registry(&settings)?;

    let mut buf = [0u8; 128];
    reader.read_exact(&mut buf)?;

    println!("{}", String::from_utf8(buf.to_vec()).unwrap());

    Ok(())
}

fn read_command_line() -> Settings {
    Settings {
        local_path: PathBuf::from("vk.xml"),
        force_update: false,
        registry_url: String::from(
            "https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/refs/heads/main/xml/vk.xml",
        ),
    }
}
