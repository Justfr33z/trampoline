use crate::Result;
use std::ffi::c_void;

pub struct Hook;

pub struct TrampolineHook;

impl Hook {
    pub fn hook(src: *mut c_void, dst: *const c_void, len: usize) -> Result<Self> {
        todo!()
    }

    pub fn unhook(&self) -> Result<()> {
        todo!()
    }

    pub fn active(&self) -> bool {
        todo!()
    }
}