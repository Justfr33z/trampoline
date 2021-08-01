# trampoline

`trampoline` is a function hooking library that supports both 32 and 64 bit.

## Example

```rust,ignore
use crate::bindings::Windows::Win32::Foundation::{HANDLE, BOOL};
use crate::bindings::Windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use std::ffi::c_void;
use std::sync::Mutex;
use std::mem::transmute;
use once_cell::sync::Lazy;
use trampoline::TrampolineHook;

mod bindings {
    windows::include_bindings!();
}

static HOOK: Lazy<Mutex<Option<TrampolineHook>>> = Lazy::new(|| {
    Mutex::new(None)
});

pub extern "stdcall" fn wgl_swap_buffers(hdc: HANDLE) -> BOOL {
    let gateway = HOOK
        .lock()
        .unwrap()
        .as_ref()
        .unwrap()
        .gateway();

    let gateway_call: extern "stdcall" fn(hdc: HANDLE) -> BOOL;
    gateway_call = unsafe { transmute(gateway) };
    gateway_call(hdc);

    BOOL::from(true)
}

fn main() {
    let module = unsafe { GetModuleHandleA("opengl32.dll") };
    let src_wgl_swap_buffers = unsafe {
        GetProcAddress(module, "wglSwapBuffers")
    }.unwrap();

    let hook = TrampolineHook::hook(
        src_wgl_swap_buffers as *mut c_void,
        wgl_swap_buffers as *mut c_void,
        21
    ).unwrap();

    *HOOK
        .lock()
        .unwrap() = Some(hook);
}

```

## OS support

| OS      | Architecture |
|---------|--------------|
| Windows | x32, x64     |

## License

This project is licensed under [MIT License](https://choosealicense.com/licenses/mit/) ([LICENSE-MIT](LICENSE.md))