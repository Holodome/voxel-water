use crate::renderer::{Renderer, WorldDTO};
use crate::voxel_water::{Camera, Map};
use crate::{math::*, renderer};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::input::Input;

pub struct App {
    input: Input,
    camera: Camera,
    map: Map,
    renderer: Renderer,
    last_time: instant::Instant,
}

impl App {
    pub async fn new(window: winit::window::Window) -> Self {
        let window_size = window.inner_size();
        let aspect_ratio = (window_size.width as f32) / (window_size.height as f32);

        let mut camera = Camera::new(aspect_ratio, 60.0_f32.to_radians(), 0.1, 1000.0);
        camera.translate(Vector3::new(10.0, 10.0, 10.0) * 1.5);

        let map = Map::random(10, 10, 10);
        let dto = WorldDTO {
            camera: camera.as_dto(),
            map: map.as_dto(),
        };
        let renderer = Renderer::new(window, &dto).await;
        let input = Input::default();

        Self {
            input,
            camera,
            map,
            renderer,
            last_time: instant::Instant::now(),
        }
    }

    fn input(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match event {
            WindowEvent::Resized(phys_size) => {
                self.renderer.resize(*phys_size);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                self.renderer.resize(**new_inner_size);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput { input: key_ev, .. } => {
                self.input.handle_key_event(*key_ev);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.input.handle_mouse_button(*state, *button);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_mouse_move(*position);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, control_flow: &mut ControlFlow) {
        let new_time = instant::Instant::now();
        let time_delta = new_time.duration_since(self.last_time);
        self.last_time = new_time;
        let rng_seed = new_time.elapsed().as_millis();
        println!("rng_seed, {:?}", rng_seed);
        self.renderer.update_random_seed(rng_seed as u32);

        let time_delta_s = (time_delta.as_micros() as f32) / 1_000_000.0;
        let mut dp = Vector3::zeros();
        if self.input.is_key_down(VirtualKeyCode::W) {
            dp.z = -1.0;
        }
        if self.input.is_key_down(VirtualKeyCode::S) {
            dp.z = 1.0;
        }
        if self.input.is_key_down(VirtualKeyCode::A) {
            dp.x = -1.0;
        }
        if self.input.is_key_down(VirtualKeyCode::D) {
            dp.x = 1.0;
        }
        if self.input.is_key_down(VirtualKeyCode::Space) {
            dp.y = 1.0;
        }
        if self.input.is_key_down(VirtualKeyCode::C) {
            dp.y = -1.0;
        }
        let mut camera_was_changed = false;
        if dp.dot(&dp) > 0.001 {
            let dp = dp.scale(time_delta_s * 2.0);
            self.camera.translate(dp);
            camera_was_changed = true;
        }
        if self.input.is_mouse_down(MouseButton::Left) {
            let delta = self.input.mouse_delta();
            let pitch_d = delta.x as f32 / 500.0;
            let yaw_d = delta.y as f32 / 500.0;
            self.camera.rotate(-yaw_d, -pitch_d);
            camera_was_changed = true;
        }

        if camera_was_changed {
            self.renderer.update_camera(&self.camera.as_dto());
        }

        match self.renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.renderer.handle_lost_frame(),
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        }
        self.input.next_frame();
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.renderer.window().id() => {
                    self.input(event, control_flow);
                }
                Event::RedrawRequested(window_id) if window_id == self.renderer.window().id() => {
                    self.render(control_flow);
                }
                Event::MainEventsCleared => self.renderer.window().request_redraw(),
                _ => {}
            }
        });
    }
}
