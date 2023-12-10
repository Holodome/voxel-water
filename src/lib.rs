#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::{event_loop::EventLoop, window::WindowBuilder};

mod app;
mod camera;
mod input;
mod map;
mod materials;
mod math;
mod perlin;
mod renderer;
mod xorshift32;

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

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(960, 720))
        .with_title({
            #[cfg(feature = "russian")]
            let t = "Визуализация воды с использованием вокселей";
            #[cfg(not(feature = "russian"))]
            let t = "voxel water";
            t
        })
        .build(&event_loop)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.set_inner_size(PhysicalSize::new(960 * 2, 720 * 2));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Failed to create canvas");
    }

    let app = App::new(window).await;
    app.run(event_loop);
}
