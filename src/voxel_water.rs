use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::{CameraDTO, MapDTO};
use rand::Rng;

#[derive(Debug, Clone)]
pub struct CameraParams {
    pub look_from: Point3,
    pub look_at: Point3,
    pub up: Vector3,
    pub aspect_ratio: f32,
    pub vfov: f32,
    pub focus_dist: f32,
}

#[derive(Debug, Clone)]
pub struct Camera {
    orig: Point3,
    x: Vector3,
    y: Vector3,
    z: Vector3,
    vertical: Vector3,
    horizontal: Vector3,
    lower_left_corner: Point3,
}

impl Camera {
    pub fn new(params: CameraParams) -> Self {
        let z = (params.look_from - params.look_at).normalize();
        let x = params.up.cross(&z).normalize();
        let y = z.cross(&x).normalize();
        let viewport_width = (params.vfov * 0.5).tan() * 2.0;
        let viewport_height = viewport_width;
        let horizontal = x * viewport_width;
        let vertical = y * viewport_height;
        let lower_left_corner =
            params.look_from - (horizontal * 0.5) - (vertical * 0.5) - (z * params.focus_dist);
        Self {
            orig: params.look_from,
            x,
            y,
            z,
            vertical,
            horizontal,
            lower_left_corner,
        }
    }

    pub fn to_dto(&self) -> CameraDTO {
        CameraDTO {
            at: self.orig,
            lower_left: self.lower_left_corner,
            horizontal: self.horizontal,
            vertical: self.vertical,
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

    pub fn to_dto<'a>(&'a self) -> MapDTO<'a> {
        let cells = unsafe {
            std::slice::from_raw_parts(self.cells.as_ptr() as *const u8, self.cells.len())
        };
        MapDTO {
            x: self.x,
            y: self.y,
            z: self.z,
            cells: cells,
        }
    }
}
