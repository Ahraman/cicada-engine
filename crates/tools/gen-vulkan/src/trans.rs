use crate::{
    error::TransError,
    parse::{self},
    repr,
};

impl TryFrom<parse::Registry> for repr::Vulkan {
    type Error = TransError;

    fn try_from(value: parse::Registry) -> Result<Self, TransError> {
        _ = value;
        todo!()
    }
}
