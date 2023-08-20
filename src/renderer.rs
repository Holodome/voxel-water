use crate::math::*;
use rand::Rng;
use wgpu::util::DeviceExt;
use winit::{event::*, window::Window};

#[derive(Clone, Debug)]
pub struct CameraDTO {
    pub at: Point3,
    pub lower_left: Point3,
    pub horizontal: Vector3,
    pub vertical: Vector3,
}

#[derive(Clone, Debug)]
pub struct MapDTO<'a> {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub cells: &'a [u8],
}

#[derive(Clone, Debug)]
pub struct WorldDTO<'a> {
    pub camera: CameraDTO,
    pub map: MapDTO<'a>,
}

pub trait RngProvider {
    fn update(&mut self) -> u32;
}

pub struct Renderer<Rng> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,

    num_vertices: u32,
    world: WorldUniform,
    world_buffer: wgpu::Buffer,
    rng_seed: u32,
    rng_buffer: wgpu::Buffer,
    ray_tracing_bind_group: wgpu::BindGroup,

    rng_provider: Rng,
}

impl<Rng> Renderer<Rng>
where
    Rng: RngProvider,
{
    pub async fn new<'a>(window: Window, dto: &WorldDTO<'a>, mut rng_provider: Rng) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(&window) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptionsBase {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ray_tracing.wgsl").into()),
        });
        let num_vertices = DISPLAY_VERTICES.len() as u32;

        let world = WorldUniform::from_dto(&dto.camera);
        let world_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("world buffer"),
            contents: bytemuck::cast_slice(&[world]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let rng_seed = rng_provider.update();
        let rng_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rng buffer"),
            contents: bytemuck::bytes_of(&rng_seed),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let voxel_texture_size = wgpu::Extent3d {
            width: dto.map.x as u32,
            height: dto.map.y as u32,
            depth_or_array_layers: dto.map.z as u32,
        };
        let voxel_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: voxel_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("voxel texture"),
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &voxel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(dto.map.cells),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: voxel_texture_size.width.into(),
                rows_per_image: voxel_texture_size.height.into(),
            },
            voxel_texture_size,
        );
        let voxel_texture_view = voxel_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let ray_tracing_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ray tracing bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let ray_tracing_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ray tracing bind group"),
            layout: &ray_tracing_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: world_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&voxel_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: rng_buffer.as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[&ray_tracing_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DisplayVertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(DISPLAY_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            num_vertices,
            world,
            world_buffer,
            rng_seed,
            rng_buffer,
            ray_tracing_bind_group,
            rng_provider,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn handle_lost_frame(&mut self) {
        self.resize(self.size);
    }
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.surface.configure(&self.device, &self.config);
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.rng_seed = self.rng_provider.update();
        self.queue
            .write_buffer(&self.rng_buffer, 0, bytemuck::bytes_of(&self.rng_seed));

        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.ray_tracing_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..self.num_vertices, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct DisplayVertex {
    pub position: Point2,
}

impl DisplayVertex {
    const ATTRS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<DisplayVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}

const DISPLAY_VERTICES: &[DisplayVertex] = &[
    DisplayVertex {
        position: Point2::new(0.0, 0.0),
    },
    DisplayVertex {
        position: Point2::new(1.0, 1.0),
    },
    DisplayVertex {
        position: Point2::new(0.0, 1.0),
    },
    DisplayVertex {
        position: Point2::new(0.0, 0.0),
    },
    DisplayVertex {
        position: Point2::new(1.0, 0.0),
    },
    DisplayVertex {
        position: Point2::new(1.0, 1.0),
    },
];

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct WorldUniform {
    pub camera_at: Point3,
    pub _pad0: u32,
    pub camera_lower_left: Point3,
    pub _pad1: u32,
    pub camera_horizontal: Vector3,
    pub _pad2: u32,
    pub camera_vertical: Vector3,
    pub _pad3: u32,
}

impl WorldUniform {
    fn from_dto(dto: &CameraDTO) -> Self {
        Self {
            camera_at: dto.at,
            _pad0: 0,
            camera_lower_left: dto.lower_left,
            _pad1: 0,
            camera_horizontal: dto.horizontal,
            _pad2: 0,
            camera_vertical: dto.vertical,
            _pad3: 0,
        }
    }
}
