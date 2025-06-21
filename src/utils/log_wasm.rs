use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init_logging() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
}
