use std::{
    ffi::{c_char, c_void, CStr},
    ptr,
};

use crate::vk;

use super::ApiVersion;

// feature VK_API_VERSION_1_0

#[cfg(feature = "vk10")]
#[repr(C)]
pub struct ApplicationInfo {
    pub struct_type: vk::StructureType,
    pub next: *const c_void,
    pub application_name: *const c_char,
    pub application_version: u32,
    pub engine_name: *const c_char,
    pub engine_version: u32,
    pub api_version: u32,
}

#[cfg(feature = "vk10")]
impl Default for ApplicationInfo {
    fn default() -> Self {
        Self {
            struct_type: vk::StructureType::APPLICATION_INFO,
            next: ptr::null(),
            application_name: ptr::null(),
            application_version: 0,
            engine_name: ptr::null(),
            engine_version: 0,
            api_version: vk::API_VERSION_1_0.into(),
        }
    }
}

#[cfg(feature = "vk10")]
pub trait ApplicationInfoNext {}

#[cfg(feature = "vk10")]
impl ApplicationInfo {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn with_application_name(mut self, application_name: &CStr) -> Self {
        self.application_name = application_name.as_ptr();
        self
    }

    pub fn with_application_version(mut self, application_version: u32) -> Self {
        self.application_version = application_version;
        self
    }

    pub fn with_engine_name(mut self, engine_name: &CStr) -> Self {
        self.engine_name = engine_name.as_ptr();
        self
    }

    pub fn with_engine_version(mut self, engine_version: u32) -> Self {
        self.engine_version = engine_version;
        self
    }

    pub fn with_api_version(mut self, api_version: ApiVersion) -> Self {
        self.api_version = api_version.into();
        self
    }
}

#[cfg(feature = "vk10")]
#[repr(C)]
pub struct InstanceCreateInfo {
    pub struct_type: vk::StructureType,
    pub next: *const c_void,
    pub flags: vk::InstanceCreateFlags,
    pub application_info: *const vk::ApplicationInfo,
    pub enabled_layer_count: u32,
    pub enabled_layers: *const *const c_char,
    pub enabled_extension_count: u32,
    pub enabled_extensions: *const *const c_char,
}

#[cfg(feature = "vk10")]
impl Default for InstanceCreateInfo {
    fn default() -> Self {
        Self {
            struct_type: vk::StructureType::INSTANCE_CREATE_INFO,
            next: ptr::null(),
            flags: Default::default(),
            application_info: ptr::null(),
            enabled_layer_count: 0,
            enabled_layers: ptr::null(),
            enabled_extension_count: 0,
            enabled_extensions: ptr::null(),
        }
    }
}

impl InstanceCreateInfo {}
