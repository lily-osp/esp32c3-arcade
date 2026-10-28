pub struct Rng {
    state: u32,
}

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 0x6A09E667 } else { seed },
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn gen_range(&mut self, min: i32, max: i32) -> i32 {
        if max <= min {
            return min;
        }
        let diff = (max - min) as u32;
        min + (self.next_u32() % diff) as i32
    }

    pub fn gen_bool_ppm(&mut self, ppm: u32) -> bool {
        (self.next_u32() % 1_000_000) < ppm
    }

    pub fn gen_percent(&mut self, percent: u32) -> bool {
        (self.next_u32() % 100) < percent
    }

    pub fn gen_f32(&mut self) -> f32 {
        (self.next_u32() & 0x00FF_FFFF) as f32 / 16777216.0
    }
}
