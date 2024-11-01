use std::{collections::HashMap, io::Read};

use xml::{attribute::OwnedAttribute, reader::XmlEvent};

use crate::{
    error::{Error, ParseError},
    repr::Vulkan,
};

trait VecAttributesExt {
    type Target;

    fn to_hash_map(self) -> Self::Target;
}

impl VecAttributesExt for Vec<OwnedAttribute> {
    type Target = HashMap<String, String>;

    fn to_hash_map(self) -> HashMap<String, String> {
        let mut hash_map = HashMap::new();
        hash_map.extend(
            self.into_iter()
                .map(|attribute| (attribute.name.local_name, attribute.value)),
        );

        hash_map
    }
}

trait Parse {
    type Target: Sized;

    fn parse(
        self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<Self::Target, ParseError>;
}

trait AutoParse {
    const ELEMENT_NAME: &'static str;

    fn parse_attribs(&mut self, attributes: HashMap<String, String>) -> Result<(), ParseError> {
        _ = attributes;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        _ = reader;
        _ = attributes;
        Err(ParseError::UnexpChild(
            Self::ELEMENT_NAME.to_owned(),
            element_name,
        ))
    }

    fn parse_content(&mut self, content: String) -> Result<(), ParseError> {
        Err(ParseError::UnexpCont(
            Self::ELEMENT_NAME.to_owned(),
            content,
        ))
    }

    fn parse_misc(&mut self, event: XmlEvent) -> Result<(), ParseError> {
        _ = event;
        Ok(())
    }
}

impl<T> Parse for T
where
    T: AutoParse + Sized,
{
    type Target = Self;

    fn parse(
        mut self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<<Self as Parse>::Target, ParseError> {
        if element_name != Self::ELEMENT_NAME {
            return Err(ParseError::BadElemStart(
                Self::ELEMENT_NAME.to_owned(),
                element_name,
            ));
        }

        self.parse_attribs(attributes)?;

        loop {
            match reader.next()? {
                XmlEvent::EndDocument => {
                    return Err(ParseError::UnexpEnd(Self::ELEMENT_NAME.to_owned()))
                }
                XmlEvent::StartElement {
                    name, attributes, ..
                } => self.parse_child(reader, name.local_name, attributes.to_hash_map())?,
                XmlEvent::EndElement { name } => {
                    if name.local_name == Self::ELEMENT_NAME {
                        break;
                    }
                }
                XmlEvent::Characters(content) => self.parse_content(content)?,
                event => self.parse_misc(event)?,
            }
        }

        Ok(self)
    }
}

impl Vulkan {
    pub fn apply_xml(self, mut reader: xml::EventReader<impl Read>) -> Result<Self, Error> {
        self.inner_apply_xml(&mut reader)
            .map_err(|error| Error::parse(&reader, error))
    }

    fn inner_apply_xml(self, reader: &mut xml::EventReader<impl Read>) -> Result<Self, ParseError> {
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => break self.parse(reader, name.local_name, attributes.to_hash_map()),
                _ => {}
            }
        }
    }
}

impl AutoParse for Vulkan {
    const ELEMENT_NAME: &'static str = "registry";

    fn parse_child(
        &mut self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        match element_name.as_str() {
            Comment::ELEMENT_NAME => {
                Comment::default().parse(reader, element_name, attributes)?;
            }
            _ => {}
        };

        Ok(())
    }
}

#[derive(Default)]
pub struct Comment;

impl AutoParse for Comment {
    const ELEMENT_NAME: &'static str = "comment";

    fn parse_content(&mut self, _: String) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Platforms;

impl AutoParse for Platforms {
    const ELEMENT_NAME: &'static str = "platforms";

    fn parse_child(
        &mut self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Platform::default().parse(reader, element_name, attributes)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Platform;

impl AutoParse for Platform {
    const ELEMENT_NAME: &'static str = "platform";
}

#[derive(Default)]
pub struct Tags;

impl AutoParse for Tags {
    const ELEMENT_NAME: &'static str = "tags";

    fn parse_child(
        &mut self,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Tags::default().parse(reader, element_name, attributes)?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Tag;

impl AutoParse for Tag {
    const ELEMENT_NAME: &'static str = "tag";
}
