use std::ops::{Add, Mul, Index};

#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(transparent)]
pub struct Poly<T> {
    // a0 + a1 * x^0 + ...
    // storing [a0, a1, ...]
    coeffs : Vec<T>,
}

impl<T> Poly<T> {
    pub const fn new(v: Vec<T>) -> Self {
        Poly {coeffs: v}
    }
}

impl<T> Index<usize> for Poly<T> {
    type Output = T;

    fn index(&self, idx: usize) -> &Self::Output {
        &self.coeffs[idx]
    }
}

impl<T: Copy + Clone + Add<T, Output = T>> Add for Poly<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        let (v1, v2) =
            if self.coeffs.len() > rhs.coeffs.len(){
                (self, rhs)
            } else {
                (rhs, self)
            };
        let mut out = v1.coeffs.clone();
        for (i, c) in v2.coeffs.iter().enumerate() {
            out[i] = out[i] + *c;
        }
        Poly {coeffs: out}
    }
}

impl<T: Copy + Clone + Default
          + Mul<T, Output = T>
          + Add<T, Output = T>> Mul for Poly<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let size = self.coeffs.len() + rhs.coeffs.len() - 1;
        let mut out = Vec::with_capacity(size);
        out.resize(size, Default::default());
        for j in 0 .. rhs.coeffs.len() {
            for i in 0 .. self.coeffs.len() {
                out[i + j] = out[i + j] + self.coeffs[i] * rhs.coeffs[j];
            }
        }
        Poly {coeffs: out}
    }
}
