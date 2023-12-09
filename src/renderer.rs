use crate::math::*;
use wgpu::util::DeviceExt;
use winit::window::Window;

struct Imgui {
    pub imgui: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl Imgui {
    fn new(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        let hidpi_factor = window.scale_factor();
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    oversample_h: 1,
                    pixel_snap_h: true,
                    size_pixels: font_size,
                    ..Default::default()
                }),
            }]);

        let renderer = imgui_wgpu::Renderer::new(
            &mut imgui,
            device,
            queue,
            imgui_wgpu::RendererConfig {
                texture_format,
                ..Default::default()
            },
        );
        Self {
            imgui,
            platform,
            renderer,
        }
    }

    fn update_time(&mut self, delta: std::time::Duration) {
        self.imgui.io_mut().update_delta_time(delta);
    }

    fn get_frame(&mut self) -> &mut imgui::Ui {
        self.imgui.frame()
    }
}

#[derive(Clone, Debug)]
pub struct CameraDTO {
    pub view_matrix: Matrix4,
    pub projection_matrix: Matrix4,
    pub inverse_projection_matrix: Matrix4,
}

#[derive(Clone, Debug)]
pub struct MapDTO<'a> {
    pub x: usize,
    pub y: usize,
    pub z: usize,
    pub cells: &'a [u8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SettingsDTO {
    pub max_bounce_count: i32,
    pub maximum_traversal_distance: i32,
    pub reproject: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialDTO {
    pub albedo: Vector3,
    pub fuzz: f32,
    pub refractive_index: f32,
    pub kind: i32,
}

#[derive(Clone, Debug)]
pub struct WorldDTO<'a> {
    pub camera: CameraDTO,
    pub map: MapDTO<'a>,
    pub materials: &'a [MaterialDTO],
    pub settings: SettingsDTO,
}

struct TargetTextures {
    prev_color_texture: wgpu::Texture,
    prev_normal_texture: wgpu::Texture,
    prev_mat_texture: wgpu::Texture,
    prev_offset_texture: wgpu::Texture,

    prev_color_texture_view: wgpu::TextureView,
    prev_normal_texture_view: wgpu::TextureView,
    prev_mat_texture_view: wgpu::TextureView,
    prev_offset_texture_view: wgpu::TextureView,
}

impl TargetTextures {
    fn new(device: &wgpu::Device, prev_texture_size: &wgpu::Extent3d) -> Self {
        let prev_color_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: prev_texture_size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("prev color texture"),
            view_formats: &[],
        });
        let prev_color_texture_view = prev_color_texture.create_view(&Default::default());
        let prev_normal_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: prev_texture_size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("prev normal texture"),
            view_formats: &[],
        });
        let prev_normal_texture_view = prev_normal_texture.create_view(&Default::default());
        let prev_mat_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: prev_texture_size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("prev mat texture"),
            view_formats: &[],
        });
        let prev_mat_texture_view = prev_mat_texture.create_view(&Default::default());
        let prev_offset_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: prev_texture_size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("prev offset texture"),
            view_formats: &[],
        });
        let prev_offset_texture_view = prev_offset_texture.create_view(&Default::default());
        Self {
            prev_color_texture,
            prev_normal_texture,
            prev_mat_texture,
            prev_offset_texture,

            prev_color_texture_view,
            prev_normal_texture_view,
            prev_mat_texture_view,
            prev_offset_texture_view,
        }
    }

    fn bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("textures bind group"),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.prev_color_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.prev_normal_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.prev_mat_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(&self.prev_offset_texture_view),
                },
            ],
        })
    }

    fn present_bind_group(
        &self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("textures bind group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.prev_color_texture_view),
            }],
        })
    }
}

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,

    voxel_texture_size: wgpu::Extent3d,
    voxel_texture: wgpu::Texture,
    rng_buffer: wgpu::Buffer,
    inverse_projection_matrix: wgpu::Buffer,
    projection_matrix: wgpu::Buffer,
    view_matrix: wgpu::Buffer,

    target_textures: [TargetTextures; 2],

    prev_view_matrix: wgpu::Buffer,
    settings_buffer: wgpu::Buffer,

    ray_tracing_bind_group: wgpu::BindGroup,
    targets_bind_groups: [wgpu::BindGroup; 2],
    targets_ping_pong: bool,

    present_pipeline: wgpu::RenderPipeline,
    present_sampl_bind_group: wgpu::BindGroup,
    present_tex_bind_groups: [wgpu::BindGroup; 2],

    gauss_vert_pipeline: wgpu::RenderPipeline,
    gauss_horiz_pipeline: wgpu::RenderPipeline,
    gauss_vert_texture_view: wgpu::TextureView,
    gauss_vert_bind_group: wgpu::BindGroup,

    last_view_matrix: Matrix4,
    should_update_last_view_matrix: bool,

    imgui: Imgui,
    gauss_enabled: bool,
}

impl Renderer {
    pub async fn new<'a>(window: Window, dto: &WorldDTO<'a>) -> Self {
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
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/ray_tracing.wgsl").into()),
        });

        let present_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("present shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/present.wgsl").into()),
        });
        let gauss_vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gauss vert"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gauss_vert.wgsl").into()),
        });
        let gauss_horiz_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gauss horiz"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/gauss_horiz.wgsl").into()),
        });

        let rng_seed = [0u32; 4];
        let rng_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rng buffer"),
            contents: bytemuck::bytes_of(&rng_seed),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let inverse_projection_matrix =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("inverse projection matrix"),
                contents: bytemuck::bytes_of(&dto.camera.inverse_projection_matrix),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let projection_matrix = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("projection matrix"),
            contents: bytemuck::bytes_of(&dto.camera.projection_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let view_matrix = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view matrix"),
            contents: bytemuck::bytes_of(&dto.camera.view_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let textures_vec = {
            let mut v = Vec::<MaterialDTO>::with_capacity(256);
            v.resize_with(256, || MaterialDTO {
                albedo: Vector3::zeros(),
                fuzz: 0.0,
                refractive_index: 0.0,
                kind: 0,
            });
            for (i, it) in dto.materials.iter().enumerate() {
                v[i] = it.clone();
            }

            v
        };
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("materials"),
            contents: bytemuck::cast_slice(&textures_vec),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let settings_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("settings"),
            contents: bytemuck::bytes_of(&dto.settings),
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
        let prev_texture_size = wgpu::Extent3d {
            width: size.width,
            height: size.height,
            depth_or_array_layers: 1,
        };
        let target_textures = [
            TargetTextures::new(&device, &prev_texture_size),
            TargetTextures::new(&device, &prev_texture_size),
        ];

        let prev_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let prev_view_matrix = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("prev view matrix"),
            contents: bytemuck::bytes_of(&dto.camera.view_matrix),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let texture_size_v = Vector2::new(size.width as f32, size.height as f32);
        let texture_size_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("texture size"),
            contents: bytemuck::bytes_of(&texture_size_v),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let gauss_vert_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: prev_texture_size.clone(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("gauss vert texture"),
            view_formats: &[],
        });
        let gauss_vert_texture_view = gauss_vert_texture.create_view(&Default::default());

        let ray_tracing_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ray tracing bind group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Uint,
                            view_dimension: wgpu::TextureViewDimension::D3,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
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
                    resource: wgpu::BindingResource::TextureView(&voxel_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: rng_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: inverse_projection_matrix.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: projection_matrix.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: view_matrix.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&prev_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: prev_view_matrix.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: material_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: settings_buffer.as_entire_binding(),
                },
            ],
        });
        let targets_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("targets group layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                ],
            });
        let targets_bind_groups = [
            target_textures[0].bind_group(&device, &targets_bind_group_layout),
            target_textures[1].bind_group(&device, &targets_bind_group_layout),
        ];
        let present_tex_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("present bind group"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                }],
            });
        let present_tex_bind_groups = [
            target_textures[0].present_bind_group(&device, &present_tex_bind_group_layout),
            target_textures[1].present_bind_group(&device, &present_tex_bind_group_layout),
        ];
        let gauss_vert_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("gauss bind group"),
            layout: &present_tex_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&gauss_vert_texture_view),
            }],
        });
        let present_sampl_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("present bind group"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let present_sampl_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("present sampl bind group"),
            layout: &present_sampl_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&prev_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: texture_size_buffer.as_entire_binding(),
                },
            ],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render pipeline layout"),
                bind_group_layouts: &[&ray_tracing_bind_group_layout, &targets_bind_group_layout],
                push_constant_ranges: &[],
            });
        let present_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("present pipeline layout"),
                bind_group_layouts: &[
                    &present_tex_bind_group_layout,
                    &present_sampl_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let gauss_horiz_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gauss vert pipeline"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gauss_horiz_shader,
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
                module: &gauss_horiz_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let gauss_vert_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("gauss vert pipeline"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &gauss_vert_shader,
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
                module: &gauss_vert_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });
        let present_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("present pipeline"),
            layout: Some(&present_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &present_shader,
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
                module: &present_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
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
                targets: &[
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                    Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::R32Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    }),
                ],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex buffer"),
            contents: bytemuck::cast_slice(DISPLAY_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let imgui = Imgui::new(&window, &device, &queue, config.format);

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,

            voxel_texture_size,
            voxel_texture,
            rng_buffer,
            inverse_projection_matrix,
            projection_matrix,
            view_matrix,

            target_textures,
            prev_view_matrix,
            settings_buffer,

            ray_tracing_bind_group,
            targets_bind_groups,
            targets_ping_pong: false,

            present_pipeline,
            present_sampl_bind_group,
            present_tex_bind_groups,

            gauss_vert_pipeline,
            gauss_horiz_pipeline,
            gauss_vert_texture_view,
            gauss_vert_bind_group,

            last_view_matrix: dto.camera.view_matrix.try_inverse().unwrap(),
            should_update_last_view_matrix: true,

            imgui,
            gauss_enabled: true,
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
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.target_textures[self.targets_ping_pong as usize]
                            .prev_color_texture_view,
                        resolve_target: None,
                        ops: Default::default(),
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.target_textures[self.targets_ping_pong as usize]
                            .prev_normal_texture_view,
                        resolve_target: None,
                        ops: Default::default(),
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.target_textures[self.targets_ping_pong as usize]
                            .prev_mat_texture_view,
                        resolve_target: None,
                        ops: Default::default(),
                    }),
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.target_textures[self.targets_ping_pong as usize]
                            .prev_offset_texture_view,
                        resolve_target: None,
                        ops: Default::default(),
                    }),
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.ray_tracing_bind_group, &[]);
            render_pass.set_bind_group(
                1,
                &self.targets_bind_groups[!self.targets_ping_pong as usize],
                &[],
            );
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..DISPLAY_VERTICES.len() as u32, 0..1);
        }
        if self.gauss_enabled {
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("gauss vert render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.gauss_vert_texture_view,
                        resolve_target: None,
                        ops: Default::default(),
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.gauss_vert_pipeline);
                render_pass.set_bind_group(
                    0,
                    &self.present_tex_bind_groups[!self.targets_ping_pong as usize],
                    &[],
                );
                render_pass.set_bind_group(1, &self.present_sampl_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..DISPLAY_VERTICES.len() as u32, 0..1);
            }
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("gauss horiz render pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: Default::default(),
                    })],
                    depth_stencil_attachment: None,
                });

                render_pass.set_pipeline(&self.gauss_horiz_pipeline);
                render_pass.set_bind_group(0, &self.gauss_vert_bind_group, &[]);
                render_pass.set_bind_group(1, &self.present_sampl_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass.draw(0..DISPLAY_VERTICES.len() as u32, 0..1);

                self.imgui
                    .renderer
                    .render(
                        self.imgui.imgui.render(),
                        &self.queue,
                        &self.device,
                        &mut render_pass,
                    )
                    .unwrap();
            }
        } else {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Default::default(),
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.present_pipeline);
            render_pass.set_bind_group(
                0,
                &self.present_tex_bind_groups[!self.targets_ping_pong as usize],
                &[],
            );
            render_pass.set_bind_group(1, &self.present_sampl_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..DISPLAY_VERTICES.len() as u32, 0..1);

            self.imgui
                .renderer
                .render(
                    self.imgui.imgui.render(),
                    &self.queue,
                    &self.device,
                    &mut render_pass,
                )
                .unwrap();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        self.targets_ping_pong = !self.targets_ping_pong;

        Ok(())
    }

    pub fn update_time(&mut self, time: std::time::Duration) {
        self.imgui.update_time(time);
    }

    pub fn update_random_seed(&mut self, seed: u32) {
        self.queue
            .write_buffer(&self.rng_buffer, 0, bytemuck::bytes_of(&seed));
    }
    pub fn update_camera(&mut self, camera: &CameraDTO, camera_was_changed: bool) {
        if self.should_update_last_view_matrix && !camera_was_changed {
            self.last_view_matrix = camera.view_matrix.try_inverse().unwrap();
            self.queue.write_buffer(
                &self.prev_view_matrix,
                0,
                bytemuck::bytes_of(&self.last_view_matrix),
            );
            self.should_update_last_view_matrix = false;
        }
        if camera_was_changed {
            self.should_update_last_view_matrix = true;
            self.queue.write_buffer(
                &self.prev_view_matrix,
                0,
                bytemuck::bytes_of(&self.last_view_matrix),
            );
            self.queue.write_buffer(
                &self.view_matrix,
                0,
                bytemuck::bytes_of(&camera.view_matrix),
            );
            self.queue.write_buffer(
                &self.projection_matrix,
                0,
                bytemuck::bytes_of(&camera.projection_matrix),
            );
            self.queue.write_buffer(
                &self.inverse_projection_matrix,
                0,
                bytemuck::bytes_of(&camera.inverse_projection_matrix),
            );
            self.last_view_matrix = camera.view_matrix.try_inverse().unwrap();
        }
    }

    pub fn update_map(&mut self, dto: MapDTO) {
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.voxel_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(dto.cells),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: self.voxel_texture_size.width.into(),
                rows_per_image: self.voxel_texture_size.height.into(),
            },
            self.voxel_texture_size,
        );
    }
    pub fn update_settings(&mut self, settings: SettingsDTO) {
        self.queue
            .write_buffer(&self.settings_buffer, 0, bytemuck::bytes_of(&settings));
    }

    pub fn get_frame(&mut self) -> &mut imgui::Ui {
        self.imgui.get_frame()
    }

    pub fn handle_input<T>(&mut self, event: &winit::event::Event<T>) {
        self.imgui
            .platform
            .handle_event(self.imgui.imgui.io_mut(), &self.window, event);
    }

    pub fn set_enable_gauss(&mut self, enabled: bool) {
        self.gauss_enabled = enabled;
    }

    pub fn want_mouse_input(&self) -> bool {
        self.imgui.imgui.io().want_capture_mouse
    }
    pub fn want_keyboard_input(&self) -> bool {
        self.imgui.imgui.io().want_capture_keyboard
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
