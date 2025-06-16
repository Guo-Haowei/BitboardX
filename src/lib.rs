// wasm-pack build --target web
pub mod engine;
pub mod game;

// @TODO: move this to a separate crate
#[macro_export]
macro_rules! named_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            let result = std::panic::catch_unwind(|| $body);
            let report = format!("{} at {}:{}", stringify!($name), file!(), line!());
            match result {
                Ok(_) => println!("✅ {}", report.green()),
                Err(err) => {
                    println!("❌ {}", report.red());
                    std::panic::resume_unwind(err);
                }
            }
        }
    };
}
