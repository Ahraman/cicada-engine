use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

use windows::core::PCWSTR;

pub(super) struct WideStr {
    buf: Vec<u16>,
}

impl WideStr {
    pub(super) fn from_os_str<O>(value: O) -> Self
    where
        O: AsRef<OsStr>,
    {
        Self {
            buf: value.as_ref().encode_wide().chain(once(0)).collect(),
        }
    }

    pub(super) fn as_pcswtr(&self) -> PCWSTR {
        PCWSTR::from_raw(self.buf.as_ptr())
    }
}
