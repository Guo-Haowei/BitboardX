// wasm-pack build --target web
pub mod ai;
pub mod core;
pub mod game;

// #[cfg(target_arch = "wasm32")]
pub mod binding;

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

pub mod logger {
    pub fn log(message: String) {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::console::log_1(&message.into());
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("{}", message);
        }
    }
}
