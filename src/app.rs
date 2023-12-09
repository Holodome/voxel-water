use crate::camera::Camera;
use crate::map::{Cell, Map, WaterSim};
use crate::materials::Material;
use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::{MaterialDTO, SettingsDTO};
use crate::renderer::{Renderer, WorldDTO};
use crate::xorshift32::{self, Xorshift32, Xorshift32Seed};
use rand::SeedableRng;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

use crate::input::Input;

struct Settings {
    max_bounce_count: i32,
    maximum_traversal_distance: i32,
    enable_reproject: bool,
    enable_gauss: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_bounce_count: 4,
            maximum_traversal_distance: 64,
            enable_reproject: true,
            enable_gauss: true,
        }
    }
}

impl Settings {
    fn as_dto(&self) -> SettingsDTO {
        SettingsDTO {
            max_bounce_count: self.max_bounce_count,
            maximum_traversal_distance: self.maximum_traversal_distance,
            reproject: if self.enable_reproject { 1.0 } else { 0.0 },
        }
    }
}

pub struct App {
    settings: Settings,
    rng: xorshift32::Xorshift32,
    input: Input,
    camera: Camera,
    materials: Vec<Material>,
    // map: Map,
    map: WaterSim,
    renderer: Renderer,
    start_time: instant::Instant,
    last_time: instant::Instant,
    frame_counter: usize,
    sim_enabled: bool,
}

impl App {
    pub async fn new(window: winit::window::Window) -> Self {
        let window_size = window.inner_size();
        let aspect_ratio = (window_size.width as f32) / (window_size.height as f32);

        let settings = Settings::default();
        let mut camera = Camera::new(aspect_ratio, 60.0_f32.to_radians(), 0.1, 1000.0);
        camera.translate(Vector3::new(10.0, 10.0, 10.0) * 1.5);

        let mut rng = Xorshift32::from_seed(Xorshift32Seed(rand::random::<[u8; 4]>()));
        let mut perlin = Perlin::new(&mut rng);
        let map = Map::with_perlin(40, 20, 40, &mut perlin);
        let map = WaterSim::new(map);
        // let map = Map::random(10, 10, 10);
        // let map = Map::cube(10, 10, 10);

        let materials = vec![
            Material::diffuse(Vector3::new(0.0, 0.0, 0.0)),
            Material::diffuse(Vector3::new(
                0.44313725490196076,
                0.6666666666666666,
                0.20392156862745098,
            )),
            Material::dielectric(Vector3::new(0.5, 0.5, 0.9), 2.045),
            Material::metal(
                Vector3::new(0.6274509803921569, 0.3568627450980392, 0.3254901960784314),
                0.5,
            ),
        ];
        let material_dto = materials
            .iter()
            .map(|it| it.as_dto())
            .collect::<Vec<MaterialDTO>>();
        let dto = WorldDTO {
            camera: camera.as_dto(),
            map: map.as_dto(),
            materials: &material_dto,
            settings: settings.as_dto(),
        };
        let renderer = Renderer::new(window, &dto).await;
        let input = Input::default();

        let start_time = instant::Instant::now();
        Self {
            settings,
            rng,
            input,
            camera,
            materials,
            map,
            renderer,
            start_time,
            last_time: start_time,
            frame_counter: 0,
            sim_enabled: false,
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
                if !self.renderer.want_mouse_input() {
                    self.input.handle_mouse_button(*state, *button);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input.handle_mouse_move(*position);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, control_flow: &mut ControlFlow) {
        self.frame_counter += 1;
        if self.frame_counter % 20 == 0 && self.sim_enabled {
            // self.map.set_mass(20, 19, 20);
            self.map.simulate();
            self.renderer.update_map(self.map.as_dto());
        }

        let new_time = instant::Instant::now();
        let time_delta = new_time.duration_since(self.last_time);
        self.last_time = new_time;
        let rng_seed = new_time.duration_since(self.start_time).as_millis();
        self.renderer.update_random_seed(rng_seed as u32);
        self.renderer.update_time(time_delta);

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

        {
            let ui = unsafe {
                std::mem::transmute::<&mut imgui::Ui, &'static mut imgui::Ui>(
                    self.renderer.get_frame(),
                )
            };
            let renderer = &mut self.renderer;
            let window = ui.window("Settings");
            window
                .size([400.0, 400.0], imgui::Condition::FirstUseEver)
                .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                .build(|| {
                    let mut was_changed = false;
                    ui.text(format!("Frame time: {:?}", time_delta));
                    ui.checkbox("sim", &mut self.sim_enabled);
                    if imgui::Drag::new("bounce count")
                        .range(0, 128)
                        .build(ui, &mut self.settings.max_bounce_count)
                    {
                        was_changed = true;
                    }
                    if imgui::Drag::new("max distance")
                        .range(0, 128)
                        .build(ui, &mut self.settings.maximum_traversal_distance)
                    {
                        was_changed = true;
                    }
                    if ui.checkbox("Enable reproject", &mut self.settings.enable_reproject) {
                        was_changed = true;
                    }
                    if ui.checkbox("Enable gauss", &mut self.settings.enable_gauss) {
                        renderer.set_enable_gauss(self.settings.enable_gauss);
                        was_changed = true;
                    }
                    if imgui::Drag::new("camera positon")
                        .build_array(ui, &mut self.camera.position_as_slice())
                    {
                        self.camera.update_view_matrix();
                        camera_was_changed = true;
                    }
                    let mut pitch = self.camera.pitch_mut().to_degrees();
                    if imgui::Drag::new("camera pitch").build(ui, &mut pitch) {
                        *self.camera.pitch_mut() = pitch.to_radians();
                        self.camera.update_view_matrix();
                        camera_was_changed = true;
                    }
                    let mut yaw = self.camera.yaw_mut().to_degrees();
                    if imgui::Drag::new("camera yaw").build(ui, &mut yaw) {
                        *self.camera.yaw_mut() = yaw.to_radians();
                        self.camera.update_view_matrix();
                        camera_was_changed = true;
                    }

                    let mut materials_changed = false;
                    match &mut self.materials[2] {
                        Material::Dielectric {
                            albedo,
                            refractive_index,
                        } => {
                            materials_changed |= ui
                                .slider_config("water color", 0.0, 1.0)
                                .build_array(&mut albedo.as_mut_slice());
                            materials_changed |= imgui::Drag::new("water ior")
                                .speed(0.05)
                                .build(ui, refractive_index);
                        }
                        _ => {}
                    }
                    if materials_changed {
                        let material_dto = self
                            .materials
                            .iter()
                            .map(|it| it.as_dto())
                            .collect::<Vec<MaterialDTO>>();
                        renderer.update_materials(&material_dto);
                    }

                    if was_changed {
                        renderer.update_settings(self.settings.as_dto());
                    }
                });
        };

        self.renderer
            .update_camera(&self.camera.as_dto(), camera_was_changed);

        match self.renderer.render() {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.renderer.handle_lost_frame(),
            Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        }
        self.input.next_frame();
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut is_initialized = false;
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
                Event::NewEvents(cause) => {
                    if cause == StartCause::Poll {
                        is_initialized = true;
                    }
                }
                _ => {}
            }
            if is_initialized {
                self.renderer.handle_input(&event);
            }
        });
    }
}
