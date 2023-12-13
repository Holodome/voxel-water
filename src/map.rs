use crate::math::*;
use crate::perlin::Perlin;
use crate::renderer::MapDTO;
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

pub struct WaterSim {
    x: usize,
    y: usize,
    z: usize,
    max_mass: f32,
    max_compress: f32,
    min_mass: f32,
    min_flow: f32,
    max_speed: f32,
    mass: Vec<f32>,
    new_mass: Vec<f32>,
    cells: Vec<Cell>,

    water_height: usize,
}

impl WaterSim {
    pub fn new(map: Map) -> Self {
        let max_mass = 1.0;
        let max_compress = 0.02;
        let min_mass = 0.0001;
        let max_speed = 1.0;
        let min_flow = 0.01;
        let x = map.x + 2;
        let y = map.y + 2;
        let z = map.z + 2;
        let mut new_cells = vec![Cell::None; x * y * z];
        let mut mass = vec![0.0; x * y * z];
        let new_mass = vec![0.0; x * y * z];
        let mut water_height = 0;
        for xi in 0..map.x {
            for yi in 0..map.y {
                for zi in 0..map.z {
                    let c = map.at(xi, yi, zi);
                    new_cells[(zi + 1) * (x * y) + (yi + 1) * x + (xi + 1)] = c;
                    if c.is_water() {
                        mass[(zi + 1) * (x * y) + (yi + 1) * x + (xi + 1)] = max_mass;
                        water_height = yi + 1;
                    }
                }
            }
        }

        let cells = new_cells;
        Self {
            x,
            y,
            z,

            max_mass,
            max_compress,
            min_mass,
            min_flow,
            max_speed,
            mass,
            new_mass,
            cells,
            water_height,
        }
    }

    pub fn x(&self) -> usize {
        self.x
    }
    pub fn y(&self) -> usize {
        self.y
    }
    pub fn z(&self) -> usize {
        self.z
    }

    pub fn at(&self, x: usize, y: usize, z: usize) -> Cell {
        self.cells[z * (self.x * self.y) + y * self.x + x]
    }

    pub fn at_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Cell {
        &mut self.cells[z * (self.x * self.y) + y * self.x + x]
    }

    pub fn set_mass(&mut self, x: usize, y: usize, z: usize) {
        let i = self.index(x, y, z);
        self.mass[i] = self.max_mass;
    }

    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        z * (self.x * self.y) + y * self.x + x
    }

    fn get_stable_state_b(&self, mass: f32) -> f32 {
        if mass <= 1.0 {
            return 1.0;
        }

        if mass < 2.0 * self.max_mass + self.max_compress {
            return (self.max_mass * self.max_mass + mass * self.max_compress)
                / (self.max_mass + self.max_compress);
        }

        (mass + self.max_compress) * 0.5
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

    pub fn simulate(&mut self) -> bool {
        for x in 1..self.x - 1 {
            for z in 1..self.z - 1 {
                for y in 1..self.y - 1 {
                    if self.cells[self.index(x, y, z)].is_solid() {
                        continue;
                    }

                    let mut remaining_mass = self.mass[self.index(x, y, z)];
                    if remaining_mass <= 0.0 {
                        continue;
                    }

                    if !self.cells[self.index(x, y - 1, z)].is_solid() {
                        let mut flow = self.get_stable_state_b(
                            remaining_mass + self.mass[self.index(x, y - 1, z)],
                        ) - self.mass[self.index(x, y - 1, z)];
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        let flow = flow.clamp(0.0, self.max_speed.min(remaining_mass));

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x, y - 1, z);
                        self.new_mass[i] += flow;
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= 0.0 {
                        continue;
                    }

                    // left
                    if x - 1 != 0 && !self.cells[self.index(x - 1, y, z)].is_solid() {
                        let mut flow = (self.mass[self.index(x, y, z)]
                            - self.mass[self.index(x - 1, y, z)])
                            / 4.0;
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        flow = flow.clamp(0.0, remaining_mass);

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x - 1, y, z);
                        self.new_mass[i] += flow;
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= 0.0 {
                        continue;
                    }
                    // right
                    if x + 1 != self.x && !self.cells[self.index(x + 1, y, z)].is_solid() {
                        let mut flow = (self.mass[self.index(x, y, z)]
                            - self.mass[self.index(x + 1, y, z)])
                            / 4.0;
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        flow = flow.clamp(0.0, remaining_mass);

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x + 1, y, z);
                        self.new_mass[i] += flow;
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= 0.0 {
                        continue;
                    }
                    // up
                    if z - 1 != 0 && !self.cells[self.index(x, y, z - 1)].is_solid() {
                        let mut flow = (self.mass[self.index(x, y, z)]
                            - self.mass[self.index(x, y, z - 1)])
                            / 4.0;
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        flow = flow.clamp(0.0, remaining_mass);

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x, y, z - 1);
                        self.new_mass[i] += flow;
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= 0.0 {
                        continue;
                    }
                    // down
                    if z + 1 != self.z && !self.cells[self.index(x, y, z + 1)].is_solid() {
                        let mut flow = (self.mass[self.index(x, y, z)]
                            - self.mass[self.index(x, y, z + 1)])
                            / 4.0;
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        flow = flow.clamp(0.0, remaining_mass);

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x, y, z + 1);
                        self.new_mass[i] += flow;
                        remaining_mass -= flow;
                    }

                    if remaining_mass <= 0.0 {
                        continue;
                    }

                    // vertical up
                    if !self.cells[self.index(x, y + 1, z)].is_solid() {
                        let mut flow = remaining_mass
                            - self.get_stable_state_b(
                                remaining_mass + self.mass[self.index(x, y + 1, z)],
                            );
                        if flow > self.min_flow {
                            flow *= 0.5;
                        }
                        let flow = flow.clamp(0.0, self.max_speed.min(remaining_mass));

                        let i = self.index(x, y, z);
                        self.new_mass[i] -= flow;
                        let i = self.index(x, y + 1, z);
                        self.new_mass[i] += flow;
                        // remaining_mass -= flow;
                    }
                }
            }
        }

        let mut changed = false;
        self.mass = self.new_mass.clone();
        for x in 1..self.x - 1 {
            for z in 1..self.z - 1 {
                for y in 1..self.y - 1 {
                    let i = self.index(x, y, z);
                    if self.cells[i].is_solid() {
                        continue;
                    }

                    if self.mass[i] > self.min_mass {
                        changed = true;
                        self.cells[i] = Cell::Water;
                    } else {
                        changed = true;
                        self.cells[i] = Cell::None;
                    }
                }
            }
        }

        for x in 1..self.x - 1 {
            for z in 1..self.z - 1 {
                let i = self.index(x, self.water_height, z);
                if self.cells[i].is_air() {
                    self.cells[i] = Cell::Water;
                    self.mass[i] = self.max_mass;
                }
            }
        }

        for x in 0..self.x {
            for z in 0..self.z {
                for y in 0..self.y {
                    let i = self.index(x, y, z);
                    if x == 0
                        || y == 0
                        || z == 0
                        || x == self.x - 1
                        || y == self.y - 1
                        || z == self.z - 1
                    {
                        self.mass[i] = 0.0;
                    }
                }
            }
        }

        changed
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
}
