// Build: wasm-pack build --target web
use wasm_bindgen::prelude::*;

mod engine;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    engine::my_add(a, b)
}