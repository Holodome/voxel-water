use crate::math::*;
use crate::perlin::Perlin;
use rand::Rng;

#[derive(Clone, Copy)]
enum Cell {
    None,
    Ground,
}

struct Map {
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
}
