use std::{num::NonZeroUsize, str::FromStr};

#[derive(Default)]
pub struct Vulkan {}

#[derive(Default)]
pub enum Deprecation {
    #[default]
    False,
    True,
    Aliased,
    Ignored,
}

impl FromStr for Deprecation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, ()> {
        Ok(match s {
            "false" => Self::False,
            "true" => Self::True,
            "aliased" => Self::Aliased,
            "ignored" => Self::Ignored,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeHandle(pub NonZeroUsize);

#[derive(Debug)]
pub struct Type {
    pub common: TypeCommon,
    pub details: TypeDetails,
}

#[derive(Debug, Default)]
pub struct TypeCommon {
    pub standard_name: String,
    pub standard_aliases: Vec<String>,
}

#[derive(Debug)]
pub enum TypeDetails {
    Include(IncludeDetails),
    Define(DefineDetails),
    Base(BaseTypeDetails),
    Handle(HandleDetails),
    Bitmask(BitmaskDetails),
    Enum(EnumDetails),
    FnPtr(FnPtrDetails),
    Struct(StructDetails),
    Union(UnionDetails),
    Imported(ImportedDetails),
}

#[derive(Debug, Default)]
pub struct IncludeDetails {}

#[derive(Debug, Default)]
pub struct IncludeType {
    pub common: TypeCommon,
    pub details: IncludeDetails,
}

impl From<IncludeType> for Type {
    fn from(value: IncludeType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Include(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct DefineDetails {}

#[derive(Debug, Default)]
pub struct DefineType {
    pub common: TypeCommon,
    pub details: DefineDetails,
}

impl From<DefineType> for Type {
    fn from(value: DefineType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Define(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct BaseTypeDetails {}

#[derive(Debug, Default)]
pub struct BaseType {
    pub common: TypeCommon,
    pub details: BaseTypeDetails,
}

impl From<BaseType> for Type {
    fn from(value: BaseType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Base(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct HandleDetails {}

#[derive(Debug, Default)]
pub struct HandleType {
    pub common: TypeCommon,
    pub details: HandleDetails,
}

impl From<HandleType> for Type {
    fn from(value: HandleType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Handle(value.details),
        }
    }
}

#[derive(Debug)]
pub struct BitmaskDetails {
    pub output_type: TypeHandle,
}

#[derive(Debug)]
pub struct BitmaskType {
    pub common: TypeCommon,
    pub details: BitmaskDetails,
}

impl From<BitmaskType> for Type {
    fn from(value: BitmaskType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Bitmask(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct EnumDetails {}

#[derive(Debug)]
pub struct EnumType {
    pub common: TypeCommon,
    pub details: EnumDetails,
}

impl From<EnumType> for Type {
    fn from(value: EnumType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Enum(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct FnPtrDetails {}

#[derive(Debug, Default)]
pub struct FnPtrType {
    pub common: TypeCommon,
    pub details: FnPtrDetails,
}

impl From<FnPtrType> for Type {
    fn from(value: FnPtrType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::FnPtr(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct StructDetails {}

#[derive(Debug, Default)]
pub struct StructType {
    pub common: TypeCommon,
    pub details: StructDetails,
}

impl From<StructType> for Type {
    fn from(value: StructType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Struct(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct UnionDetails {}

#[derive(Debug, Default)]
pub struct UnionType {
    pub common: TypeCommon,
    pub details: UnionDetails,
}

impl From<UnionType> for Type {
    fn from(value: UnionType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Union(value.details),
        }
    }
}

#[derive(Debug, Default)]
pub struct ImportedDetails {}

#[derive(Debug, Default)]
pub struct ImportedType {
    pub common: TypeCommon,
    pub details: ImportedDetails,
}

impl From<ImportedType> for Type {
    fn from(value: ImportedType) -> Self {
        Type {
            common: value.common,
            details: TypeDetails::Imported(value.details),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandHandle(pub NonZeroUsize);

pub struct Command {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureHandle(pub NonZeroUsize);

#[derive(Debug)]
pub struct Feature {}

pub enum FeatureDetails {
    Core(CoreFeature),
    Extension(Extension),
    Platform(Platform),
}

#[derive(Debug, Default)]
pub struct CoreFeature {}

#[derive(Debug, Default)]
pub struct Extension {}

#[derive(Debug, Default)]
pub struct Platform {}
