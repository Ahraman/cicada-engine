use std::{collections::HashMap, io::Read, str::FromStr};

use xml::{attribute::OwnedAttribute, reader::XmlEvent};

use crate::{
    error::{Error, ParseError},
    repr::{Deprecation, ImportedBody, IncludeBody, Type, TypeBody, TypeHandle, TypeHead, Vulkan},
};

trait VecAttributesExt {
    type Target;

    fn into_hash_map(self) -> Self::Target;
}

impl VecAttributesExt for Vec<OwnedAttribute> {
    type Target = HashMap<String, String>;

    fn into_hash_map(self) -> HashMap<String, String> {
        let mut hash_map = HashMap::new();
        hash_map.extend(
            self.into_iter()
                .map(|attribute| (attribute.name.local_name, attribute.value)),
        );

        hash_map
    }
}

impl Deprecation {
    const ELEMENT: &'static str = "deprecated";
}

impl FromStr for Deprecation {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Ok(match s {
            "false" => Self::False,
            "true" => Self::True,
            "aliased" => Self::Aliased,
            "ignored" => Self::Ignored,
            s => {
                return Err(ParseError::BadDeprAttrib(
                    Self::ELEMENT.to_string(),
                    s.to_string(),
                ))
            }
        })
    }
}

#[derive(Default)]
pub struct ParseSettings {
    pub path: Option<String>,
}

impl Vulkan {
    pub fn from_xml(mut reader: xml::EventReader<impl Read>) -> Result<Self, Error> {
        Self::default()
            .parse_xml(&mut reader)
            .map_err(|error| Error::parse(&reader, error))
    }

    fn parse_xml(mut self, reader: &mut xml::EventReader<impl Read>) -> Result<Self, ParseError> {
        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    _ = Regist::parse(
                        &mut self,
                        reader,
                        name.local_name,
                        attributes.into_hash_map(),
                    )?
                }
                XmlEvent::EndDocument => break,
                XmlEvent::EndElement { name } => {
                    return Err(ParseError::UnexpEnd(Default::default(), name.local_name))
                }
                XmlEvent::Characters(content) => {
                    return Err(ParseError::UnexpCont(Default::default(), content))
                }
                _ => {}
            }
        }

        Ok(self)
    }
}

trait Parse {
    type Target;

    fn parse(
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: HashMap<String, String>,
    ) -> Result<Self::Target, ParseError>;
}

trait BasicParse {
    const NAME: &'static str;

    fn parse_attribs(
        &mut self,
        vk: &mut Vulkan,
        attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        _ = vk;
        _ = attribs;
        Ok(())
    }

    fn parse_child(
        &mut self,
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        _ = vk;
        _ = reader;
        _ = attribs;
        Err(ParseError::UnexpStart(Self::NAME.to_string(), element))
    }

    fn parse_cont(&mut self, vk: &mut Vulkan, content: String) -> Result<(), ParseError> {
        _ = vk;
        Err(ParseError::UnexpCont(Self::NAME.to_string(), content))
    }

    fn parse_misc(&mut self, vk: &mut Vulkan, event: XmlEvent) -> Result<(), ParseError> {
        _ = vk;
        _ = event;
        Ok(())
    }
}

impl<T> Parse for T
where
    T: BasicParse + Default,
{
    type Target = Self;

    fn parse(
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        let mut item = Self::default();

        item.parse_attribs(vk, attribs)?;

        loop {
            match reader.next()? {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => item.parse_child(vk, reader, name.local_name, attributes.into_hash_map())?,
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    }
                }
                XmlEvent::EndDocument => return Err(ParseError::DocEnd(element)),
                XmlEvent::Characters(content) => item.parse_cont(vk, content)?,

                event => item.parse_misc(vk, event)?,
            }
        }

        Ok(item)
    }
}

#[derive(Default)]
struct Regist;

impl BasicParse for Regist {
    const NAME: &'static str = "registry";

    fn parse_child(
        &mut self,
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        match element.as_str() {
            Comment::NAME => _ = Comment::parse(vk, reader, element, attribs)?,
            ParsePlatforms::NAME => _ = ParsePlatforms::parse(vk, reader, element, attribs)?,
            Tags::NAME => _ = Tags::parse(vk, reader, element, attribs)?,
            ParseTypes::NAME => _ = ParseTypes::parse(vk, reader, element, attribs)?,
            _ => {}
        }

        Ok(())
    }
}

#[derive(Default)]
struct Comment;

impl BasicParse for Comment {
    const NAME: &'static str = "comment";

    fn parse_cont(&mut self, _: &mut Vulkan, _: String) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
struct ParsePlatforms;

impl BasicParse for ParsePlatforms {
    const NAME: &'static str = "platforms";

    fn parse_child(
        &mut self,
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let _ = ParsePlatform::parse(vk, reader, element_name, attributes)?;
        Ok(())
    }
}

#[derive(Default)]
struct ParsePlatform;

impl BasicParse for ParsePlatform {
    const NAME: &'static str = "platform";
}

#[derive(Default)]
struct Tags;

impl BasicParse for Tags {
    const NAME: &'static str = "tags";

    fn parse_child(
        &mut self,
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element_name: String,
        attributes: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let _ = Tag::parse(vk, reader, element_name, attributes)?;
        Ok(())
    }
}

#[derive(Default)]
struct Tag;

impl BasicParse for Tag {
    const NAME: &'static str = "tag";
}

#[derive(Default)]
struct ParseTypes;

impl BasicParse for ParseTypes {
    const NAME: &'static str = "types";

    fn parse_child(
        &mut self,
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        match element.as_str() {
            Comment::NAME => _ = Comment::parse(vk, reader, element, attribs)?,
            _ => {
                let item = ParseType::parse(vk, reader, element, attribs)?;
                _ = vk.types.insert(item);
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct ParseType;

impl ParseType {
    const NAME: &'static str = "type";
}

impl Parse for ParseType {
    type Target = Type;

    fn parse(
        vk: &mut Vulkan,
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        mut attribs: HashMap<String, String>,
    ) -> Result<Type, ParseError> {
        Ok(match attribs.remove("category") {
            Some(category) => match category.as_str() {
                ParsedBaseType::CATEGORY => {
                    ParsedBaseType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedBitmaskType::CATEGORY => {
                    ParsedBitmaskType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedDefineType::CATEGORY => {
                    ParsedDefineType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedEnumType::CATEGORY => {
                    ParsedEnumType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedFnPtrType::CATEGORY => {
                    ParsedFnPtrType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedHandleType::CATEGORY => {
                    ParsedHandleType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedIncludeType::CATEGORY => {
                    ParsedIncludeType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedStructType::CATEGORY => {
                    ParsedStructType::parse(vk, reader, element, attribs)?.into()
                }
                ParsedUnionType::CATEGORY => {
                    ParsedUnionType::parse(vk, reader, element, attribs)?.into()
                }
                _ => {
                    return Err(ParseError::BadAttrb(
                        Self::NAME.to_string(),
                        "category".to_string(),
                        category,
                    ))
                }
            },
            None => ParsedImportedType::parse(vk, reader, element, attribs)?.into(),
        })
    }
}

pub struct ParsedTypeHead {
    pub standard_name: Option<String>,
    pub requires: Option<TypeHandle>,
    pub deprecated: Option<Deprecation>,
}

impl ParsedTypeHead {
    fn from_attribs(
        vk: &Vulkan,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        let standard_name = attribs.remove("name");
        let requires = attribs
            .remove("requires")
            .map(|s| vk.types.find(&s))
            .flatten();
        let deprecated = attribs
            .remove("deprecated")
            .map(|s| Deprecation::from_str(&s))
            .transpose()?;

        Ok(Self {
            standard_name,
            requires,
            deprecated,
        })
    }

    fn into_head<F>(self, transform: F) -> Result<TypeHead, ParseError>
    where
        F: FnOnce(&str) -> String,
    {
        match self.standard_name {
            Some(standard_name) => {
                let output_name = transform(&standard_name);
                Ok(TypeHead::new(
                    standard_name,
                    output_name,
                    self.requires,
                    self.deprecated,
                ))
            }
            None => Err(ParseError::BadType),
        }
    }
}

#[derive(Default)]
struct ParsedIncludeType {
    pub head: TypeHead,
    pub body: IncludeBody,
}

impl ParsedIncludeType {
    const CATEGORY: &'static str = "include";
}

impl BasicParse for ParsedIncludeType {
    const NAME: &'static str = ParseType::NAME;

    fn parse_attribs(
        &mut self,
        vk: &mut Vulkan,
        mut attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.head =
            ParsedTypeHead::from_attribs(vk, &mut attribs)?.into_head(|_| Default::default())?;
        self.body.header_name = self.head.standard_name.clone();

        Ok(())
    }

    fn parse_cont(&mut self, _: &mut Vulkan, content: String) -> Result<(), ParseError> {
        let content = content.trim();
        if content.starts_with("#include") {
            if let Some(header) = content.split(' ').last() {
                if header.starts_with('"') {
                    self.body.header_name = header[1..(header.len() - 1)].to_string();
                    self.body.is_local = true;
                } else if header.starts_with('<') {
                    self.body.header_name = header[1..(header.len() - 1)].to_string();
                }
            }
        }

        Ok(())
    }
}

impl Into<Type> for ParsedIncludeType {
    fn into(self) -> Type {
        Type::new(self.head, TypeBody::Include(self.body))
    }
}

#[derive(Default)]
struct ParsedImportedType {
    pub head: TypeHead,
    pub body: ImportedBody,
}

impl BasicParse for ParsedImportedType {
    const NAME: &'static str = ParseType::NAME;

    fn parse_attribs(
        &mut self,
        vk: &mut Vulkan,
        mut attribs: HashMap<String, String>,
    ) -> Result<(), ParseError> {
        const OUTPUT_NAMES: &[(&'static str, &'static str)] = &[
            // C types
            ("int", "std::ffi::c_int"),
            ("void", "std::ffi::c_void"),
            ("char", "std::ffi::c_char"),
            ("float", "f32"),
            ("double", "f64"),
            ("int8_t", "i8"),
            ("int16_t", "i16"),
            ("int32_t", "i32"),
            ("int64_t", "i64"),
            ("uint8_t", "u8"),
            ("uint16_t", "u16"),
            ("uint32_t", "u32"),
            ("uint64_t", "u64"),
            ("size_t", "usize"),
            // windows
            ("HINSTANCE", "Win32::Foundation::HINSTANCE"),
            ("HWND", "Win32::Foundation::HWND"),
            ("HMONITOR", "Win32::Graphics::Gdi::HMONITOR"),
            ("HANDLE", "Win32::Foundation::HANDLE"),
            (
                "SECURITY_ATTRIBUTES",
                "Win32::Security::SECURITY_ATTRIBUTES",
            ),
            ("DWORD", "u32"),
            // TODO: make sure LPCWSTR vs PCWSTR is the same thing
            ("LPCWSTR", "windows::core::PCWSTR"),
        ];

        self.head = ParsedTypeHead::from_attribs(vk, &mut attribs)?.into_head(|standard_name| {
            OUTPUT_NAMES
                .iter()
                .find(|(key, _)| standard_name == *key)
                .map(|(_, value)| value.to_string())
                .unwrap_or_else(|| standard_name.to_string())
        })?;
        Ok(())
    }
}

impl Into<Type> for ParsedImportedType {
    fn into(self) -> Type {
        Type::new(self.head, TypeBody::Imported(self.body))
    }
}

#[derive(Default)]
struct ParsedDefineType {}

impl ParsedDefineType {
    const CATEGORY: &'static str = "define";
}

impl BasicParse for ParsedDefineType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedDefineType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedBaseType {}

impl ParsedBaseType {
    const CATEGORY: &'static str = "basetype";
}

impl BasicParse for ParsedBaseType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedBaseType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedBitmaskType {}

impl ParsedBitmaskType {
    const CATEGORY: &'static str = "bitmask";
}

impl BasicParse for ParsedBitmaskType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedBitmaskType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedHandleType {}

impl ParsedHandleType {
    const CATEGORY: &'static str = "handle";
}

impl BasicParse for ParsedHandleType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedHandleType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedEnumType {}

impl ParsedEnumType {
    const CATEGORY: &'static str = "enum";
}

impl BasicParse for ParsedEnumType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedEnumType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedFnPtrType {}

impl ParsedFnPtrType {
    const CATEGORY: &'static str = "funcpointer";
}

impl BasicParse for ParsedFnPtrType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedFnPtrType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedStructType {}

impl ParsedStructType {
    const CATEGORY: &'static str = "struct";
}

impl BasicParse for ParsedStructType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedStructType {
    fn into(self) -> Type {
        todo!()
    }
}

#[derive(Default)]
struct ParsedUnionType {}

impl ParsedUnionType {
    const CATEGORY: &'static str = "union";
}

impl BasicParse for ParsedUnionType {
    const NAME: &'static str = ParseType::NAME;
}

impl Into<Type> for ParsedUnionType {
    fn into(self) -> Type {
        todo!()
    }
}
