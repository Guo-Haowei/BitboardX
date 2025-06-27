#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

pub struct Timer {
    #[cfg(not(target_arch = "wasm32"))]
    start: Instant,

    #[cfg(target_arch = "wasm32")]
    start: f64,
}

impl Timer {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            Timer { start: Instant::now() }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let perf = web_sys::window()
                .expect("no global `window` exists")
                .performance()
                .expect("performance should be available");
            Timer { start: perf.now() }
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let duration = self.start.elapsed();
            duration.as_secs_f64() * 1000.0
        }
        #[cfg(target_arch = "wasm32")]
        {
            let perf = web_sys::window()
                .expect("no global `window` exists")
                .performance()
                .expect("performance should be available");
            perf.now() - self.start
        }
    }
}
