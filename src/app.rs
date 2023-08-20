use crate::renderer::{CameraDTO, MapDTO, Renderer, WorldDTO};
use crate::voxel_water::Map;
use crate::{math::*, renderer};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[derive(Debug)]
struct TimeRngProvider {
    last_time: std::time::SystemTime,
}

impl TimeRngProvider {
    fn new() -> Self {
        Self {
            last_time: std::time::SystemTime::now(),
        }
    }
}

impl renderer::RngProvider for TimeRngProvider {
    fn update(&mut self) -> u32 {
        let new_time = std::time::SystemTime::now();
        self.last_time = new_time;
        self.last_time
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u32
    }
}

pub struct App {
    map: Map,
    renderer: Renderer<TimeRngProvider>,
}

impl App {
    pub async fn new(window: winit::window::Window) -> Self {
        let focus_dist = 1.0;
        let camera_at = Point3::new(10.0, 10.0, 10.0) * 1.5;
        let look_at = Point3::new(0.0, 0.0, 0.0);
        let z = (camera_at - look_at).normalize();
        let x = Vector3::new(0.0, 1.0, 0.0).cross(&z).normalize();
        let y = z.cross(&x).normalize();
        let viewport_width = (60.0_f32.to_radians() * 0.5).tan() * 2.0;
        let viewport_height = viewport_width;
        let camera_horizontal = x * viewport_width;
        let camera_vertical = y * viewport_height;
        let camera_lower_left =
            camera_at - (camera_horizontal * 0.5) - (camera_vertical * 0.5) - (z * focus_dist);
        let camera_dto = CameraDTO {
            at: camera_at,
            lower_left: camera_lower_left,
            horizontal: camera_horizontal,
            vertical: camera_vertical,
        };
        let map = Map::random(10, 10, 10);
        let map_dto = map.to_dto();
        let dto = WorldDTO {
            camera: camera_dto,
            map: map_dto,
        };
        let rng_provider = TimeRngProvider::new();
        let renderer = Renderer::new(window, &dto, rng_provider).await;

        Self { map, renderer }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            control_flow.set_poll();
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == self.renderer.window().id() => {
                    if !self.input(event) {
                        match event {
                            WindowEvent::Resized(phys_size) => {
                                self.renderer.resize(*phys_size);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                self.renderer.resize(**new_inner_size);
                            }
                            WindowEvent::CloseRequested
                            | WindowEvent::KeyboardInput {
                                input:
                                    KeyboardInput {
                                        state: ElementState::Pressed,
                                        virtual_keycode: Some(VirtualKeyCode::Escape),
                                        ..
                                    },
                                ..
                            } => *control_flow = ControlFlow::Exit,
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) if window_id == self.renderer.window().id() => {
                    self.renderer.update();
                    match self.renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.renderer.handle_lost_frame(),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => self.renderer.window().request_redraw(),
                _ => {}
            }
        });
    }
}
