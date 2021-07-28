use crate::Result;
use std::ffi::c_void;

pub struct Hook {
    src: *mut c_void,
    len: usize,
    orig_bytes: Vec<u8>,
    active: bool,
}

pub struct TrampolineHook {
    gateway: *mut c_void,
    hook: Hook,
}

impl Hook {
    pub fn hook(src: *mut c_void, dst: *mut c_void, len: usize) -> Result<Self> {
        todo!()
    }

    pub fn unhook(&self) -> Result<()> {
        todo!()
    }

    pub fn active(&self) -> bool {
        todo!()
    }
}

impl TrampolineHook {
    pub fn hook(src: *mut c_void, dst: *mut c_void, len: usize) -> Result<Self> {
        todo!()
    }

    pub fn unhook(&self) -> Result<()> {
        todo!()
    }

    pub fn active(&self) -> bool {
        todo!()
    }
}