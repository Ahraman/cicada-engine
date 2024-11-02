use std::{collections::HashMap, num::NonZeroUsize};

#[derive(Default)]
pub struct Vulkan {
    pub types: Types,
    pub enums: Enums,
    pub commands: Commands,
    pub features: Features,
}

impl Vulkan {
    pub fn has_result(&self) -> bool {
        self.types.result_type.is_some()
    }

    pub fn has_enums(&self) -> bool {
        !self.types.enum_types.is_empty()
    }

    pub fn has_structs(&self) -> bool {
        !self.types.struct_types.is_empty()
    }

    pub fn has_unions(&self) -> bool {
        !self.types.union_types.is_empty()
    }

    pub fn has_commands(&self) -> bool {
        !self.commands.items.is_empty()
    }
}

#[derive(Default)]
pub struct Types {
    pub items: HashMap<TypeHandle, Type>,

    pub result_type: Option<TypeHandle>,
    pub struct_types: Vec<TypeHandle>,
    pub union_types: Vec<TypeHandle>,
    pub enum_types: Vec<TypeHandle>,
}

impl Types {
    pub fn get(&self, handle: TypeHandle) -> Option<&Type> {
        self.items.get(&handle)
    }

    pub fn insert(&mut self, ty: Type) -> TypeHandle {
        let handle = self.next_handle();
        self.items.insert(handle, ty);
        handle
    }

    pub fn next_handle(&self) -> TypeHandle {
        TypeHandle(NonZeroUsize::MIN.saturating_add(self.items.len()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeHandle(pub NonZeroUsize);

pub struct DecorType {
    pub handle: TypeHandle,
    pub decors: Vec<Decor>,
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

impl Type {}

pub struct TypeInfo {
    pub standard_name: String,
    pub output_name: String,
    pub feature: Option<FeatureHandle>,
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

#[derive(Default)]
pub struct Enums {
    pub items: Vec<Enum>,
}

#[derive(Default)]
pub struct Enum {
    pub values: Vec<EnumValue>,
}

pub struct EnumValue {}

#[derive(Default)]
pub struct Commands {
    pub items: HashMap<CommandHandle, Command>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandHandle(pub NonZeroUsize);

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
impl Features {
    pub fn get(&self, handle: FeatureHandle) -> Option<&Feature> {
        self.items.get(&handle)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FeatureHandle(pub NonZeroUsize);

pub struct Feature {
    pub header: FeatureHeader,
}

pub struct FeatureHeader {
    pub standard_name: String,
    pub output_name: String,
}

pub enum FeatureKind {
    Core(CoreFeature),
    Extension(Extension),
}

pub struct CoreFeature {}

pub struct Extension {}
