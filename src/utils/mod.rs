mod misc;

#[cfg(not(target_arch = "wasm32"))]
pub mod logger;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn init_logging() {
    wasm_logger::init(wasm_logger::Config::default());
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

pub use misc::*;
