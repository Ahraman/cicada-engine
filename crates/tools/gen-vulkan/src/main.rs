use std::{io::Read, path::PathBuf};

use cicada_vkgen::{error::Error, loader::load_xml_registry, Settings};

fn main() -> Result<(), Error> {
    let settings = read_command_line()?;
    let mut reader = load_xml_registry(&settings)?;

    let mut buf = [0u8; 128];
    reader.read_exact(&mut buf)?;

    println!("{}", String::from_utf8(buf.to_vec()).unwrap());

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
