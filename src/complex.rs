//! Re-implementing complex numbers because some algorithms in num_complex are slow
//! Plus i'm not sure if Enzyme deal with them properly

use std::fmt;
use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Copy, Clone)]
pub struct C64 {
    r: f64,
    i: f64,
}

impl fmt::Display for C64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.r, self.i) {
            (r, 0.0) => write!(f, "{r}"),
            (0.0, i) => write!(f, "{i}j"),
            (r, i) if i < 0.0 => write!(f, "{r} - {}j", -i),
            (r, i) => write!(f, "{r} + {i}j"),
        }
    }
}

impl C64 {
    pub const ZERO: Self = Self::new(0., 0.);
    pub const ONE: Self = Self::new(1., 0.);
    pub const J: Self = Self::j();

    #[inline]
    pub fn norm(self) -> f64 {
        self.r.hypot(self.i)
    }

    #[inline]
    pub const fn new(r: f64, i: f64) -> Self {
        Self { r, i }
    }

    #[inline]
    pub const fn real(r: f64) -> Self {
        Self { r, i: 0. }
    }

    #[inline]
    pub const fn imag(i: f64) -> Self {
        Self { r: 0., i }
    }

    #[inline]
    pub const fn j() -> Self {
        Self { r: 0., i: 1. }
    }

    #[inline]
    pub fn cosh_sinh(self) -> (Self, Self) {
        let (sin_b, cos_b) = self.i.sin_cos();
        let ea = self.r.exp();
        let em = (-self.r).exp();
        let cosh_a = f64::mul_add(ea, 0.5, em * 0.5);
        let sinh_a = f64::mul_add(ea, 0.5, -(em * 0.5));
        (
            Self::new(cosh_a * cos_b, sinh_a * sin_b),
            Self::new(sinh_a * cos_b, cosh_a * sin_b),
        )
    }

    #[inline]
    pub fn sqrt(self) -> Self {
        let modulus = self.r.hypot(self.i);
        let r = ((modulus + self.r.abs()) * 0.5).sqrt();
        if self.r >= 0.0 {
            Self::new(r, self.i / (r * 2.0))
        } else {
            Self::new(self.i.abs() / (r * 2.0), self.i.signum() * r)
        }
    }

    #[inline]
    pub fn conj(self) -> Self {
        Self::new(self.r, -self.i)
    }

    #[inline]
    pub fn abs2(self) -> f64 {
        f64::mul_add(self.r, self.r, self.i * self.i)
    }
}

impl Add for C64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            i: self.i + rhs.i,
        }
    }
}

impl Sub for C64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r - rhs.r,
            i: self.i - rhs.i,
        }
    }
}

impl Mul for C64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(
            f64::mul_add(self.r, rhs.r, -(self.i * rhs.i)),
            f64::mul_add(self.r, rhs.i, self.i * rhs.r),
        )
    }
}

impl Div for C64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: Self) -> Self {
        let denom = rhs.abs2();
        Self::new(
            f64::mul_add(self.r, rhs.r, self.i * rhs.i) / denom,
            f64::mul_add(self.i, rhs.r, -(self.r * rhs.i)) / denom,
        )
    }
}

impl Mul<f64> for C64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self {
        Self::new(self.r * rhs, self.i * rhs)
    }
}

impl Mul<C64> for f64 {
    type Output = C64;

    #[inline]
    fn mul(self, rhs: C64) -> C64 {
        C64::new(self * rhs.r, self * rhs.i)
    }
}

impl Add<f64> for C64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: f64) -> Self {
        Self::new(self.r + rhs, self.i)
    }
}

impl Add<C64> for f64 {
    type Output = C64;

    #[inline]
    fn add(self, rhs: C64) -> C64 {
        C64::new(self + rhs.r, rhs.i)
    }
}

impl Sub<f64> for C64 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: f64) -> Self {
        Self::new(self.r - rhs, self.i)
    }
}

impl Sub<C64> for f64 {
    type Output = C64;

    #[inline]
    fn sub(self, rhs: C64) -> C64 {
        C64::new(self - rhs.r, -rhs.i)
    }
}

impl Div<f64> for C64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self {
        Self::new(self.r / rhs, self.i / rhs)
    }
}

impl Div<C64> for f64 {
    type Output = C64;

    #[inline]
    fn div(self, rhs: C64) -> C64 {
        let denom = rhs.abs2();
        C64::new(self * rhs.r / denom, -(self * rhs.i) / denom)
    }
}

impl Neg for C64 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.r, -self.i)
    }
}
