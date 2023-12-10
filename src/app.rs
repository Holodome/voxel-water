use crate::camera::Camera;
use crate::map::{Cell, Map, WaterSim};
use crate::materials::Material;
use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::{MaterialDTO, SettingsDTO};
use crate::renderer::{Renderer, WorldDTO};
use crate::xorshift32::{self, Xorshift32, Xorshift32Seed};
use egui_winit_platform::Platform;
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
            pad: 0.0,
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
    sim_divider: usize,

    water_source_coord: [usize; 3],
    source_enabled: bool,
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
            sim_divider: 10,
            water_source_coord: [20, 19, 20],
            source_enabled: false,
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
        self.frame_counter += 1;
        if self.frame_counter % self.sim_divider == 0 && self.sim_enabled {
            if self.source_enabled {
                self.map.set_mass(
                    self.water_source_coord[0],
                    self.water_source_coord[1],
                    self.water_source_coord[2],
                );
            }
            if self.map.simulate() {
                self.renderer.update_map(self.map.as_dto());
            }
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
            self.camera.rotate(-pitch_d, -yaw_d);
            camera_was_changed = true;
        }

        let egui_ctx = self.renderer.begin_ui_frame();
        egui::Window::new("Settings").show(&egui_ctx, |ui| {
            let mut was_changed = false;
            #[cfg(feature = "russian")]
            ui.label(format!("Время кадра: {:?}", time_delta));
            #[cfg(not(feature = "russian"))]
            ui.label(format!("Frame time: {:?}", time_delta));

            #[cfg(feature = "russian")]
            ui.checkbox(&mut self.sim_enabled, "включить симуляцию");
            #[cfg(not(feature = "russian"))]
            ui.checkbox(&mut self.sim_enabled, "sim enabled");
            ui.horizontal(|ui| {
                ui.add(egui::DragValue::new(&mut self.sim_divider).clamp_range(1..=32));

                #[cfg(feature = "russian")]
                ui.label("частота симуляции");
                #[cfg(not(feature = "russian"))]
                ui.label("sim divider");
            });
            ui.horizontal(|ui| {
                was_changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.settings.max_bounce_count)
                            .clamp_range(0..=128),
                    )
                    .dragged();

                #[cfg(feature = "russian")]
                ui.label("число отскоков");
                #[cfg(not(feature = "russian"))]
                ui.label("bounce count");
            });
            ui.horizontal(|ui| {
                was_changed |= ui
                    .add(
                        egui::DragValue::new(&mut self.settings.maximum_traversal_distance)
                            .clamp_range(0..=128),
                    )
                    .dragged();

                #[cfg(feature = "russian")]
                ui.label("дальность отрисовки");
                #[cfg(not(feature = "russian"))]
                ui.label("max distance");
            });

            #[cfg(feature = "russian")]
            {
                was_changed |= ui
                    .checkbox(&mut self.settings.enable_reproject, "включить репроекцию")
                    .clicked();
            }
            #[cfg(not(feature = "russian"))]
            {
                was_changed |= ui
                    .checkbox(&mut self.settings.enable_reproject, "enable reproject")
                    .clicked();
            }

            if was_changed {
                self.renderer.update_settings(self.settings.as_dto());
            }

            #[cfg(feature = "russian")]
            {
                if ui
                    .checkbox(
                        &mut self.settings.enable_gauss,
                        "включить размытие по Гауссу",
                    )
                    .clicked()
                {
                    self.renderer.set_enable_gauss(self.settings.enable_gauss);
                }
            }
            #[cfg(not(feature = "russian"))]
            {
                if ui
                    .checkbox(&mut self.settings.enable_gauss, "enable gauss")
                    .clicked()
                {
                    self.renderer.set_enable_gauss(self.settings.enable_gauss);
                }
            }

            ui.horizontal(|ui| {
                if ui
                    .add(egui::DragValue::new(
                        &mut self.camera.position_as_slice()[0],
                    ))
                    .dragged()
                {
                    camera_was_changed = true;
                    self.camera.update_view_matrix();
                }
                if ui
                    .add(egui::DragValue::new(
                        &mut self.camera.position_as_slice()[1],
                    ))
                    .dragged()
                {
                    camera_was_changed = true;
                    self.camera.update_view_matrix();
                }
                if ui
                    .add(egui::DragValue::new(
                        &mut self.camera.position_as_slice()[2],
                    ))
                    .dragged()
                {
                    camera_was_changed = true;
                    self.camera.update_view_matrix();
                }

                #[cfg(feature = "russian")]
                ui.label("координаты камеры");
                #[cfg(not(feature = "russian"))]
                ui.label("camera position");
            });
            ui.horizontal(|ui| {
                let mut pitch = self.camera.pitch_mut().to_degrees();
                if ui.add(egui::DragValue::new(&mut pitch)).dragged() {
                    *self.camera.pitch_mut() = pitch.to_radians();
                    self.camera.update_view_matrix();
                    camera_was_changed = true;
                }
                #[cfg(feature = "russian")]
                ui.label("тангаж камеры");
                #[cfg(not(feature = "russian"))]
                ui.label("camera pitch");
            });
            ui.horizontal(|ui| {
                let mut yaw = self.camera.yaw_mut().to_degrees();
                if ui.add(egui::DragValue::new(&mut yaw)).dragged() {
                    *self.camera.yaw_mut() = yaw.to_radians();
                    self.camera.update_view_matrix();
                    camera_was_changed = true;
                }
                #[cfg(feature = "russian")]
                ui.label("рысканье камеры");
                #[cfg(not(feature = "russian"))]
                ui.label("camera yaw");
            });
            let mut materials_changed = false;
            match &mut self.materials[2] {
                Material::Dielectric {
                    albedo,
                    refractive_index,
                } => {
                    ui.horizontal(|ui| {
                        materials_changed |= ui
                            .add(egui::Slider::new(&mut albedo.x, 0.0..=1.0))
                            .dragged();
                        materials_changed |= ui
                            .add(egui::Slider::new(&mut albedo.y, 0.0..=1.0))
                            .dragged();
                        materials_changed |= ui
                            .add(egui::Slider::new(&mut albedo.z, 0.0..=1.0))
                            .dragged();
                        #[cfg(feature = "russian")]
                        ui.label("цвет воды");
                        #[cfg(not(feature = "russian"))]
                        ui.label("water color");
                    });
                    ui.horizontal(|ui| {
                        materials_changed |= ui
                            .add(egui::DragValue::new(refractive_index).speed(0.05))
                            .dragged();
                        #[cfg(feature = "russian")]
                        ui.label("коэффициент преломления воды");
                        #[cfg(not(feature = "russian"))]
                        ui.label("water ior");
                    });
                }
                _ => {}
            }
            ui.horizontal(|ui| {
                materials_changed |= ui
                    .add(egui::Slider::new(
                        &mut self.water_source_coord[0],
                        1..=self.map.x() - 2,
                    ))
                    .dragged();
                materials_changed |= ui
                    .add(egui::Slider::new(
                        &mut self.water_source_coord[1],
                        1..=self.map.y() - 2,
                    ))
                    .dragged();
                materials_changed |= ui
                    .add(egui::Slider::new(
                        &mut self.water_source_coord[2],
                        1..=self.map.z() - 2,
                    ))
                    .dragged();
                #[cfg(feature = "russian")]
                ui.label("координаты источника воды");
                #[cfg(not(feature = "russian"))]
                ui.label("water source coord");
            });
            #[cfg(feature = "russian")]
            ui.checkbox(&mut self.source_enabled, "включить источник воды");
            #[cfg(not(feature = "russian"))]
            ui.checkbox(&mut self.source_enabled, "water source enable");
            if {
                #[cfg(feature = "russian")]
                let result = ui.button("сбросить сцену").clicked();
                #[cfg(not(feature = "russian"))]
                let result = ui.button("reset scene").clicked();
                result
            } {
                let mut rng = Xorshift32::from_seed(Xorshift32Seed(rand::random::<[u8; 4]>()));
                let mut perlin = Perlin::new(&mut rng);
                let map = Map::with_perlin(40, 20, 40, &mut perlin);
                let map = WaterSim::new(map);
                self.map = map;

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
                self.materials = materials;
                self.renderer.update_map(self.map.as_dto());
                materials_changed = true;
            }
            if materials_changed {
                let material_dto = self
                    .materials
                    .iter()
                    .map(|it| it.as_dto())
                    .collect::<Vec<MaterialDTO>>();
                self.renderer.update_materials(&material_dto);
            }
        });

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
            if is_initialized {
                if self.renderer.handle_input(&event) {
                    return;
                }
            }
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
        });
    }
}
