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

/// A 32 or 64 bit hook.
///
/// After creating a `Hook` by [`hook`]ing a function, it redirects the control flow.
///
/// The function will be unhooked when the value is dropped.
///
/// [`hook`]: #method.hook
///
/// # Examples
///
/// ```no_run
/// use crate::bindings::Windows::Win32::Foundation::{HANDLE, BOOL};
/// use crate::bindings::Windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
/// use std::ffi::c_void;
/// use std::mem::transmute;
/// use trampoline::Hook;
///
/// mod bindings {
///     windows::include_bindings!();
/// }
///
/// pub extern "stdcall" fn wgl_swap_buffers(hdc: HANDLE) -> BOOL {
///     BOOL::from(true)
/// }
///
/// fn main() {
///     let module = unsafe { GetModuleHandleA("opengl32.dll") };
///     let src_wgl_swap_buffers = unsafe {
///         GetProcAddress(module, "wglSwapBuffers")
///     }.unwrap();
///
///     let hook = Hook::hook(
///         src_wgl_swap_buffers as *mut c_void,
///         wgl_swap_buffers as *mut c_void,
///         21
///     ).unwrap();
/// }
/// ```
pub struct Hook {
    src: *mut c_void,
    len: usize,
    orig_bytes: Vec<u8>,
    active: bool,
}


/// A 32 or 64 bit trampoline hook.
///
/// After creating a `TrampolineHook` by [`hook`]ing a function, it redirects the control flow.
///
/// The function will be unhooked when the value is dropped.
///
/// [`hook`]: #method.hook
///
/// # Examples
///
/// ```no_run
/// use crate::bindings::Windows::Win32::Foundation::{HANDLE, BOOL};
/// use crate::bindings::Windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
/// use std::ffi::c_void;
/// use std::sync::Mutex;
/// use std::mem::transmute;
/// use once_cell::sync::Lazy;
/// use trampoline::TrampolineHook;
///
/// mod bindings {
///     windows::include_bindings!();
/// }
///
/// static HOOK: Lazy<Mutex<Option<TrampolineHook>>> = Lazy::new(|| {
///     Mutex::new(None)
/// });
///
/// pub extern "stdcall" fn wgl_swap_buffers(hdc: HANDLE) -> BOOL {
///     let gateway = HOOK
///         .lock()
///         .unwrap()
///         .as_ref()
///         .unwrap()
///         .gateway();
///
///     let gateway_call: extern "stdcall" fn(hdc: HANDLE) -> BOOL;
///     gateway_call = unsafe { transmute(gateway) };
///     gateway_call(hdc);
///
///     BOOL::from(true)
/// }
///
/// fn main() {
///     let module = unsafe { GetModuleHandleA("opengl32.dll") };
///     let src_wgl_swap_buffers = unsafe {
///         GetProcAddress(module, "wglSwapBuffers")
///     }.unwrap();
///
///     let hook = TrampolineHook::hook(
///         src_wgl_swap_buffers as *mut c_void,
///         wgl_swap_buffers as *mut c_void,
///         21
///     ).unwrap();
///
///     *HOOK
///         .lock()
///         .unwrap() = Some(hook);
/// }
/// ```
pub struct TrampolineHook {
    gateway: *mut c_void,
    hook: Hook,
}

impl Hook {
    /// Hooks a function.
    ///
    /// `src` is the function to be hooked.
    ///
    /// `dst` is the destination of the hook.
    ///
    /// `len` is the amount of bytes that should be overridden.
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

    /// Unhooks the function.
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

    /// Returns the state of this hook.
    pub fn active(&self) -> bool {
        self.active
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        let _ = self.unhook();
    }
}

unsafe impl Sync for Hook { }
unsafe impl Send for Hook { }

impl TrampolineHook {
    /// Hooks a function and allocates a gateway with the overridden bytes.
    ///
    /// `src` is the function to be hooked.
    ///
    /// `dst` is the destination of the hook.
    ///
    /// `len` is the amount of bytes that should be overridden.
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

    /// Unhooks the function and deallocates the gateway.
    pub fn unhook(&mut self) -> Result<()> {
        if !self.active() {
            return Ok(());
        }

        unsafe { VirtualFree(self.gateway, 0, MEM_RELEASE) }.ok()?;
        self.hook.unhook()?;
        Ok(())
    }

    /// Returns the state of this hook.
    pub fn active(&self) -> bool {
        self.hook.active()
    }

    /// Returns the allocated gateway of this hook.
    pub fn gateway(&self) -> *mut c_void {
        self.gateway
    }
}

impl Drop for TrampolineHook {
    fn drop(&mut self) {
        let _ = self.unhook();
    }
}

unsafe impl Sync for TrampolineHook { }
unsafe impl Send for TrampolineHook { }