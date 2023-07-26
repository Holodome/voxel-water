#[derive(Copy, Clone)]
pub struct Xorshift32 {
    state: u32,
}

impl rand::RngCore for Xorshift32 {
    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    fn next_u64(&mut self) -> u64 {
        rand_core::impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        rand_core::impls::fill_bytes_via_next(self, dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        Ok(self.fill_bytes(dest))
    }
}

#[derive(Clone, Copy)]
pub struct Xorshift32Seed(pub [u8; 4]);

impl Default for Xorshift32Seed {
    fn default() -> Self {
        Self([0; 4])
    }
}

impl AsMut<[u8]> for Xorshift32Seed {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl rand::SeedableRng for Xorshift32 {
    type Seed = Xorshift32Seed;

    fn from_seed(seed: Xorshift32Seed) -> Self {
        Self {
            state: u32::from_le_bytes(seed.0),
        }
    }
}
