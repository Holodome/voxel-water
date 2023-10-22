use crate::math::*;

const PERLIN_POINT_COUNT: usize = 256;
const PERLIN_POINT_MASK: usize = PERLIN_POINT_COUNT - 1;

pub struct Perlin {
    ranvec: Vec<Vector3>,
    perm_x: Vec<u32>,
    perm_y: Vec<u32>,
    perm_z: Vec<u32>,
}

impl Perlin {
    fn permute(vec: &mut Vec<u32>, rng: &mut impl RandNalgebra) {
        for i in (1..PERLIN_POINT_COUNT).rev() {
            let target: usize = rng.gen_range(0..i);
            vec.swap(i, target);
        }
    }

    fn generate_perm(rng: &mut impl RandNalgebra) -> Vec<u32> {
        let mut base = (0..PERLIN_POINT_COUNT).map(|i| i as u32).collect();
        Self::permute(&mut base, rng);
        base
    }

    pub fn new(rng: &mut impl RandNalgebra) -> Self {
        let ranvec = (0..PERLIN_POINT_COUNT)
            .map(|_| rng.vector3(-1.0, 1.0).normalize())
            .collect();
        let perm_x = Self::generate_perm(rng);
        let perm_y = Self::generate_perm(rng);
        let perm_z = Self::generate_perm(rng);

        Self {
            ranvec,
            perm_x,
            perm_y,
            perm_z,
        }
    }

    fn interp(&mut self, c: &[[[Vector3; 2]; 2]; 2], u: f32, v: f32, w: f32) -> f32 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let vec = c[i][j][k];
                    let i = i as f32;
                    let j = j as f32;
                    let k = k as f32;
                    let a = (i * uu + (1.0 - i) * (1.0 - uu))
                        * (j * vv + (1.0 - j) * (1.0 - vv))
                        * (k * ww + (1.0 - k) * (1.0 - ww));
                    let weight_v = Vector3::new(u - i, v - j, w - k);
                    accum += a * vec.dot(&weight_v);
                }
            }
        }
        accum
    }

    pub fn noise(&mut self, p: Vector3) -> f32 {
        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;
        let mut c = [[[nalgebra::zero::<Vector3>(); 2]; 2]; 2];
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let target = &mut c[di][dj][dk];
                    let di = di as i32;
                    let dj = dj as i32;
                    let dk = dk as i32;
                    let x = self.perm_x[((i + di) as usize) & PERLIN_POINT_MASK] as usize;
                    let y = self.perm_y[((j + dj) as usize) & PERLIN_POINT_MASK] as usize;
                    let z = self.perm_z[((k + dk) as usize) & PERLIN_POINT_MASK] as usize;
                    *target = self.ranvec[x ^ y ^ z];
                }
            }
        }

        let u = p.x - (i as f32);
        let v = p.y - (j as f32);
        let w = p.z - (k as f32);
        self.interp(&c, u, v, w)
    }

    pub fn turb(&mut self, p: Vector3, octaves: usize) -> f32 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;
        for _ in 0..octaves {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }
}
