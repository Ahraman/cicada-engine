
#[cfg(feature = "vk10")]
pub struct StructureType(pub u32);

impl StructureType {
    #[cfg(feature = "vk10")]
    pub const APPLICATION_INFO: Self = Self(0u32);
    #[cfg(feature = "vk10")]
    pub const INSTANCE_CREATE_INFO: Self = Self(1u32);
}

#[cfg(feature = "vk10")]
pub struct InstanceCreateFlag(pub u32);

impl InstanceCreateFlag {
    #[cfg(feature = "VK_KHR_portability_enumeration")]
    pub const ENUMERATE_PORTABILITY_BIT_KHR: Self = Self(0x00000001u32);
}

#[cfg(feature = "vk10")]
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct InstanceCreateFlags(pub u32);

#[cfg(feature = "vk10")]
impl From<InstanceCreateFlag> for InstanceCreateFlags {
    fn from(value: InstanceCreateFlag) -> Self {
        Self(value.0)
    }
}

impl Into<u32> for InstanceCreateFlags {
    fn into(self) -> u32 {
        self.0
    }
}
