use crate::engine::Engine;
use wasm_bindgen::prelude::*;

pub mod wasm_game;

#[wasm_bindgen]
pub fn name() -> String {
    Engine::name()
}
