fn main() {
    windows::build!(
        Windows::Win32::System::Memory::{
            VirtualProtect,
            VirtualAlloc,
            VirtualFree,
            PAGE_PROTECTION_FLAGS
        }
    );
}