use crate::engine::Engine;
use wasm_bindgen::prelude::*;

pub mod wasm_game;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn name() -> String {
    Engine::name()
}
