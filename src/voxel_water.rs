use crate::math::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DisplayVertex {
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

pub const DISPLAY_VERTICES: &[DisplayVertex] = &[
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
pub struct SphereUniform {
    pub point: Point3,
    pub radius: f32,
    pub color: Vector3,
    pub padding: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WorldUniform {
    pub camera_at: Point3,
    pub _pad0: u32,
    pub camera_lower_left: Point3,
    pub _pad1: u32,
    pub camera_horizontal: Vector3,
    pub _pad2: u32,
    pub camera_vertical: Vector3,
    pub _pad3: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Triangle {
    p: [Point3; 3],
    n: Vector3,
}
