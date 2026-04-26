#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum RandomType {
    EmNone = 0,
    EmJianghu = 5, // We need emJianghu
}

pub struct GRandom {
    pub seed: u32,
}

impl GRandom {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    pub fn rand(&mut self) -> u32 {
        self.seed = (self.seed as u64).wrapping_mul(1103515245).wrapping_add(12345) as u32;
        self.seed = (self.seed << 16) | (self.seed >> 16);
        self.seed
    }

    pub fn rand_range(&mut self, mut min: i32, mut max: i32) -> i32 {
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        let num2 = ((self.rand() as f64 / 4294967295.0) * (max as f64 - min as f64 + 1.0)) as u32;
        let mut num3 = num2.wrapping_add(min as u32) as i32;
        if num3 > max {
            num3 = max;
        }
        num3
    }
}

pub struct DotNetRandom {
    inext: usize,
    inextp: usize,
    seed_array: [i32; 56],
}
impl DotNetRandom {
    pub fn new(seed: i32) -> Self {
        let mut r = Self {
            inext: 0,
            inextp: 31,
            seed_array: [0; 56],
        };
        let mut mj = 161803398 - seed.saturating_abs();
        r.seed_array[55] = mj;
        let mut mk = 1;
        for i in 1..55 {
            let ii = (21 * i) % 55;
            r.seed_array[ii] = mk;
            mk = mj - mk;
            if mk < 0 {
                mk += i32::MAX;
            }
            mj = r.seed_array[ii];
        }
        for _ in 1..5 {
            for k in 1..56 {
                let mut val = r.seed_array[k] - r.seed_array[1 + (k + 30) % 55];
                if val < 0 {
                    val += i32::MAX;
                }
                r.seed_array[k] = val;
            }
        }
        r
    }
    pub fn next_double(&mut self) -> f64 {
        self.inext = if self.inext + 1 == 56 {
            1
        } else {
            self.inext + 1
        };
        self.inextp = if self.inextp + 1 == 56 {
            1
        } else {
            self.inextp + 1
        };
        let mut n = self.seed_array[self.inext] - self.seed_array[self.inextp];
        if n == i32::MAX {
            n -= 1;
        }
        if n < 0 {
            n += i32::MAX;
        }
        self.seed_array[self.inext] = n;
        n as f64 * 4.656612875245797e-10
    }
    pub fn next_range(&mut self, mut mi: i32, mut ma: i32) -> i32 {
        if mi > ma {
            std::mem::swap(&mut mi, &mut ma);
        }
        let diff = ma as i64 - mi as i64;
        if diff <= 1 {
            return mi;
        }
        self.inext = if self.inext + 1 == 56 {
            1
        } else {
            self.inext + 1
        };
        self.inextp = if self.inextp + 1 == 56 {
            1
        } else {
            self.inextp + 1
        };
        let mut n = self.seed_array[self.inext] - self.seed_array[self.inextp];
        if n < 0 {
            n += i32::MAX;
        }
        self.seed_array[self.inext] = n;
        ((n as f64 * 4.656612875245797e-10 * diff as f64) as u32).wrapping_add(mi as u32) as i32
    }
    pub fn next_range_strict(&mut self, mut mi: i32, mut ma: i32) -> i32 {
        if mi > ma {
            std::mem::swap(&mut mi, &mut ma);
        }
        ((self.next_double() * (ma as i64 - mi as i64) as f64) as i64 + mi as i64) as i32
    }
    pub fn next_float(&mut self, min: f32, max: f32) -> f32 {
        self.next_double() as f32 * (max - min) + min
    }
    pub fn random_rate(&mut self, rate: f32) -> bool {
        (self.next_double() as f32) < rate
    }
    pub fn advance(&mut self, steps: usize) {
        (0..steps).for_each(|_| {
            self.next_double();
        });
    }
    pub fn box_muller_trap(&mut self) -> f32 {
        loop {
            let (n1, n2) = (
                self.next_double() as f32 * 2. - 1.,
                self.next_double() as f32 * 2. - 1.,
            );
            let n3 = n1 * n1 + n2 * n2;
            if (0.0..1.0).contains(&n3) {
                return n1 * ((-2. * n3.ln()) / n3).sqrt();
            }
        }
    }
}
pub struct GMathUtl {
    sys_random: DotNetRandom,
}
impl GMathUtl {
    pub fn new(s: i32) -> Self {
        Self {
            sys_random: DotNetRandom::new(s),
        }
    }
    #[inline]
    pub fn random_range_int(&mut self, mi: i32, ma: i32, _: RandomType, _: &str) -> i32 {
        self.sys_random.next_range(mi, ma)
    }
}
