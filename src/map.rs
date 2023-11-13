use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::MapDTO;
use rand::Rng;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Cell {
    None = 0,
    Grass = 1,
    Stone = 2,
    Ground = 3,
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        cell as Self
    }
}

impl Cell {
    pub fn color(self) -> u32 {
        match self {
            Self::None => 0x0,
            Self::Grass => 0x71aa34,
            Self::Stone => 0x7d7071,
            Self::Ground => 0xa05b53,
        }
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

    pub fn cube(x: usize, y: usize, z: usize) -> Self {
        let cells = (0..x * y * z).map(|_| Cell::Grass).collect();
        Self { x, y, z, cells }
    }

    pub fn random(x: usize, y: usize, z: usize) -> Self {
        let mut rng = rand::thread_rng();
        let cells = (0..x * y * z)
            .map(|_| match rng.gen::<u32>() % 4 {
                //0 => Cell::None,
                1 | 2 => Cell::Stone,
                //2 => Cell::Stone,
                _ => Cell::Ground,
            })
            .collect();
        Self { x, y, z, cells }
    }

    pub fn with_perlin(x: usize, y: usize, z: usize, perlin: &mut Perlin) -> Self {
        let cells = vec![Cell::None; x * y * z];
        let mut map = Self { x, y, z, cells };
        let mut min_height = usize::MAX;
        for px in 0..x {
            for pz in 0..z {
                let p = Vector3::new(px as f32 / x as f32, 0.0, pz as f32 / z as f32);
                let perlin_value = perlin.turb(p, 4);
                let height = (perlin_value.sin() + 1.0) * 0.5 * (y as f32);
                let height = height as usize;
                for py in 0..height {
                    let cell = if py < height - 1 {
                        Cell::Ground
                    } else if py == height - 1 {
                        if py < min_height {
                            min_height = py;
                        }
                        Cell::Grass
                    } else {
                        Cell::None
                    };
                    *map.at_mut(px, py, pz) = cell;
                }
            }
        }

        for px in 0..x {
            for pz in 0..z {
                let cur = map.at(px, min_height, pz);
                if let Cell::Grass = cur {
                    *map.at_mut(px, min_height, pz) = Cell::Stone;
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
