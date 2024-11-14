use std::{
    collections::HashMap,
    io::Read,
    iter::Peekable,
    str::{Chars, FromStr},
};

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

use crate::{
    error::{Error, ParseError},
    repr::{Deprecation, Vulkan},
};

#[derive(Default)]
pub struct ParseSettings {
    pub path: Option<String>,
}

trait IntoAttributeMapExt {
    fn into_hash_map(self) -> HashMap<String, String>;
}

impl IntoAttributeMapExt for Vec<OwnedAttribute> {
    fn into_hash_map(self) -> HashMap<String, String> {
        HashMap::from_iter(
            self.into_iter()
                .map(|attrib| (attrib.name.local_name, attrib.value)),
        )
    }
}

trait AttributeMapExt {
    fn req_attrib(&mut self, element: &str, attrib: &str) -> Result<String, ParseError>;

    fn try_get<T>(&mut self, element: &str, attrib: &str) -> Result<Option<T>, ParseError>
    where
        T: FromStr;
}

impl AttributeMapExt for HashMap<String, String> {
    fn req_attrib(&mut self, element: &str, attrib: &str) -> Result<String, ParseError> {
        self.remove(attrib).ok_or(ParseError::ReqAttrib(
            element.to_string(),
            attrib.to_string(),
        ))
    }

    fn try_get<T>(&mut self, element: &str, attrib: &str) -> Result<Option<T>, ParseError>
    where
        T: FromStr,
    {
        self.remove(attrib)
            .map(|value| {
                T::from_str(&value).map_err(|_| {
                    ParseError::BadAttrib(element.to_string(), attrib.to_string(), value)
                })
            })
            .transpose()
    }
}

pub trait Parse: Sized {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError>;
}

pub trait GenericParse: Sized {
    const NAME: &'static str;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        _ = element;
        _ = attribs;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        _ = reader;
        _ = attribs;
        Err(ParseError::BadChild(Self::NAME.to_string(), element))
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        Err(ParseError::BadCont(text))
    }

    fn parse_misc(&mut self, event: XmlEvent) -> Result<(), ParseError> {
        _ = event;
        Ok(())
    }
}

impl<T> Parse for T
where
    T: GenericParse + Default,
{
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        if element != Self::NAME {
            Err(ParseError::BadStart(element))
        } else {
            let mut item = Self::default();
            item.parse_attribs(&element, attribs)?;
            if let Some((attrib, value)) = attribs.iter().next() {
                return Err(ParseError::UnreadAttrib(
                    element,
                    attrib.clone(),
                    value.clone(),
                ));
            }

            loop {
                match reader.next()? {
                    XmlEvent::EndDocument => return Err(ParseError::DocEnd),
                    XmlEvent::StartElement {
                        name, attributes, ..
                    } => {
                        item.parse_child(reader, name.local_name, &mut attributes.into_hash_map())?
                    }
                    XmlEvent::EndElement { name } => {
                        if name.local_name == Self::NAME {
                            break;
                        } else {
                            return Err(ParseError::BadEnd(name.local_name));
                        }
                    }
                    XmlEvent::Characters(text) => item.parse_text(text)?,
                    event => item.parse_misc(event)?,
                }
            }

            Ok(item)
        }
    }
}

pub struct GenericItem {
    pub text: String,
    pub kind: GenericItemKind,
}

impl GenericItem {
    pub fn text(text: String) -> GenericItem {
        Self {
            text,
            kind: GenericItemKind::Text,
        }
    }
}

impl Parse for GenericItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        _: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        let kind = match element.as_str() {
            "name" => GenericItemKind::Name,
            "type" => GenericItemKind::Type,
            "enum" => GenericItemKind::Enum,
            "comment" => GenericItemKind::Comment,
            _ => return Err(ParseError::BadStart(element)),
        };

        let mut content = String::new();
        loop {
            match reader.next()? {
                XmlEvent::EndDocument => return Err(ParseError::DocEnd),
                XmlEvent::StartElement { name, .. } => {
                    return Err(ParseError::BadChild(element, name.local_name))
                }
                XmlEvent::EndElement { name } => {
                    if name.local_name == element {
                        break;
                    }
                }
                XmlEvent::Characters(text) => content += &text,
                _ => {}
            }
        }

        Ok(Self {
            text: content,
            kind,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GenericItemKind {
    Text,
    Type,
    Name,
    Enum,
    Comment,
}

impl Vulkan {
    pub fn from_xml(mut reader: xml::EventReader<impl Read>) -> Result<Self, Error> {
        Ok(Self::create_registry(&mut reader)
            .map_err(|error| Error::parse(&reader, error))?
            .try_into()?)
    }

    fn create_registry(reader: &mut xml::EventReader<impl Read>) -> Result<Registry, ParseError> {
        let mut registry = None;
        loop {
            match reader.next()? {
                XmlEvent::EndDocument => break,
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    registry = Some(Registry::parse(
                        reader,
                        name.local_name,
                        &mut attributes.into_hash_map(),
                    )?)
                }
                XmlEvent::EndElement { name } => return Err(ParseError::BadEnd(name.local_name)),
                XmlEvent::Characters(content) => return Err(ParseError::BadCont(content)),
                _ => {}
            }
        }

        registry.ok_or(ParseError::EmptyReg)
    }
}

#[derive(Default)]
pub struct Registry {
    pub items: Vec<RegistryItem>,
    pub comment: Option<String>,
}

impl GenericParse for Registry {
    const NAME: &'static str = "registry";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(RegistryItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum RegistryItem {
    Comment(Comment),
    Platforms(Platforms),
    Tags(Tags),
    Types(Types),
    Enums(Enums),
    Commands(Commands),
    Feature(Feature),
    Extensions(Extensions),
    Formats(Formats),
    SpirvExtensions(SpirvExtensions),
    SpirvCapabilities(SpirvCapabilities),
    Sync(Sync),
    VideoCodecs(VideoCodecs),
}

impl Parse for RegistryItem {
    fn parse(
        reader: &mut xml::EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            Platforms::NAME => Self::Platforms(Platforms::parse(reader, element, attribs)?),
            Tags::NAME => Self::Tags(Tags::parse(reader, element, attribs)?),
            Types::NAME => Self::Types(Types::parse(reader, element, attribs)?),
            Enums::NAME => Self::Enums(Enums::parse(reader, element, attribs)?),
            Commands::NAME => Self::Commands(Commands::parse(reader, element, attribs)?),
            Feature::NAME => Self::Feature(Feature::parse(reader, element, attribs)?),
            Extensions::NAME => Self::Extensions(Extensions::parse(reader, element, attribs)?),
            Formats::NAME => Self::Formats(Formats::parse(reader, element, attribs)?),
            SpirvExtensions::NAME => {
                Self::SpirvExtensions(SpirvExtensions::parse(reader, element, attribs)?)
            }
            SpirvCapabilities::NAME => {
                Self::SpirvCapabilities(SpirvCapabilities::parse(reader, element, attribs)?)
            }
            Sync::NAME => Self::Sync(Sync::parse(reader, element, attribs)?),
            VideoCodecs::NAME => Self::VideoCodecs(VideoCodecs::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Registry::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct Comment {
    pub text: String,
}

impl GenericParse for Comment {
    const NAME: &'static str = "comment";

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.text += &text;
        Ok(())
    }
}

#[derive(Default)]
pub struct Platforms {
    pub items: Vec<Platform>,

    pub comment: Option<String>,
}

impl GenericParse for Platforms {
    const NAME: &'static str = "platforms";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(Platform::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct Platform {
    pub name: String,
    pub protect: String,

    pub comment: Option<String>,
}

impl GenericParse for Platform {
    const NAME: &'static str = "platform";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.protect = attribs.req_attrib(element, "protect")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct Tags {
    pub items: Vec<Tag>,

    pub comment: Option<String>,
}

impl GenericParse for Tags {
    const NAME: &'static str = "tags";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(Tag::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct Tag {
    pub name: String,
    pub author: String,
    pub contact: String,

    pub comment: Option<String>,
}

impl GenericParse for Tag {
    const NAME: &'static str = "tag";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.author = attribs.req_attrib(element, "author")?;
        self.contact = attribs.req_attrib(element, "contact")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct Types {
    pub items: Vec<TypesItem>,

    pub comment: Option<String>,
}

impl GenericParse for Types {
    const NAME: &'static str = "types";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");

        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(TypesItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum TypesItem {
    Comment(Comment),
    Type(Type),
}

impl Parse for TypesItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => TypesItem::Comment(Comment::parse(reader, element, attribs)?),
            Type::NAME => TypesItem::Type(Type::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Types::NAME.to_string(), element)),
        })
    }
}

pub struct Type {
    pub common: TypeCommon,
    pub details: TypeDetails,
}

impl Type {
    pub const NAME: &'static str = "type";
}

impl Parse for Type {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match attribs.remove("category") {
            Some(category) => match category.as_str() {
                IncludeType::CATEGORY => IncludeType::parse(reader, element, attribs)?.into(),
                DefineType::CATEGORY => DefineType::parse(reader, element, attribs)?.into(),
                BaseType::CATEGORY => BaseType::parse(reader, element, attribs)?.into(),
                HandleType::CATEGORY => HandleType::parse(reader, element, attribs)?.into(),
                BitmaskType::CATEGORY => BitmaskType::parse(reader, element, attribs)?.into(),
                EnumType::CATEGORY => EnumType::parse(reader, element, attribs)?.into(),
                FnPtrType::CATEGORY => FnPtrType::parse(reader, element, attribs)?.into(),
                StructType::CATEGORY => StructType::parse(reader, element, attribs)?.into(),
                UnionType::CATEGORY => UnionType::parse(reader, element, attribs)?.into(),
                _ => {
                    return Err(ParseError::BadAttrib(
                        element,
                        "category".to_string(),
                        category,
                    ))
                }
            },
            None => ImportedType::parse(reader, element, attribs)?.into(),
        })
    }
}

#[derive(Default)]
pub struct TypeCommon {
    pub name: String,
    pub requires: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub api: Option<Vec<String>>,
    pub alias: Option<String>,

    pub comment: Option<String>,
}

impl TypeCommon {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        if let Some(name) = attribs.remove("name") {
            self.name = name;
        }

        self.requires = attribs.remove("requires");
        self.deprecated = attribs.try_get(element, "deprecated")?;
        self.api = attribs
            .remove("api")
            .map(|value| value.split(',').map(|s| s.to_string()).collect());
        self.alias = attribs.remove("alias");

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

pub enum TypeDetails {
    Include(IncludeDetails),
    Define(DefineDetails),
    Base(BaseTypeDetails),
    Handle(HandleDetails),
    Bitmask(BitmaskDetails),
    Enum(EnumTypeDetails),
    FnPtr(FnPtrDetails),
    Struct(StructDetails),
    Union(UnionDetails),
    Imported(ImportedDetails),
}

#[derive(Default)]
pub struct IncludeType {
    pub common: TypeCommon,
    pub details: IncludeDetails,
}

impl IncludeType {
    pub const CATEGORY: &'static str = "include";
}

impl GenericParse for IncludeType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));

        Ok(())
    }
}

impl From<IncludeType> for Type {
    fn from(value: IncludeType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Include(value.details),
        }
    }
}

#[derive(Default)]
pub struct IncludeDetails {
    pub items: Vec<GenericItem>,
}

impl IncludeDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DefineType {
    pub common: TypeCommon,
    pub details: DefineDetails,
}

impl DefineType {
    pub const CATEGORY: &'static str = "define";
}

impl GenericParse for DefineType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.common.name = item.text.clone();
        }

        self.details.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));
        Ok(())
    }
}

impl From<DefineType> for Type {
    fn from(value: DefineType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Define(value.details),
        }
    }
}

#[derive(Default)]
pub struct DefineDetails {
    pub items: Vec<GenericItem>,
}

impl DefineDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct BaseType {
    pub common: TypeCommon,
    pub details: BaseTypeDetails,
}

impl BaseType {
    pub const CATEGORY: &'static str = "basetype";
}

impl GenericParse for BaseType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.common.name = item.text.clone();
        }

        self.details.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));
        Ok(())
    }
}

impl From<BaseType> for Type {
    fn from(value: BaseType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Base(value.details),
        }
    }
}

#[derive(Default)]
pub struct BaseTypeDetails {
    pub items: Vec<GenericItem>,
}

impl BaseTypeDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct HandleType {
    pub common: TypeCommon,
    pub details: HandleDetails,
}

impl HandleType {
    pub const CATEGORY: &'static str = "handle";
}

impl GenericParse for HandleType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.common.name = item.text.clone();
        }

        self.details.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));
        Ok(())
    }
}

impl From<HandleType> for Type {
    fn from(value: HandleType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Handle(value.details),
        }
    }
}

#[derive(Default)]
pub struct HandleDetails {
    pub parent: Option<String>,
    pub obj_type_enum: Option<String>,

    pub items: Vec<GenericItem>,
}

impl HandleDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.parent = attribs.remove("parent");
        self.obj_type_enum = attribs.remove("objtypeenum");
        Ok(())
    }
}

#[derive(Default)]
pub struct BitmaskType {
    pub common: TypeCommon,
    pub details: BitmaskDetails,
}

impl BitmaskType {
    pub const CATEGORY: &'static str = "bitmask";
}

impl GenericParse for BitmaskType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;

        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.common.name = item.text.clone();
        }

        self.details.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));
        Ok(())
    }
}

impl From<BitmaskType> for Type {
    fn from(value: BitmaskType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Bitmask(value.details),
        }
    }
}

#[derive(Default)]
pub struct BitmaskDetails {
    pub bit_values: Option<String>,

    pub items: Vec<GenericItem>,
}

impl BitmaskDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.bit_values = attribs.remove("bitvalues");
        Ok(())
    }
}

#[derive(Default)]
pub struct EnumType {
    pub common: TypeCommon,
    pub details: EnumTypeDetails,
}

impl EnumType {
    pub const CATEGORY: &'static str = "enum";
}

impl GenericParse for EnumType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl From<EnumType> for Type {
    fn from(value: EnumType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Enum(value.details),
        }
    }
}

#[derive(Default)]
pub struct EnumTypeDetails {}

impl EnumTypeDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct FnPtrType {
    pub common: TypeCommon,
    pub details: FnPtrDetails,
}

impl FnPtrType {
    pub const CATEGORY: &'static str = "funcpointer";
}

impl GenericParse for FnPtrType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.common.name = item.text.clone();
        }

        self.details.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.details.items.push(GenericItem::text(text));
        Ok(())
    }
}

impl From<FnPtrType> for Type {
    fn from(value: FnPtrType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::FnPtr(value.details),
        }
    }
}

#[derive(Default)]
pub struct FnPtrDetails {
    pub items: Vec<GenericItem>,
}

impl FnPtrDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct StructType {
    pub common: TypeCommon,
    pub details: StructDetails,
}

impl StructType {
    pub const CATEGORY: &'static str = "struct";
}

impl GenericParse for StructType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;

        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.details
            .members
            .push(StructItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

impl From<StructType> for Type {
    fn from(value: StructType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Struct(value.details),
        }
    }
}

#[derive(Default)]
pub struct StructDetails {
    pub returned_only: Option<bool>,
    pub allow_duplicate: Option<bool>,
    pub extends: Option<Vec<String>>,

    pub members: Vec<StructItem>,
}

impl StructDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.allow_duplicate = attribs
            .remove("allowduplicate")
            .map(|s| s.parse())
            .transpose()?;
        self.returned_only = attribs
            .remove("returnedonly")
            .map(|s| s.parse())
            .transpose()?;
        self.extends = attribs
            .remove("structextends")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        Ok(())
    }
}

#[derive(Default)]
pub struct UnionType {
    pub common: TypeCommon,
    pub details: UnionDetails,
}

impl UnionType {
    pub const CATEGORY: &'static str = "union";
}

impl GenericParse for UnionType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;

        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.details
            .members
            .push(StructItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

impl From<UnionType> for Type {
    fn from(value: UnionType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Union(value.details),
        }
    }
}

#[derive(Default)]
pub struct UnionDetails {
    pub returned_only: Option<bool>,

    pub members: Vec<StructItem>,
}

impl UnionDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.returned_only = attribs
            .remove("returnedonly")
            .map(|s| s.parse())
            .transpose()?;
        Ok(())
    }
}

pub enum StructItem {
    Comment(Comment),
    Member(Member),
}

impl Parse for StructItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => StructItem::Comment(Comment::parse(reader, element, attribs)?),
            Member::NAME => StructItem::Member(Member::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(StructType::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct Member {
    pub name: String,

    pub api: Option<String>,
    pub values: Option<Vec<String>>,
    pub optional: Option<Vec<bool>>,
    pub deprecated: Option<Deprecation>,
    pub no_auto_validity: Option<bool>,
    pub extern_sync: Option<bool>,

    pub limit_type: Option<Vec<LimitType>>,
    pub len: Option<Vec<Len>>,
    pub object_type: Option<String>,

    pub selection: Option<String>,
    pub selector: Option<String>,

    pub items: Vec<GenericItem>,

    pub comment: Option<String>,
}

impl GenericParse for Member {
    const NAME: &'static str = "member";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.api = attribs.remove("api");
        self.values = attribs
            .remove("values")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.optional = attribs
            .remove("optional")
            .map(|s| s.split(',').map(|value| value.parse()).collect())
            .transpose()?;
        self.deprecated = attribs.try_get(element, "deprecated")?;
        self.no_auto_validity = attribs
            .remove("noautovalidity")
            .map(|s| s.parse())
            .transpose()?;
        self.extern_sync = attribs.try_get(element, "externsync")?;

        self.limit_type = LimitType::from_attrib(attribs.remove("limittype")).map_err(|value| {
            ParseError::BadAttrib(Self::NAME.to_string(), "limittype".to_string(), value)
        })?;
        self.len = Len::from_attribs(attribs.remove("len"), attribs.remove("altlen"));
        self.object_type = attribs.remove("objecttype");

        self.selection = attribs.remove("selection");
        self.selector = attribs.remove("selector");

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        let item = GenericItem::parse(reader, element, attribs)?;
        if let GenericItemKind::Name = item.kind {
            self.name = item.text.clone();
        }

        self.items.push(item);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.items.push(GenericItem::text(text));
        Ok(())
    }
}

pub enum LimitType {
    NoAuto,
    Struct,
    Range,
    Exact,
    Pot,
    Min,
    Max,
    Mul,
    Bitmask,
    Bits,
    Not,
}

impl LimitType {
    pub fn from_attrib(limit_type: Option<String>) -> Result<Option<Vec<Self>>, String> {
        if let Some(limit_type) = limit_type {
            Some(
                limit_type
                    .as_str()
                    .split(',')
                    .map(|s| Self::from_str(s))
                    .collect::<Result<_, _>>()
                    .map_err(|_| limit_type),
            )
            .transpose()
        } else {
            Ok(None)
        }
    }
}

impl FromStr for LimitType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "noauto" => Self::NoAuto,
            "struct" => Self::Struct,
            "range" => Self::Range,
            "exact" => Self::Exact,
            "pot" => Self::Pot,
            "min" => Self::Min,
            "max" => Self::Max,
            "mul" => Self::Mul,
            "bitmask" => Self::Bitmask,
            "bits" => Self::Bits,
            "not" => Self::Not,
            _ => return Err(()),
        })
    }
}

pub enum Len {
    NullTerminated,
    Member(String),
    Number(String),
    Alt(String, String),
}

impl Len {
    pub fn from_attribs(len: Option<String>, alt_len: Option<String>) -> Option<Vec<Self>> {
        if let Some(len) = len {
            if let Some(alt_len) = alt_len {
                Some(vec![Self::Alt(len, alt_len)])
            } else {
                len.split(',')
                    .map(|s| Self::from_str(s))
                    .collect::<Result<_, _>>()
                    .ok()
            }
        } else {
            None
        }
    }
}

impl FromStr for Len {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "null-terminated" => Self::NullTerminated,
            value => {
                if value.starts_with(|c: char| c.is_alphabetic()) {
                    Self::Member(value.to_string())
                } else {
                    Self::Number(value.to_string())
                }
            }
        })
    }
}

#[derive(Default)]
pub struct ImportedType {
    pub common: TypeCommon,
    pub details: ImportedDetails,
}

impl GenericParse for ImportedType {
    const NAME: &'static str = Type::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl From<ImportedType> for Type {
    fn from(value: ImportedType) -> Self {
        Self {
            common: value.common,
            details: TypeDetails::Imported(value.details),
        }
    }
}

#[derive(Default)]
pub struct ImportedDetails {}

impl ImportedDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        _: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct Enums {
    pub name: String,
    pub ty: EnumsType,
    pub bit_width: Option<String>,

    pub contents: Vec<EnumsItem>,

    pub comment: Option<String>,
}

impl GenericParse for Enums {
    const NAME: &'static str = "enums";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.ty = attribs.try_get(element, "type")?.unwrap_or_default();
        self.bit_width = attribs.remove("bitwidth");

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.contents
            .push(EnumsItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub enum EnumsType {
    #[default]
    Constants,
    Enum,
    Bitmask,
}

impl FromStr for EnumsType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "constants" => Self::Constants,
            "enum" => Self::Enum,
            "bitmask" => Self::Bitmask,
            _ => return Err(()),
        })
    }
}

pub enum EnumsItem {
    Comment(Comment),
    Enum(Enum),
    Unused(Unused),
}

impl Parse for EnumsItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            Enum::NAME => Self::Enum(Enum::parse(reader, element, attribs)?),
            Unused::NAME => Self::Unused(Unused::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Enums::NAME.to_string(), element)),
        })
    }
}

pub struct Enum {
    pub common: EnumCommon,
    pub details: EnumDetails,
}

impl Enum {
    pub const NAME: &'static str = "enum";
}

impl Parse for Enum {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(if attribs.contains_key("alias") {
            AliasEnum::parse(reader, element, attribs)?.into()
        } else if attribs.contains_key("bitpos") {
            BitEnum::parse(reader, element, attribs)?.into()
        } else {
            if attribs.contains_key("type") {
                ConstantEnum::parse(reader, element, attribs)?.into()
            } else {
                ValueEnum::parse(reader, element, attribs)?.into()
            }
        })
    }
}

#[derive(Default)]
pub struct EnumCommon {
    pub name: String,
    pub api: Option<String>,
    pub deprecated: Option<Deprecation>,

    pub comment: Option<String>,
}

impl EnumCommon {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.api = attribs.remove("api");
        self.deprecated = attribs.try_get(element, "deprecated")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

pub enum EnumDetails {
    Constant(ConstantDetails),
    Value(ValueDetails),
    Bit(BitDetails),
    Alias(AliasEnumDetails),
}

#[derive(Default)]
pub struct ConstantEnum {
    pub common: EnumCommon,
    pub details: ConstantDetails,
}

impl GenericParse for ConstantEnum {
    const NAME: &'static str = Enum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<Enum> for ConstantEnum {
    fn into(self) -> Enum {
        Enum {
            common: self.common,
            details: EnumDetails::Constant(self.details),
        }
    }
}

#[derive(Default)]
pub struct ConstantDetails {
    pub value: String,
    pub ty: String,
}

impl ConstantDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.value = attribs.req_attrib(element, "value")?;
        self.ty = attribs.req_attrib(element, "type")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct ValueEnum {
    pub common: EnumCommon,
    pub details: ValueDetails,
}

impl GenericParse for ValueEnum {
    const NAME: &'static str = Enum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<Enum> for ValueEnum {
    fn into(self) -> Enum {
        Enum {
            common: self.common,
            details: EnumDetails::Value(self.details),
        }
    }
}

#[derive(Default)]
pub struct ValueDetails {
    pub value: String,
}

impl ValueDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.value = attribs.req_attrib(element, "value")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct BitEnum {
    pub common: EnumCommon,
    pub details: BitDetails,
}

impl GenericParse for BitEnum {
    const NAME: &'static str = Enum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<Enum> for BitEnum {
    fn into(self) -> Enum {
        Enum {
            common: self.common,
            details: EnumDetails::Bit(self.details),
        }
    }
}

#[derive(Default)]
pub struct BitDetails {
    pub bit_pos: String,
}

impl BitDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.bit_pos = attribs.req_attrib(element, "bitpos")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct AliasEnum {
    pub common: EnumCommon,
    pub details: AliasEnumDetails,
}

impl GenericParse for AliasEnum {
    const NAME: &'static str = Enum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<Enum> for AliasEnum {
    fn into(self) -> Enum {
        Enum {
            common: self.common,
            details: EnumDetails::Alias(self.details),
        }
    }
}

#[derive(Default)]
pub struct AliasEnumDetails {
    pub name: String,
    pub alias: String,

    pub comment: Option<String>,
}

impl AliasEnumDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.alias = attribs.req_attrib(element, "alias")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct Unused {
    pub start: String,

    pub comment: Option<String>,
}

impl GenericParse for Unused {
    const NAME: &'static str = "unused";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.start = attribs.req_attrib(element, "start")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct Commands {
    pub items: Vec<CommandsItem>,

    pub comment: Option<String>,
}

impl GenericParse for Commands {
    const NAME: &'static str = "commands";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(CommandsItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum CommandsItem {
    Comment(Comment),
    Command(Command),
}

impl Parse for CommandsItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_ref() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            Command::NAME => Self::Command(Command::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Commands::NAME.to_string(), element)),
        })
    }
}

pub enum Command {
    Defined(DefinedCommand),
    Alias(AliasCommand),
}

impl Command {
    pub const NAME: &'static str = "command";
}

impl Parse for Command {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(if attribs.contains_key("alias") {
            Self::Alias(AliasCommand::parse(reader, element, attribs)?)
        } else {
            Self::Defined(DefinedCommand::parse(reader, element, attribs)?)
        })
    }
}

#[derive(Default)]
pub struct DefinedCommand {
    pub api: Option<String>,

    pub success_codes: Option<Vec<String>>,
    pub error_codes: Option<Vec<String>>,
    pub queues: Option<Vec<String>>,
    pub render_pass: Option<String>,
    pub video_coding: Option<String>,
    pub cmd_buffer_level: Option<Vec<String>>,
    pub tasks: Option<Vec<String>>,

    pub proto: CommandProto,
    pub items: Vec<CommandItem>,

    pub comment: Option<String>,
}

impl GenericParse for DefinedCommand {
    const NAME: &'static str = Command::NAME;

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.api = attribs.remove("api");
        self.success_codes = attribs
            .remove("successcodes")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.error_codes = attribs
            .remove("errorcodes")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.queues = attribs
            .remove("queues")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.render_pass = attribs.remove("renderpass");
        self.video_coding = attribs.remove("videocoding");
        self.queues = attribs
            .remove("cmdbufferlevel")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.tasks = attribs
            .remove("tasks")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        if element == CommandProto::NAME {
            self.proto = CommandProto::parse(reader, element, attribs)?;
        } else {
            self.items
                .push(CommandItem::parse(reader, element, attribs)?);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct CommandProto {
    pub items: Vec<GenericItem>,

    pub comment: Option<String>,
}

impl GenericParse for CommandProto {
    const NAME: &'static str = "proto";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(GenericItem::parse(reader, element, attribs)?);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.items.push(GenericItem::text(text));
        Ok(())
    }
}

pub enum CommandItem {
    Comment(Comment),
    Param(CommandParam),
    ImplicitExternSyncParams(ImplicitExternSyncParams),
}

impl Parse for CommandItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            CommandParam::NAME => Self::Param(CommandParam::parse(reader, element, attribs)?),
            ImplicitExternSyncParams::NAME => Self::ImplicitExternSyncParams(
                ImplicitExternSyncParams::parse(reader, element, attribs)?,
            ),
            _ => return Err(ParseError::BadChild(Command::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct CommandParam {
    pub api: Option<String>,
    pub optional: Option<Vec<bool>>,
    pub extern_sync: Option<ExternSync>,
    pub no_auto_validity: Option<bool>,

    pub len: Option<Vec<Len>>,
    pub stride: Option<String>,
    pub object_type: Option<String>,
    pub valid_structs: Option<Vec<String>>,

    pub items: Vec<GenericItem>,

    pub comment: Option<String>,
}

impl GenericParse for CommandParam {
    const NAME: &'static str = "param";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.api = attribs.remove("api");
        self.optional = attribs
            .remove("optional")
            .map(|s| s.split(',').map(|value| value.parse()).collect())
            .transpose()?;
        self.extern_sync = attribs.try_get(element, "externsync")?;
        self.no_auto_validity = attribs.try_get(element, "noautovalidity")?;

        self.len = Len::from_attribs(attribs.remove("len"), attribs.remove("altlen"));
        self.stride = attribs.remove("stride");
        self.object_type = attribs.remove("objecttype");
        self.valid_structs = attribs
            .remove("validstructs")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(GenericItem::parse(reader, element, attribs)?);
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.items.push(GenericItem::text(text));
        Ok(())
    }
}

#[derive(Debug, Default)]
pub enum ExternSync {
    #[default]
    False,
    True,
    Field(String),
}

impl FromStr for ExternSync {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "false" => Self::False,
            "true" => Self::True,
            value => Self::Field(value.to_string()),
        })
    }
}

#[derive(Default)]
pub struct ImplicitExternSyncParams {
    pub items: Vec<ImplicitExternSyncParam>,

    pub comment: Option<String>,
}

impl GenericParse for ImplicitExternSyncParams {
    const NAME: &'static str = "implicitexternsyncparams";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(ImplicitExternSyncParam::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct ImplicitExternSyncParam {
    pub text: String,

    pub comment: Option<String>,
}

impl GenericParse for ImplicitExternSyncParam {
    const NAME: &'static str = "param";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.text += &text;
        Ok(())
    }
}

#[derive(Default)]
pub struct AliasCommand {
    pub name: String,
    pub alias: String,

    pub comment: Option<String>,
}

impl GenericParse for AliasCommand {
    const NAME: &'static str = Command::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.alias = attribs.req_attrib(element, "alias")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct Feature {
    pub name: String,
    pub number: String,
    pub api: Vec<String>,
    pub depends: Option<Depends>,

    pub items: Vec<FeatureItem>,

    pub comment: Option<String>,
}

impl GenericParse for Feature {
    const NAME: &'static str = "feature";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.number = attribs.req_attrib(element, "number")?;
        self.api = attribs
            .req_attrib(element, "api")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        self.depends = attribs.try_get(element, "depends")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(FeatureItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum FeatureItem {
    Comment(Comment),
    Require(Require),
    Remove(Remove),
}

impl Parse for FeatureItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            Require::NAME => Self::Require(Require::parse(reader, element, attribs)?),
            Remove::NAME => Self::Remove(Remove::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Feature::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct Extensions {
    pub items: Vec<Extension>,

    pub comment: Option<String>,
}

impl GenericParse for Extensions {
    const NAME: &'static str = "extensions";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(Extension::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct Extension {
    pub name: String,
    pub number: String,
    pub supported_api: Vec<String>,
    pub extension_type: Option<String>,
    pub depends: Option<Depends>,
    pub platform: Option<String>,

    pub deprecated_by: Option<String>,
    pub obsoleted_by: Option<String>,
    pub promoted_to: Option<String>,

    pub author: Option<String>,
    pub contact: Option<String>,
    pub ratified: Option<String>,
    pub provisional: Option<bool>,
    pub special_use: Option<String>,
    pub sort_order: Option<String>,

    pub items: Vec<ExtensionItem>,

    pub comment: Option<String>,
}

impl GenericParse for Extension {
    const NAME: &'static str = "extension";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.number = attribs.req_attrib(element, "number")?;
        self.supported_api = attribs
            .req_attrib(element, "supported")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        self.extension_type = attribs.remove("type");
        self.depends = attribs.try_get(element, "depends")?;
        self.platform = attribs.remove("platform");

        self.deprecated_by = attribs.remove("deprecatedby");
        self.obsoleted_by = attribs.remove("obsoletedby");
        self.promoted_to = attribs.remove("promotedto");

        self.author = attribs.remove("author");
        self.contact = attribs.remove("contact");
        self.ratified = attribs.remove("ratified");
        self.provisional = attribs.try_get(element, "provisional")?;
        self.special_use = attribs.remove("specialuse");
        self.sort_order = attribs.remove("sortorder");

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(ExtensionItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum Depends {
    Feature(String),
    And(Vec<Depends>),
    Or(Vec<Depends>),
}

impl Depends {
    fn parse(chars: &mut Peekable<Chars>) -> Option<Self> {
        Self::parse_or(chars)
    }

    fn parse_or(chars: &mut Peekable<Chars>) -> Option<Self> {
        let first = Self::parse_paren(chars)?;
        Some(match chars.peek() {
            Some(',') => {
                _ = chars.next();
                let mut children = vec![first];
                match Self::parse_or(chars)? {
                    Self::Or(vec) => children.extend(vec),
                    next => children.push(next),
                }

                Self::Or(children)
            }
            None => first,
            _ => return None,
        })
    }

    fn parse_paren(chars: &mut Peekable<Chars>) -> Option<Self> {
        match chars.peek() {
            Some('(') => {
                _ = chars.next();
                Self::parse(chars)
            }
            Some(_) => Self::parse_and(chars),
            None => None,
        }
    }

    fn parse_and(chars: &mut Peekable<Chars>) -> Option<Self> {
        Some(if let Some(pos) = chars.clone().position(|ch| ch == '+') {
            let mut children = vec![Self::Feature(chars.take(pos).collect())];
            let _ = chars.next();
            match Self::parse(chars) {
                Some(next) => {
                    match next {
                        Depends::And(vec) => children.extend(vec),
                        next => children.push(next),
                    }

                    Depends::And(children)
                }
                None => return None,
            }
        } else {
            Self::Feature(chars.collect())
        })
    }
}

impl FromStr for Depends {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Self::parse(&mut s.chars().peekable()).ok_or(())
    }
}

pub enum ExtensionItem {
    Comment(Comment),
    Require(Require),
    Remove(Remove),
}

impl Parse for ExtensionItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            Require::NAME => Self::Require(Require::parse(reader, element, attribs)?),
            Remove::NAME => Self::Remove(Remove::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Extension::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct Require {
    pub depends: Option<Depends>,
    pub api: Option<String>,

    pub items: Vec<RequireItem>,

    pub comment: Option<String>,
}

impl GenericParse for Require {
    const NAME: &'static str = "require";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.depends = attribs.try_get(element, "depends")?;
        self.api = attribs.remove("api");

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(RequireItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct Remove {
    pub reason_link: Option<String>,

    pub items: Vec<RequireItem>,

    pub comment: Option<String>,
}

impl GenericParse for Remove {
    const NAME: &'static str = "remove";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.reason_link = attribs.try_get(element, "reasonlink")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(RequireItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum RequireItem {
    Comment(Comment),
    Type(RequireType),
    Enum(RequireEnum),
    Command(RequireCommand),
    Feature(RequireFeature),
}

impl Parse for RequireItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            RequireType::NAME => Self::Type(RequireType::parse(reader, element, attribs)?),
            RequireEnum::NAME => Self::Enum(RequireEnum::parse(reader, element, attribs)?),
            RequireCommand::NAME => Self::Command(RequireCommand::parse(reader, element, attribs)?),
            RequireFeature::NAME => Self::Feature(RequireFeature::parse(reader, element, attribs)?),
            _ => Self::Comment(Comment::parse(reader, element, attribs)?),
        })
    }
}

#[derive(Default)]
pub struct RequireType {
    pub name: String,

    pub comment: Option<String>,
}

impl GenericParse for RequireType {
    const NAME: &'static str = "type";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

pub struct RequireEnum {
    pub common: RequireEnumCommon,
    pub details: RequireEnumDetails,
}

impl RequireEnum {
    pub const NAME: &'static str = "enum";
}

impl Parse for RequireEnum {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(if attribs.contains_key("alias") {
            RequireEnumAlias::parse(reader, element, attribs)?.into()
        } else if attribs.contains_key("offset") {
            RequireEnumOffset::parse(reader, element, attribs)?.into()
        } else if attribs.contains_key("bitpos") {
            RequireEnumBit::parse(reader, element, attribs)?.into()
        } else {
            RequireEnumValue::parse(reader, element, attribs)?.into()
        })
    }
}

#[derive(Default)]
pub struct RequireEnumCommon {
    pub name: String,
    pub extends: Option<String>,
    pub deprecated: Option<Deprecation>,
    pub api: Option<Vec<String>>,
    pub protect: Option<String>,

    pub comment: Option<String>,
}

impl RequireEnumCommon {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.extends = attribs.remove("extends");
        self.deprecated = attribs.try_get(element, "deprecated")?;
        self.api = attribs
            .remove("api")
            .map(|value| value.split(',').map(|s| s.to_string()).collect());
        self.protect = attribs.remove("protect");

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

pub enum RequireEnumDetails {
    Value(RequireEnumValueDetails),
    Offset(RequireEnumOffsetDetails),
    Bit(RequireEnumBitDetails),
    Alias(RequireEnumAliasDetails),
}

#[derive(Default)]
pub struct RequireEnumValue {
    pub common: RequireEnumCommon,
    pub details: RequireEnumValueDetails,
}

impl GenericParse for RequireEnumValue {
    const NAME: &'static str = RequireEnum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<RequireEnum> for RequireEnumValue {
    fn into(self) -> RequireEnum {
        RequireEnum {
            common: self.common,
            details: RequireEnumDetails::Value(self.details),
        }
    }
}

#[derive(Default)]
pub struct RequireEnumValueDetails {
    pub value: Option<String>,
}

impl RequireEnumValueDetails {
    pub fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.value = attribs.remove("value");
        Ok(())
    }
}

#[derive(Default)]
pub struct RequireEnumOffset {
    pub common: RequireEnumCommon,
    pub details: RequireEnumOffsetDetails,
}

impl GenericParse for RequireEnumOffset {
    const NAME: &'static str = RequireEnum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<RequireEnum> for RequireEnumOffset {
    fn into(self) -> RequireEnum {
        RequireEnum {
            common: self.common,
            details: RequireEnumDetails::Offset(self.details),
        }
    }
}

#[derive(Default)]
pub struct RequireEnumOffsetDetails {
    pub ext_number: Option<String>,
    pub offset: String,
    pub dir: Option<Dir>,
}

impl RequireEnumOffsetDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.ext_number = attribs.remove("extnumber");
        self.offset = attribs.req_attrib(element, "offset")?;
        self.dir = attribs.try_get(element, "dir")?;

        Ok(())
    }
}

pub enum Dir {
    Pos,
    Neg,
}

impl FromStr for Dir {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        Ok(match value {
            "+" => Self::Pos,
            "-" => Self::Neg,
            _ => return Err(()),
        })
    }
}

#[derive(Default)]
pub struct RequireEnumBit {
    pub common: RequireEnumCommon,
    pub details: RequireEnumBitDetails,
}

impl GenericParse for RequireEnumBit {
    const NAME: &'static str = RequireEnum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<RequireEnum> for RequireEnumBit {
    fn into(self) -> RequireEnum {
        RequireEnum {
            common: self.common,
            details: RequireEnumDetails::Bit(self.details),
        }
    }
}

#[derive(Default)]
pub struct RequireEnumBitDetails {
    pub ext_number: Option<String>,
    pub bit_pos: String,
}

impl RequireEnumBitDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.ext_number = attribs.remove("extnumber");
        self.bit_pos = attribs.req_attrib(element, "bitpos")?;

        Ok(())
    }
}

#[derive(Default)]
pub struct RequireEnumAlias {
    pub common: RequireEnumCommon,
    pub details: RequireEnumAliasDetails,
}

impl GenericParse for RequireEnumAlias {
    const NAME: &'static str = RequireEnum::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.common.parse_attribs(element, attribs)?;
        self.details.parse_attribs(element, attribs)?;
        Ok(())
    }
}

impl Into<RequireEnum> for RequireEnumAlias {
    fn into(self) -> RequireEnum {
        RequireEnum {
            common: self.common,
            details: RequireEnumDetails::Alias(self.details),
        }
    }
}

#[derive(Default)]
pub struct RequireEnumAliasDetails {
    pub alias: String,
}

impl RequireEnumAliasDetails {
    pub fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.alias = attribs.req_attrib(element, "alias")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct RequireCommand {
    pub name: String,

    pub comment: Option<String>,
}

impl GenericParse for RequireCommand {
    const NAME: &'static str = "command";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct RequireFeature {
    pub name: String,
    pub struct_name: String,

    pub comment: Option<String>,
}

impl GenericParse for RequireFeature {
    const NAME: &'static str = "feature";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.struct_name = attribs.req_attrib(element, "struct")?;

        self.comment = attribs.remove("comment");
        Ok(())
    }
}

#[derive(Default)]
pub struct Formats {
    pub items: Vec<Format>,

    pub comment: Option<String>,
}

impl GenericParse for Formats {
    const NAME: &'static str = "formats";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(Format::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct Format {
    pub name: String,
    pub class: String,
    pub block_size: String,
    pub texels_per_block: String,
    pub packed: Option<String>,
    pub block_extent: Option<Vec<String>>,
    pub compressed: Option<String>,
    pub chroma: Option<String>,

    pub items: Vec<FormatItem>,
}

impl GenericParse for Format {
    const NAME: &'static str = "format";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.class = attribs.req_attrib(element, "class")?;
        self.block_size = attribs.req_attrib(element, "blockSize")?;
        self.texels_per_block = attribs.req_attrib(element, "texelsPerBlock")?;
        self.packed = attribs.remove("packed");
        self.block_extent = attribs
            .remove("blockExtent")
            .map(|s| s.split(',').map(|value| value.to_string()).collect());
        self.compressed = attribs.remove("compressed");
        self.chroma = attribs.remove("chroma");

        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(FormatItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum FormatItem {
    Component(Component),
    Plane(Plane),
    SpirvImageFormat(SpirvImageFormat),
}

impl Parse for FormatItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Component::NAME => Self::Component(Component::parse(reader, element, attribs)?),
            Plane::NAME => Self::Plane(Plane::parse(reader, element, attribs)?),
            SpirvImageFormat::NAME => {
                Self::SpirvImageFormat(SpirvImageFormat::parse(reader, element, attribs)?)
            }
            _ => return Err(ParseError::BadChild(Format::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct Component {
    pub name: String,
    pub bits: String,
    pub numeric_format: String,
    pub plane_index: Option<String>,
}

impl GenericParse for Component {
    const NAME: &'static str = "component";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.bits = attribs.req_attrib(element, "bits")?;
        self.numeric_format = attribs.req_attrib(element, "numericFormat")?;
        self.plane_index = attribs.remove("planeIndex");

        Ok(())
    }
}

#[derive(Default)]
pub struct Plane {
    pub index: String,
    pub width_divisor: String,
    pub height_divisor: String,
    pub compatible: String,
}

impl GenericParse for Plane {
    const NAME: &'static str = "plane";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.index = attribs.req_attrib(element, "index")?;
        self.width_divisor = attribs.req_attrib(element, "widthDivisor")?;
        self.height_divisor = attribs.req_attrib(element, "heightDivisor")?;
        self.compatible = attribs.req_attrib(element, "compatible")?;

        Ok(())
    }
}

#[derive(Default)]
pub struct SpirvImageFormat {
    pub name: String,
}

impl GenericParse for SpirvImageFormat {
    const NAME: &'static str = "spirvimageformat";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct SpirvExtensions {
    pub items: Vec<SpirvExtension>,

    pub comment: Option<String>,
}

impl GenericParse for SpirvExtensions {
    const NAME: &'static str = "spirvextensions";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(SpirvExtension::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct SpirvExtension {
    pub name: String,

    pub items: Vec<SpirvItem>,
}

impl GenericParse for SpirvExtension {
    const NAME: &'static str = "spirvextension";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(SpirvItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct SpirvCapabilities {
    pub items: Vec<SpirvCapability>,

    pub comment: Option<String>,
}

impl GenericParse for SpirvCapabilities {
    const NAME: &'static str = "spirvcapabilities";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(SpirvCapability::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct SpirvCapability {
    pub name: String,

    pub items: Vec<SpirvItem>,
}

impl GenericParse for SpirvCapability {
    const NAME: &'static str = "spirvcapability";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(SpirvItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum SpirvItem {
    Enable(Enable),
}

impl Parse for SpirvItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            _ => Self::Enable(Enable::parse(reader, element, attribs)?),
        })
    }
}

pub enum Enable {
    Version(EnableVersion),
    Extension(EnableExtension),
    Feature(EnableFeature),
    Property(EnableProperty),
}

impl Enable {
    pub const NAME: &'static str = "enable";
}

impl Parse for Enable {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(if attribs.contains_key("version") {
            Self::Version(EnableVersion::parse(reader, element, attribs)?)
        } else if attribs.contains_key("extension") {
            Self::Extension(EnableExtension::parse(reader, element, attribs)?)
        } else if attribs.contains_key("feature") {
            Self::Feature(EnableFeature::parse(reader, element, attribs)?)
        } else if attribs.contains_key("property") {
            Self::Property(EnableProperty::parse(reader, element, attribs)?)
        } else {
            return Err(ParseError::BadStart(element));
        })
    }
}

#[derive(Default)]
pub struct EnableVersion {
    pub version: String,
}

impl GenericParse for EnableVersion {
    const NAME: &'static str = Enable::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.version = attribs.req_attrib(element, "version")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct EnableExtension {
    pub extension: String,
}

impl GenericParse for EnableExtension {
    const NAME: &'static str = Enable::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.extension = attribs.req_attrib(element, "extension")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct EnableFeature {
    pub struct_name: String,
    pub feature: String,
    pub requires: Vec<String>,
    pub alias: Option<String>,
}

impl GenericParse for EnableFeature {
    const NAME: &'static str = Enable::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.struct_name = attribs.req_attrib(element, "struct")?;
        self.feature = attribs.req_attrib(element, "feature")?;
        self.requires = attribs
            .req_attrib(element, "requires")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        self.alias = attribs.remove("alias");

        Ok(())
    }
}

#[derive(Default)]
pub struct EnableProperty {
    pub property: String,
    pub member: String,
    pub value: String,
    pub requires: Vec<String>,
}

impl GenericParse for EnableProperty {
    const NAME: &'static str = Enable::NAME;

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.property = attribs.req_attrib(element, "property")?;
        self.member = attribs.req_attrib(element, "member")?;
        self.value = attribs.req_attrib(element, "value")?;
        self.requires = attribs
            .req_attrib(element, "requires")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        Ok(())
    }
}

#[derive(Default)]
pub struct Sync {
    pub items: Vec<SyncItem>,

    pub comment: Option<String>,
}

impl GenericParse for Sync {
    const NAME: &'static str = "sync";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items.push(SyncItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum SyncItem {
    Stage(SyncStage),
    Access(SyncAccess),
    Pipeline(SyncPipeline),
}

impl Parse for SyncItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            SyncStage::NAME => Self::Stage(SyncStage::parse(reader, element, attribs)?),
            SyncAccess::NAME => Self::Access(SyncAccess::parse(reader, element, attribs)?),
            SyncPipeline::NAME => Self::Pipeline(SyncPipeline::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(Sync::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct SyncStage {
    pub name: String,
    pub alias: Option<String>,

    pub items: Vec<SyncStageItem>,
}

impl GenericParse for SyncStage {
    const NAME: &'static str = "syncstage";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.alias = attribs.remove("alias");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(SyncStageItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum SyncStageItem {
    Comment(Comment),
    Support(SyncStageSupport),
    Equivalent(SyncStageEquivalent),
}

impl Parse for SyncStageItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            SyncStageSupport::NAME => {
                Self::Support(SyncStageSupport::parse(reader, element, attribs)?)
            }
            SyncStageEquivalent::NAME => {
                Self::Equivalent(SyncStageEquivalent::parse(reader, element, attribs)?)
            }
            _ => return Err(ParseError::BadChild(SyncStage::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct SyncStageSupport {
    pub queues: Vec<String>,
}

impl GenericParse for SyncStageSupport {
    const NAME: &'static str = "syncsupport";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.queues = attribs
            .req_attrib(element, "queues")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncStageEquivalent {
    pub stage: Vec<String>,
}

impl GenericParse for SyncStageEquivalent {
    const NAME: &'static str = "syncequivalent";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.stage = attribs
            .req_attrib(element, "stage")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncAccess {
    pub name: String,
    pub alias: Option<String>,

    pub items: Vec<SyncAccessItem>,
}

impl GenericParse for SyncAccess {
    const NAME: &'static str = "syncaccess";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.alias = attribs.remove("alias");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(SyncAccessItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum SyncAccessItem {
    Comment(Comment),
    Support(SyncAccessSupport),
    Equivalent(SyncAccessEquivalent),
}

impl Parse for SyncAccessItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            Comment::NAME => Self::Comment(Comment::parse(reader, element, attribs)?),
            SyncAccessSupport::NAME => {
                Self::Support(SyncAccessSupport::parse(reader, element, attribs)?)
            }
            SyncAccessEquivalent::NAME => {
                Self::Equivalent(SyncAccessEquivalent::parse(reader, element, attribs)?)
            }
            _ => return Err(ParseError::BadChild(SyncStage::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct SyncAccessSupport {
    pub stage: Vec<String>,
}

impl GenericParse for SyncAccessSupport {
    const NAME: &'static str = "syncsupport";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.stage = attribs
            .req_attrib(element, "stage")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncAccessEquivalent {
    pub access: Vec<String>,
}

impl GenericParse for SyncAccessEquivalent {
    const NAME: &'static str = "syncequivalent";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.access = attribs
            .req_attrib(element, "access")?
            .split(',')
            .map(|value| value.to_string())
            .collect();
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncPipeline {
    pub name: String,
    pub depends: Option<Depends>,

    pub items: Vec<SyncPipelineStage>,
}

impl GenericParse for SyncPipeline {
    const NAME: &'static str = "syncpipeline";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.depends = attribs.try_get(element, "depends")?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(SyncPipelineStage::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct SyncPipelineStage {
    pub order: Option<String>,
    pub before: Option<String>,

    pub content: String,
}

impl GenericParse for SyncPipelineStage {
    const NAME: &'static str = "syncpipelinestage";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.order = attribs.remove("order");
        self.before = attribs.remove("before");
        Ok(())
    }

    fn parse_text(&mut self, text: String) -> Result<(), ParseError> {
        self.content += &text;
        Ok(())
    }
}

#[derive(Default)]
pub struct VideoCodecs {
    pub items: Vec<VideoCodec>,

    pub comment: Option<String>,
}

impl GenericParse for VideoCodecs {
    const NAME: &'static str = "videocodecs";

    fn parse_attribs(
        &mut self,
        _: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(VideoCodec::parse(reader, element, attribs)?);

        Ok(())
    }
}

#[derive(Default)]
pub struct VideoCodec {
    pub name: String,
    pub extend: Option<String>,
    pub value: Option<String>,

    pub items: Vec<VideoCodecItem>,

    pub comment: Option<String>,
}

impl GenericParse for VideoCodec {
    const NAME: &'static str = "videocodec";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.extend = attribs.remove("extend");
        self.value = attribs.remove("value");

        self.comment = attribs.remove("comment");
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(VideoCodecItem::parse(reader, element, attribs)?);
        Ok(())
    }
}

pub enum VideoCodecItem {
    Capabilities(VideoCapabilities),
    Format(VideoFormat),
    Profiles(VideoProfiles),
}

impl Parse for VideoCodecItem {
    fn parse(
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<Self, ParseError> {
        Ok(match element.as_str() {
            VideoCapabilities::NAME => {
                Self::Capabilities(VideoCapabilities::parse(reader, element, attribs)?)
            }
            VideoFormat::NAME => Self::Format(VideoFormat::parse(reader, element, attribs)?),
            VideoProfiles::NAME => Self::Profiles(VideoProfiles::parse(reader, element, attribs)?),
            _ => return Err(ParseError::BadChild(VideoCodec::NAME.to_string(), element)),
        })
    }
}

#[derive(Default)]
pub struct VideoCapabilities {
    pub struct_name: String,
}

impl GenericParse for VideoCapabilities {
    const NAME: &'static str = "videocapabilities";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.struct_name = attribs.req_attrib(element, "struct")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct VideoFormat {
    pub name: String,
    pub usage: String,
}

impl GenericParse for VideoFormat {
    const NAME: &'static str = "videoformat";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.usage = attribs.req_attrib(element, "usage")?;
        Ok(())
    }
}

#[derive(Default)]
pub struct VideoProfiles {
    pub struct_name: String,

    pub members: Vec<VideoProfileMember>,
}

impl GenericParse for VideoProfiles {
    const NAME: &'static str = "videoprofiles";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.struct_name = attribs.req_attrib(element, "struct")?;
        Ok(())
    }

    fn parse_child(
            &mut self,
            reader: &mut EventReader<impl Read>,
            element: String,
            attribs: &mut HashMap<String, String>,
        ) -> Result<(), ParseError> {
        self.members.push(VideoProfileMember::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct VideoProfileMember {
    pub name: String,

    pub items: Vec<VideoProfile>,
}

impl GenericParse for VideoProfileMember {
    const NAME: &'static str = "videoprofilemember";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        Ok(())
    }

    fn parse_child(
        &mut self,
        reader: &mut EventReader<impl Read>,
        element: String,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.items
            .push(VideoProfile::parse(reader, element, attribs)?);
        Ok(())
    }
}

#[derive(Default)]
pub struct VideoProfile {
    pub name: String,
    pub value: String,
}

impl GenericParse for VideoProfile {
    const NAME: &'static str = "videoprofile";

    fn parse_attribs(
        &mut self,
        element: &str,
        attribs: &mut HashMap<String, String>,
    ) -> Result<(), ParseError> {
        self.name = attribs.req_attrib(element, "name")?;
        self.value = attribs.req_attrib(element, "value")?;
        Ok(())
    }
}
