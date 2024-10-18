use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use windows::core::PCWSTR;

pub(crate) struct WideStr {
    buf: Vec<u16>,
}

impl WideStr {
    pub(crate) fn new(value: impl AsRef<OsStr>) -> Self {
        Self {
            buf: value.as_ref().encode_wide().collect(),
        }
    }

    pub(crate) fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR::from_raw(self.buf.as_ptr())
    }
}
