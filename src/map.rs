use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::MapDTO;
use rand::prelude::SliceRandom;
use rand::Rng;

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Cell {
    None = 0,
    Grass = 1,
    Water = 2,
    Ground = 3,
}

impl From<Cell> for u8 {
    fn from(cell: Cell) -> Self {
        cell as Self
    }
}

impl Cell {
    fn is_water(&self) -> bool {
        if let Self::Water = self {
            return true;
        }

        false
    }
    fn is_air(&self) -> bool {
        if let Self::None = self {
            return true;
        }

        false
    }
    fn is_solid(&self) -> bool {
        match self {
            Self::Grass | Self::Ground => true,
            _ => false,
        }
    }
}

#[derive(Clone)]
pub struct Map {
    x: usize,
    y: usize,
    z: usize,
    cells: Vec<Cell>,
}

impl Map {
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
                1 | 2 => Cell::Water,
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
                    *map.at_mut(px, min_height, pz) = Cell::Water;
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

    pub fn simulate(&mut self, rng: &mut impl RandNalgebra) {
        let mut new_map = Self {
            x: self.x,
            y: self.y,
            z: self.z,
            cells: vec![Cell::None; self.cells.len()],
        };

        // copy static cells
        for x in 0..self.x {
            for y in 0..self.y {
                for z in 0..self.z {
                    let cell = self.at(x, y, z);
                    match cell {
                        Cell::Water => {}
                        _ => *new_map.at_mut(x, y, z) = cell,
                    }
                }
            }
        }

        for x in 0..self.x {
            for y in 0..self.y {
                for z in 0..self.z {
                    let cell = self.at(x, y, z);
                    if let Cell::Water = cell {
                        // this is last cell, discard the water
                        if y == 0 {
                        } else {
                            let below = new_map.at(x, y - 1, z);
                            if below.is_solid() {
                                // this cell has hit ground
                                // check if we can flow in some random direction
                                let mut neighbour_coords = Vec::with_capacity(9);
                                if x != 0 {
                                    neighbour_coords.push((x - 1, y, z));
                                    if z != 0 {
                                        neighbour_coords.push((x - 1, y, z - 1));
                                    }
                                    if z != self.z - 1 {
                                        neighbour_coords.push((x - 1, y, z + 1));
                                    }
                                }
                                if x != self.x - 1 {
                                    neighbour_coords.push((x + 1, y, z));
                                    if z != 0 {
                                        neighbour_coords.push((x + 1, y, z - 1));
                                    }
                                    if z != self.z - 1 {
                                        neighbour_coords.push((x + 1, y, z + 1));
                                    }
                                }
                                if z != 0 {
                                    neighbour_coords.push((x, y, z - 1));
                                }
                                if z != self.z - 1 {
                                    neighbour_coords.push((x, y, z + 1));
                                }

                                let neighbour_cells = neighbour_coords
                                    .iter()
                                    .cloned()
                                    .filter(|(x, y, z)| new_map.at(*x, *y, *z).is_air())
                                    .collect::<Vec<(usize, usize, usize)>>();
                                if let Some(new_coord) = neighbour_cells.choose(rng) {
                                    // have some direction to flow
                                    *new_map.at_mut(new_coord.0, new_coord.1, new_coord.2) =
                                        Cell::Water;
                                } else {
                                    // if we have howhere to flow stay in this cell
                                    *new_map.at_mut(x, y, z) = Cell::Water;
                                }
                            } else if below.is_water() {
                                //
                            } else {
                                // in this case we have free space below us, just fall there
                                *new_map.at_mut(x, y - 1, z) = Cell::Water;
                            }
                        }
                    }
                }
            }
        }

        *self = new_map;
    }
}
