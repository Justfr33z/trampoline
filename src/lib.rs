pub use error::*;
pub use hook::{TrampolineHook, Hook};

mod error;
mod hook;

mod bindings {
    windows::include_bindings!();
}

#[cfg(target_pointer_width = "32")]
const JMP_MIN: usize = 5;

#[cfg(target_pointer_width = "64")]
const JMP_MIN: usize = 14;
