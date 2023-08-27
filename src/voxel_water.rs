use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::{CameraDTO, MapDTO};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Camera {
    aspect_ratio: f32,
    vfov: f32,
    znear: f32,
    zfar: f32,
    projection_matrix: Matrix4,
    inverse_projection_matrix: Matrix4,

    pitch: f32,
    yaw: f32,
    position: Vector3,
    view_matrix: Matrix4,
}

impl Camera {
    pub fn new(aspect_ratio: f32, vfov: f32, znear: f32, zfar: f32) -> Self {
        let projection_matrix =
            *nalgebra::Perspective3::new(aspect_ratio, vfov, znear, zfar).as_matrix();
        let inverse_projection_matrix = projection_matrix.try_inverse().unwrap();
        let view_matrix = Matrix4::identity();
        Self {
            aspect_ratio,
            vfov,
            znear,
            zfar,
            pitch: 0.0,
            yaw: 0.0,
            position: Vector3::new(0.0, 0.0, 0.0),
            view_matrix,
            projection_matrix,
            inverse_projection_matrix,
        }
    }

    pub fn rotate(&mut self, yaw_d: f32, pitch_d: f32) {
        self.pitch += pitch_d;
        self.yaw += yaw_d;
        self.update_view_matrix();
    }
    pub fn translate(&mut self, translation: Vector3) {
        self.position += translation;
        self.update_view_matrix();
    }

    fn update_view_matrix(&mut self) {
        let rotation = Matrix4::from_euler_angles(self.yaw, self.pitch, 0.0);
        self.view_matrix = Matrix4::new_translation(&self.position) * rotation;
    }

    pub fn as_dto(&self) -> CameraDTO {
        CameraDTO {
            view_matrix: self.view_matrix,
            projection_matrix: self.projection_matrix,
            inverse_projection_matrix: self.inverse_projection_matrix,
        }
    }
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum Cell {
    None = 0,
    Ground = 1,
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        cell as Self
    }
}

pub struct Map {
    x: usize,
    y: usize,
    z: usize,
    cells: Vec<Cell>,
}

impl Map {
    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn at(&self, x: usize, y: usize, z: usize) -> Cell {
        self.cells[z * (self.x * self.y) + y * self.x + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Cell {
        &mut self.cells[z * (self.x * self.y) + y * self.x + x]
    }

    pub fn random(x: usize, y: usize, z: usize) -> Self {
        let mut rng = rand::thread_rng();
        let cells = (0..x * y * z)
            .map(|_| match rng.gen::<u32>() % 2 {
                0 => Cell::None,
                _ => Cell::Ground,
            })
            .collect();
        Self { x, y, z, cells }
    }

    pub fn with_perlin(x: usize, y: usize, z: usize, perlin: &mut Perlin) -> Self {
        let cells = vec![Cell::None; x * y * z];
        let mut map = Self { x, y, z, cells };
        for px in 0..x {
            for pz in 0..z {
                let p = Vector3::new(px as f32, 0.0, pz as f32);
                let perlin_value = perlin.turb(p, 4);
                let height = (perlin_value.sin() + 1.0) * 0.5 * (y as f32);
                let height = height as usize;
                for py in 0..height {
                    let cell = if py < height {
                        Cell::Ground
                    } else {
                        Cell::None
                    };
                    *map.at_mut(px, py, pz) = cell;
                }
            }
        }
        map
    }

    pub fn as_dto<'a>(&'a self) -> MapDTO<'a> {
        let cells = unsafe {
            std::slice::from_raw_parts(self.cells.as_ptr() as *const u8, self.cells.len())
        };
        MapDTO {
            x: self.x,
            y: self.y,
            z: self.z,
            cells,
        }
    }
}
