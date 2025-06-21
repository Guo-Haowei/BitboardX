mod misc;

#[cfg(not(target_arch = "wasm32"))]
mod log;

#[cfg(target_arch = "wasm32")]
mod wasm_log;

pub use misc::*;
