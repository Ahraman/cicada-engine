use std::io::Read;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::error::Error;

pub fn parse_vk_xml<R>(reader: R) -> Result<Registry, Error>
where
    R: Read,
{
    let mut reader = EventReader::new(reader);
    loop {
        match reader.next()? {
            XmlEvent::StartDocument { .. } => {}
            XmlEvent::EndDocument => return Err(Error::DocEnd),
            XmlEvent::StartElement { name, .. } => {
                if name.local_name == Registry::TAG {
                    return Registry::parse(&mut reader);
                } else {
                    return Err(Error::BadElemStart(name.local_name));
                }
            }
            XmlEvent::EndElement { name } => return Err(Error::BadElemEnd(name.local_name)),

            _ => continue,
        }
    }
}

pub trait Element {
    const TAG: &'static str;
}

#[derive(Debug, Default)]
pub struct Comment {
    pub text: String,
}
impl Comment {
    fn parse<R>(reader: &mut EventReader<R>) -> Result<Comment, Error>
    where
        R: Read,
    {
        let mut comment = Self::default();
        loop {
            match reader.next()? {
                XmlEvent::Characters(text) => comment.text += &text,
                XmlEvent::StartElement { name, .. } => {
                    return Err(Error::BadElemStart(name.local_name))
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    } else {
                        return Err(Error::BadElemEnd(name.local_name));
                    }
                }
                XmlEvent::EndDocument => return Err(Error::DocEnd),

                _ => continue,
            }
        }

        Ok(comment)
    }
}

impl Element for Comment {
    const TAG: &'static str = "comment";
}

#[derive(Debug, Default)]
pub struct Registry {
    // items
    pub comment: Option<Comment>,
    pub items: Vec<RegistryItem>,
}

impl Registry {
    pub fn parse<R>(reader: &mut EventReader<R>) -> Result<Self, Error>
    where
        R: Read,
    {
        let mut registry = Self::default();

        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    if name.local_name == Comment::TAG {
                        if let None = registry.comment {
                            registry.comment = Some(Comment::parse(reader)?);
                        } else {
                            return Err(Error::BadElemStart(name.local_name));
                        }
                    }

                    registry
                        .items
                        .push(RegistryItem::parse(reader, &name.local_name, &attributes)?)
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::TAG {
                        break;
                    }
                }
                XmlEvent::EndDocument => return Err(Error::DocEnd),

                _ => continue,
            }
        }

        Ok(registry)
    }
}

impl Element for Registry {
    const TAG: &'static str = "registry";
}

#[derive(Debug)]
pub enum RegistryItem {
    Platforms(Platforms),
    Tags(Tags),
    Types(Types),
    Enums(Enums),
    Commands(Commands),
    Feature(Feature),
    Extensions(Extensions),
    Formats(Formats),
    Sync(Sync),
    VideoCodecs(VideoCodecs),
    SpirvExtensions(SpirvExtensions),
    SpirvCapabilities(SpirvCapabilities),
}
impl RegistryItem {
    fn parse<R>(
        reader: &mut EventReader<R>,
        local_name: &str,
        attributes: &[OwnedAttribute],
    ) -> Result<Self, Error>
    where
        R: Read,
    {
        match reader.next()? {
            XmlEvent::StartDocument {
                version,
                encoding,
                standalone,
            } => todo!(),
            XmlEvent::EndDocument => todo!(),
            XmlEvent::ProcessingInstruction { name, data } => todo!(),
            XmlEvent::StartElement {
                name,
                attributes,
                namespace,
            } => todo!(),
            XmlEvent::EndElement { name } => todo!(),
            XmlEvent::CData(_) => todo!(),
            XmlEvent::Comment(_) => todo!(),
            XmlEvent::Characters(_) => todo!(),
            XmlEvent::Whitespace(_) => todo!(),
        }
        todo!()
    }
}

#[derive(Debug, Default)]
pub struct Platforms {}

impl Element for Platforms {
    const TAG: &'static str = "platforms";
}

#[derive(Debug, Default)]
pub struct Tags {}

impl Element for Tags {
    const TAG: &'static str = "tags";
}

#[derive(Debug, Default)]
pub struct Types {}

impl Element for Types {
    const TAG: &'static str = "types";
}

#[derive(Debug, Default)]
pub struct Enums {}

impl Element for Enums {
    const TAG: &'static str = "enums";
}

#[derive(Debug, Default)]
pub struct Commands {}

impl Element for Commands {
    const TAG: &'static str = "commands";
}

#[derive(Debug, Default)]
pub struct Feature {}

impl Element for Feature {
    const TAG: &'static str = "feature";
}

#[derive(Debug, Default)]
pub struct Extensions {}

impl Element for Extensions {
    const TAG: &'static str = "extensions";
}

#[derive(Debug, Default)]
pub struct Formats {}

impl Element for Formats {
    const TAG: &'static str = "formats";
}

#[derive(Debug, Default)]
pub struct Sync {}

impl Element for Sync {
    const TAG: &'static str = "sync";
}

#[derive(Debug, Default)]
pub struct VideoCodecs {}

impl Element for VideoCodecs {
    const TAG: &'static str = "videocodecs";
}

#[derive(Debug, Default)]
pub struct SpirvExtensions {}

impl Element for SpirvExtensions {
    const TAG: &'static str = "spirvextensions";
}

#[derive(Debug, Default)]
pub struct SpirvCapabilities {}

impl Element for SpirvCapabilities {
    const TAG: &'static str = "spirvcapabilities";
}
