mod misc;

#[cfg(not(target_arch = "wasm32"))]
mod log;

#[cfg(target_arch = "wasm32")]
mod log_wasm;

pub use misc::*;
