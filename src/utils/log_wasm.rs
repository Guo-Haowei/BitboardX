#[cfg(target_arch = "wasm32")]
pub fn init_logging() {
    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();
}
