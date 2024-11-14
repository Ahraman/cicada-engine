use crate::{
    error::Error,
    repr::{StructType, Vulkan},
};

#[derive(Default)]
pub struct EmitSettings {}

impl Vulkan {
    pub fn emit(self, _settings: &EmitSettings) -> Result<(), Error> {
        Ok(())
    }
}

impl StructType {
    pub fn output_name(&self) -> String {
        if self.common.standard_name.starts_with("Vk") {
            // Remove the initial "Vk" segment
            let (_, output_name) = self.common.standard_name.split_at("Vk".len());
            output_name.to_string()
        } else {
            // Name already does not have "Vk" (for some reason), so we shouldn't change it
            self.common.standard_name.clone()
        }
    }
}
