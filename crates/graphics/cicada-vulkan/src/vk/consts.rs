#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct ApiVersion(pub u32);

#[cfg(feature = "vk10")]
pub const API_VERSION_1_0: ApiVersion = ApiVersion::new(1, 0, 0, 0);

impl ApiVersion {
    pub const fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self((variant << 29u32) | (major << 22u32) | (minor << 12u32) | (patch))
    }
}

impl Default for ApiVersion {
    fn default() -> Self {
        API_VERSION_1_0
    }
}

impl Into<u32> for ApiVersion {
    fn into(self) -> u32 {
        self.0
    }
}
