use std::{collections::HashMap, num::NonZeroUsize};

pub enum Deprecation {
    False,
    True,
    Aliased,
    Ignored,
}

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
    pub bitmask_types: Vec<TypeHandle>,
}

impl Types {
    pub fn get(&self, handle: TypeHandle) -> Option<&Type> {
        self.items.get(&handle)
    }

    pub fn find(&self, standard_name: &str) -> Option<TypeHandle> {
        self.items
            .iter()
            .find(|(_, ty)| ty.head.standard_name == standard_name)
            .map(|(&handle, _)| handle)
    }

    pub fn insert(&mut self, ty: Type) -> TypeHandle {
        let handle = self.next_handle();

        if ty.head.standard_name == "VkResult" {
            let _ = self.result_type.replace(handle);
        } else {
            match &ty.body {
                TypeBody::Include(_) => {}
                TypeBody::Define(_) => {}
                TypeBody::Imported(_) => {}
                TypeBody::Base(_) => {}
                TypeBody::Enum(_) => self.enum_types.push(handle),
                TypeBody::Bitmask(_) => self.bitmask_types.push(handle),
                TypeBody::Struct(_) => self.struct_types.push(handle),
                TypeBody::Union(_) => {}
                TypeBody::FnPtr(_) => {}
            }
        }

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
    pub decors: Box<[Decor]>,
}

pub enum Decor {
    Const,
    ConstPtr,
    MutPtr,
}

pub struct Type {
    pub head: TypeHead,
    pub body: TypeBody,
}

impl Type {
    pub fn new(head: TypeHead, body: TypeBody) -> Self {
        Self { head, body }
    }
}

#[derive(Default)]
pub struct TypeHead {
    pub standard_name: String,
    pub output_name: String,
    pub deprecated: Option<Deprecation>,
    pub requires: Option<TypeHandle>,
    pub feature: Option<FeatureHandle>,
}

impl TypeHead {
    pub fn new(
        standard_name: String,
        output_name: String,
        requires: Option<TypeHandle>,
        deprecated: Option<Deprecation>,
    ) -> Self {
        Self {
            standard_name,
            output_name,
            deprecated,
            requires,
            feature: None,
        }
    }
}

pub enum TypeBody {
    Include(IncludeBody),
    Define(DefineBody),
    Imported(ImportedBody),
    Base(BaseTypeBody),
    Enum(EnumBody),
    Bitmask(BitmaskBody),
    Struct(StructBody),
    Union(UnionBody),
    FnPtr(FnPtrBody),
}

#[derive(Default)]
pub struct IncludeBody {
    pub header_name: String,
    pub is_local: bool,
}

#[derive(Default)]
pub struct DefineBody {}

#[derive(Default)]
pub struct ImportedBody {}

#[derive(Default)]
pub struct BaseTypeBody {
    pub output_type: Option<TypeHandle>,
}

pub struct EnumBody {}

#[derive(Default)]
pub struct BitmaskBody {
    pub output_type: Option<TypeHandle>,
    pub bitmask_type: Option<TypeHandle>,
}

pub struct BitfieldBody {
    pub bitmask_type: TypeHandle,
}

#[derive(Default)]
pub struct StructBody {
    pub returned_only: Option<bool>,
    pub allow_duplicate: Option<bool>,
    pub extends_structs: Vec<TypeHandle>,
    pub members: Vec<StructMember>,
}

pub struct StructMember {
    pub standard_name: String,
    pub output_name: String,
    pub decor_type: DecorType,
}

#[derive(Default)]
pub struct UnionBody {}

#[derive(Default)]
pub struct FnPtrBody {}

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
