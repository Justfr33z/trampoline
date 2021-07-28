use crate::{Result, Error, JMP_SIZE};
use crate::bindings::Windows::Win32::System::Memory::{
    PAGE_PROTECTION_FLAGS,
    VirtualProtect,
    VirtualAlloc,
    VirtualFree,
    PAGE_EXECUTE_READWRITE,
    MEM_COMMIT,
    MEM_RESERVE,
    MEM_RELEASE
};
use std::ffi::c_void;
use std::ptr::{copy_nonoverlapping, write_bytes};

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
        if len < JMP_SIZE {
            return Err(Error::ToSmall);
        }

        let mut protection = PAGE_PROTECTION_FLAGS::default();

        unsafe {
            VirtualProtect(
            src,
            len,
            PAGE_EXECUTE_READWRITE,
            &mut protection
            )
        }.ok()?;

        let mut orig_bytes: Vec<u8> = vec![0x90; len];
        unsafe { copy_nonoverlapping(src, orig_bytes.as_mut_ptr() as *mut c_void, len); }
        unsafe { write_bytes(src, 0x90, len); }

        if cfg!(target_pointer_width = "32") {
            unsafe { *(src as *mut usize) = 0xE9; }
            unsafe {
                *(((src as *mut usize) as usize + 1) as *mut usize) =
                    (((dst as *mut isize) as isize - (src as *mut isize) as isize) - 5) as usize;
            }
        } else if cfg!(target_pointer_width = "64") {
            let mut jmp_bytes: [u8; 14] = [
                0xFF, 0x25, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ];

            let jmp_bytes_ptr = jmp_bytes.as_mut_ptr() as *mut c_void;

            unsafe {
                copy_nonoverlapping(
                    (&(dst as usize) as *const usize) as *mut c_void,
                    jmp_bytes_ptr.offset(6),
                    8
                );
            }

            unsafe { copy_nonoverlapping(jmp_bytes_ptr, src, JMP_SIZE); }
        } else {
            return Err(Error::InvalidTarget);
        }

        unsafe {
            VirtualProtect(
                src,
                len,
                protection,
                &mut protection
            )
        }.ok()?;

        Ok(Self { src, len, orig_bytes, active: true })
    }

    pub fn unhook(&mut self) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        let mut protection = PAGE_PROTECTION_FLAGS::default();

        unsafe {
            VirtualProtect(
                self.src,
                self.len,
                PAGE_EXECUTE_READWRITE,
                &mut protection
            )
        }.ok()?;

        unsafe {
            copy_nonoverlapping(
                self.orig_bytes.as_ptr() as *mut c_void,
                self.src,
                self.len
            );
        }

        unsafe {
            VirtualProtect(
                self.src,
                self.len,
                protection,
                &mut protection
            )
        }.ok()?;

        self.active = false;
        Ok(())
    }

    pub fn active(&self) -> bool {
        self.active
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        let _ = self.unhook();
    }
}

impl TrampolineHook {
    pub fn hook(src: *mut c_void, dst: *mut c_void, len: usize) -> Result<Self> {
        if len < JMP_SIZE {
            return Err(Error::ToSmall);
        }

        let gateway = unsafe {
            VirtualAlloc(
                0 as *mut c_void,
                len + JMP_SIZE,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_EXECUTE_READWRITE
            )
        };

        unsafe { copy_nonoverlapping(src, gateway, len); }

        if cfg!(target_pointer_width = "32") {
            unsafe { *(((gateway as *mut usize) as usize + len) as *mut usize) = 0xE9; }
            unsafe {
                *(((gateway as *mut usize) as usize + len + 1) as *mut usize) =
                    (((src as *mut isize) as isize - (gateway as *mut isize) as isize) - 5) as usize;
            }
        } else if cfg!(target_pointer_width = "64") {
            let mut jmp_bytes: [u8; 14] = [
                0xFF, 0x25, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
            ];

            let jmp_bytes_ptr = jmp_bytes.as_mut_ptr() as *mut c_void;

            unsafe {
                copy_nonoverlapping(
                    ((&((src as usize) + len)) as *const usize) as *mut c_void,
                    jmp_bytes_ptr.offset(6),
                    8
                );
            }

            unsafe {
                copy_nonoverlapping(
                    jmp_bytes_ptr,
                    ((gateway as usize) + len) as *mut c_void,
                    JMP_SIZE
                );
            }
        } else {
            return Err(Error::InvalidTarget);
        }

        let hook = Hook::hook(src, dst, len)?;
        Ok(Self { gateway, hook })
    }

    pub fn unhook(&mut self) -> Result<()> {
        if !self.active() {
            return Ok(());
        }

        unsafe { VirtualFree(self.gateway, 0, MEM_RELEASE) }.ok()?;
        self.hook.unhook()?;
        Ok(())
    }

    pub fn active(&self) -> bool {
        self.hook.active()
    }

    pub fn gateway(&self) -> *mut c_void {
        self.gateway
    }
}

impl Drop for TrampolineHook {
    fn drop(&mut self) {
        let _ = self.unhook();
    }
}