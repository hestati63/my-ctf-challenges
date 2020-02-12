use std::ops::{Add, Mul};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct GF256 {
    v : u8,
}

impl GF256 {
    pub const fn new(v: u8) -> Self {
        GF256 {v : v}
    }

    pub fn is_zero(&self) -> bool {
        self.v == 0
    }

    pub fn pow(x: GF256, times: usize) -> Self {
        if times == 0 {
            GF256::new(1)
        } else if times == 1 {
            x
        } else if times % 2 == 1 {
            let v = GF256::pow(x, times / 2);
            v * v * x
        } else {
            let v = GF256::pow(x, times / 2);
            v * v
        }
    }

    pub fn value(&self) -> u8 {
        self.v
    }
}

impl Default for GF256 {
    fn default() -> GF256 {
        GF256 {v : 0}
    }
}

impl Add for GF256 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        GF256 { v : self.v ^ rhs.v }
    }
}

impl Mul for GF256 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        let mut p: u16 = 0;
        let mut x = self.v as u16;
        let mut y = rhs.v as u16;
        while y > 0 {
            if y & 1 == 1 {
                p ^= x;
            }
            y = y >> 1;
            x = x << 1;
            if x >= 256 {
                x = x ^ 0x11d;
            }
        }
        GF256 { v: p as u8 }
    }
}
