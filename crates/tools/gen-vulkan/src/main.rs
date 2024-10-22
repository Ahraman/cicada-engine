use std::path::PathBuf;

use gen_vulkan::{error::Error, load::load_xml_registry, parse::parse_vk_xml, Settings};

fn main() -> Result<(), Error> {
    let settings = read_command_line()?;
    let reader = load_xml_registry(&settings)?;
    let registry = parse_vk_xml(reader)?;

    println!("{registry:?}");

    Ok(())
}

fn read_command_line() -> Result<Settings, Error> {
    let mut args = std::env::args();

    let mut local_path = PathBuf::from("vk.xml");
    let mut force_update = false;
    let mut registry_url = String::from(
        "https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/refs/heads/main/xml/vk.xml",
    );

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--path" => {
                local_path = match args.next() {
                    Some(path) => PathBuf::from(path),
                    None => return Err(Error::BadArg(arg, String::new())),
                }
            }
            "--url" => {
                registry_url = match args.next() {
                    Some(url) => url,
                    None => return Err(Error::BadArg(arg, String::new())),
                }
            }
            "--update" => force_update = true,

            _ => {}
        }
    }

    Ok(Settings {
        local_path,
        force_update,
        registry_url,
    })
}
