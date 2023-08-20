#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

mod app;
mod math;
mod perlin;
mod renderer;
mod voxel_water;

use app::App;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch="wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Failed to initialize logger");
        } else {
            env_logger::init();
        }
    }

    App::new().await.run();
}
