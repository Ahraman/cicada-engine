use std::collections::HashMap;

#[derive(Default)]
pub struct Vulkan {
    pub metadata: Metadata,
    pub types: Types,
    pub features: Features,
}

#[derive(Default)]
pub struct Metadata {
    pub has_structs: bool,
    pub has_enum_types: bool,
    pub has_commands: bool,
}

#[derive(Default)]
pub struct Types {
    pub items: HashMap<TypeHandle, Type>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeHandle(pub usize);

pub struct DecorType {
    pub handle: TypeHandle,
    pub decor: Vec<Decor>,
}

pub enum Decor {
    Const,
    ConstPtr,
    MutPtr,
}

pub struct Type {
    pub info: TypeInfo,
    pub kind: TypeKind,
}

pub struct TypeInfo {
    pub standard_name: String,
    pub output_name: String,
    pub feature_set: FeatureHandle,
}

pub enum TypeKind {
    Builtin(BuiltinType),
    Enum(EnumType),
    Bitmask(BitmaskType),
    Bitfield(BitfieldType),
    Struct(StructType),
}

pub struct BuiltinType {}

pub struct EnumType {}

pub struct BitmaskType {
    pub bitfield_type: TypeHandle,
}

pub struct BitfieldType {
    pub bitmask_type: TypeHandle,
}

pub struct StructType {
    pub members: Vec<StructMember>,
}

pub struct StructMember {
    pub standard_name: String,
    pub output_name: String,
    pub decor_type: DecorType,
}

pub struct CommandHandle(pub usize);

pub struct Command {
    pub info: CommandInfo,
}

pub struct CommandInfo {
    pub standard_name: String,
    pub output_name: String,
}

#[derive(Default)]
pub struct Features {
    pub items: HashMap<FeatureHandle, Feature>,
}

pub struct FeatureHandle(pub usize);

pub struct Feature {
    pub header: FeatureHeader,
}

pub struct FeatureHeader {}

pub enum FeatureKind {
    Core(CoreFeature),
    Extension(Extension),
}

pub struct CoreFeature {}

pub struct Extension {}
