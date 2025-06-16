pub mod move_gen;
pub mod position;
pub mod types;
pub mod utils;

use wasm_bindgen::prelude::*;

pub const NAME: &str = "BitboardX";
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 2;

pub fn version() -> String {
    format!("{}.{}.{}", VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH)
}

#[wasm_bindgen]
pub fn name() -> String {
    format!("{} {}", NAME, version())
}
