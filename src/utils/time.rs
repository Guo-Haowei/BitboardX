#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_time() -> f64 {
    // Uses `performance.now()` in the browser for high precision milliseconds
    web_sys::js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn get_time() -> f64 {
    // Uses std::time::Instant for monotonic high precision timer
    use std::time::Instant;
    static START: once_cell::sync::Lazy<Instant> = once_cell::sync::Lazy::new(Instant::now);

    let elapsed = Instant::now().duration_since(*START);
    elapsed.as_secs_f64() * 1000.0 // convert seconds to milliseconds
}
