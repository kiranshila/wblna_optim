use crate::complex::C64;
use anyhow::Result;
use serde::Deserialize;
use std::f64::consts::PI;
use std::path::Path;

const MU_0: f64 = 4e-7 * PI;
const MAX_CHEBY: usize = 32;

/// Chebyshev polynomial fit: f(w) = sum_k c_k T_k(u).
#[derive(Debug, Clone, Deserialize)]
pub struct ChebyshevFit {
    pub ln_w_min: f64,
    pub ln_w_max: f64,
    pub coeffs: Vec<f64>,
}

impl ChebyshevFit {
    /// Evaluate at strip width `w` (meters).
    pub fn eval(&self, w: f64) -> f64 {
        let u = 2.0 * (w.ln() - self.ln_w_min) / (self.ln_w_max - self.ln_w_min) - 1.0;
        let mut t_prev = 1.0;
        let mut t_curr = u;
        let mut result = self.coeffs[0];
        if self.coeffs.len() > 1 {
            result += self.coeffs[1] * u;
        }
        for &c in &self.coeffs[2..] {
            let t_next = 2.0 * u * t_curr - t_prev;
            result += c * t_next;
            t_prev = t_curr;
            t_curr = t_next;
        }
        result
    }
}

/// Conductor material properties for surface resistance calculation.
#[derive(Debug, Clone, Deserialize)]
pub struct ConductorMaterial {
    pub sigma: f64,
    pub mu_r: f64,
}

impl ConductorMaterial {
    /// Surface resistance Rs(f) = sqrt(pi * f * mu_0 * mu_r / sigma).
    pub fn rs(&self, freq: f64) -> f64 {
        (PI * freq * MU_0 * self.mu_r / self.sigma).sqrt()
    }
}

/// A transmission line model built from Chebyshev curve fits.
///
/// Loaded from `fit_coeffs.json` produced by `xsec --fit`. Provides fast
/// RLGC evaluation at any (width, frequency) without running the field solver.
#[derive(Debug, Clone, Deserialize)]
pub struct TLine {
    #[serde(rename = "C")]
    c_fit: ChebyshevFit,
    #[serde(rename = "L")]
    l_fit: ChebyshevFit,
    strip_factor: ChebyshevFit,
    wall_factor: ChebyshevFit,
    strip_material: ConductorMaterial,
    enclosure_material: ConductorMaterial,
    dc_deps: Option<ChebyshevFit>,
    tan_delta: Option<f64>,
    eps_r: Option<f64>,
}

impl TLine {
    /// Load from a fit_coeffs.json file.
    pub fn load(path: &Path) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let tline: Self = serde_json::from_reader(file)?;
        assert!(tline.c_fit.coeffs.len() <= MAX_CHEBY);
        assert!(tline.l_fit.coeffs.len() <= MAX_CHEBY);
        assert!(tline.strip_factor.coeffs.len() <= MAX_CHEBY);
        assert!(tline.wall_factor.coeffs.len() <= MAX_CHEBY);
        Ok(tline)
    }

    /// Width range (meters) over which the fits are valid.
    pub fn width_range(&self) -> (f64, f64) {
        (self.c_fit.ln_w_min.exp(), self.c_fit.ln_w_max.exp())
    }

    /// Per-unit-length capacitance C (F/m) at strip width `w` (meters).
    fn c(&self, w: f64) -> f64 {
        self.c_fit.eval(w)
    }

    /// Per-unit-length inductance L (H/m) at strip width `w` (meters).
    fn l(&self, w: f64) -> f64 {
        self.l_fit.eval(w)
    }

    /// Per-unit-length resistance R (Ohm/m) at strip width `w` and frequency `freq` (Hz).
    fn r(&self, w: f64, freq: f64) -> f64 {
        self.strip_material.rs(freq) * self.strip_factor.eval(w)
            + self.enclosure_material.rs(freq) * self.wall_factor.eval(w)
    }

    /// Per-unit-length conductance G (S/m) at strip width `w` and frequency `freq` (Hz).
    fn g(&self, w: f64, freq: f64) -> f64 {
        match (&self.dc_deps, self.tan_delta, self.eps_r) {
            (Some(dc), Some(td), Some(er)) => 2.0 * PI * freq * td * er * dc.eval(w),
            _ => 0.0,
        }
    }

    /// Characteristic impedance and propagation constant at strip width `w` (m) and frequency `freq` (Hz)
    fn prop_consts(&self, w: f64, freq: f64) -> (C64, C64) {
        let r = self.r(w, freq);
        let l = self.l(w);
        let g = self.g(w, freq);
        let c = self.c(w);

        let omega = 2.0 * PI * freq;

        let z_ser = C64::new(r, omega * l);
        let y_shu = C64::new(g, omega * c);

        // Careful math here to avoid multiple calls to hypot
        let zc = (z_ser / y_shu).sqrt();
        let gamma = zc * y_shu;
        (zc, gamma)
    }

    /// Lossless characteristic impedance Z0 = sqrt(L/C) at strip width `w` (m).
    /// Frequency-independent; used for the log-impedance parameterization.
    pub fn z0_lossless(&self, w: f64) -> f64 {
        (self.l(w) / self.c(w)).sqrt()
    }

    /// Find strip width (m) that produces lossless impedance `z0_target`.
    /// Z0 is monotonically decreasing with width, so bisection is robust.
    pub fn width_from_z0(&self, z0_target: f64) -> f64 {
        let (mut lo, mut hi) = self.width_range();
        // Z0(lo) is the max impedance, Z0(hi) is the min impedance.
        for _ in 0..40 {
            let mid = (lo + hi) * 0.5;
            if self.z0_lossless(mid) > z0_target {
                lo = mid; // Z0 too high → need wider strip
            } else {
                hi = mid; // Z0 too low → need narrower strip
            }
        }
        (lo + hi) * 0.5
    }

    pub fn abcd(&self, w: f64, freq: f64, l: f64) -> [C64; 4] {
        let (zc, gamma) = self.prop_consts(w, freq);
        let (ch, sh) = (gamma * l).cosh_sinh();
        [ch, zc * sh, sh / zc, ch]
    }

    fn eval_four(&self, w: f64) -> (f64, f64, f64, f64) {
        let n = self.c_fit.coeffs.len();
        let c0 = &self.c_fit.coeffs;
        let l0 = &self.l_fit.coeffs;
        let sf = &self.strip_factor.coeffs;
        let wf = &self.wall_factor.coeffs;

        let u = 2.0 * (w.ln() - self.c_fit.ln_w_min) / (self.c_fit.ln_w_max - self.c_fit.ln_w_min)
            - 1.0;

        let mut t_prev = 1.0f64;
        let mut t_curr = u;
        let mut rc = c0[0] + c0[1] * u;
        let mut rl = l0[0] + l0[1] * u;
        let mut rs = sf[0] + sf[1] * u;
        let mut rw = wf[0] + wf[1] * u;
        for k in 2..n {
            let t_next = 2.0 * u * t_curr - t_prev;
            rc += c0[k] * t_next;
            rl += l0[k] * t_next;
            rs += sf[k] * t_next;
            rw += wf[k] * t_next;
            t_prev = t_curr;
            t_curr = t_next;
        }
        (rc, rl, rs, rw)
    }

    pub fn cascade(&self, widths: &[f64], freq: f64, delta: f64) -> [C64; 4] {
        let rs_strip = self.strip_material.rs(freq);
        let rs_wall = self.enclosure_material.rs(freq);
        let omega = 2.0 * PI * freq;
        let g_factor = match (self.tan_delta, self.eps_r, self.dc_deps.is_some()) {
            (Some(td), Some(er), true) => Some(omega * td * er),
            _ => None,
        };

        let abcd_for = |w: f64| {
            let (c, l, sf, wf) = self.eval_four(w);
            let r = rs_strip * sf + rs_wall * wf;
            let g = g_factor
                .zip(self.dc_deps.as_ref())
                .map_or(0.0, |(gf, dc)| gf * dc.eval(w));

            let z_ser = C64::new(r, omega * l);
            let y_shu = C64::new(g, omega * c);
            let zc = (z_ser / y_shu).sqrt();
            let gamma = (z_ser * y_shu).sqrt();
            let (ch, sh) = (gamma * delta).cosh_sinh();
            [ch, zc * sh, sh / zc, ch]
        };

        let mut it = widths.iter();
        let first = abcd_for(*it.next().expect("widths must be non-empty"));
        it.fold(first, |acc, &w| mul(&acc, &abcd_for(w)))
    }
}

#[inline]
pub fn mul(a: &[C64; 4], b: &[C64; 4]) -> [C64; 4] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
    ]
}

/// Convert ABCD parameters to S-parameters.
/// S-param layout: [S11, S12, S21, S22].
pub fn abcd_to_s(abcd: &[C64; 4], z0: f64) -> [C64; 4] {
    let [a, b, c, d] = *abcd;
    let b_z = b / z0;
    let c_z = c * z0;
    let denom = a + b_z + c_z + d;
    [
        (a + b_z - c_z - d) / denom,   // S11
        2.0 / denom,                   // S12
        2.0 * (a * d - b * c) / denom, // S21
        (-a + b_z - c_z + d) / denom,  // S22
    ]
}

/// Convert S-parameters to ABCD parameters.
pub fn s_to_abcd(s: &[C64; 4], z0: f64) -> [C64; 4] {
    let [s11, s12, s21, s22] = *s;
    let two_s21 = 2.0 * s21;
    [
        ((1.0 + s11) * (1.0 - s22) + s12 * s21) / two_s21, // A
        ((1.0 + s11) * (1.0 + s22) - s12 * s21) * z0 / two_s21, // B
        ((1.0 - s11) * (1.0 - s22) - s12 * s21) / (two_s21 * z0), // C
        ((1.0 - s11) * (1.0 + s22) + s12 * s21) / two_s21, // D
    ]
}

/// Input reflection coefficient: Γ_in = S11 + S12*S21*Γl / (1 - S22*Γl)
pub fn gamma_in(s: &[C64; 4], gamma_l: C64) -> C64 {
    s[0] + s[1] * s[2] * gamma_l / (1.0 - s[3] * gamma_l)
}

/// Output reflection coefficient: Γ_out = S22 + S12*S21*Γs / (1 - S11*Γs)
pub fn gamma_out(s: &[C64; 4], gamma_s: C64) -> C64 {
    s[3] + s[1] * s[2] * gamma_s / (1.0 - s[0] * gamma_s)
}

/// Mismatch factor between two reflection coefficients.
pub fn mismatch(gamma_b: C64, gamma_a: C64) -> C64 {
    -((gamma_b - 1.0) * (gamma_a - gamma_b.conj()))
        / ((gamma_a * gamma_b - 1.0) * (gamma_b.conj() - 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(re: f64, im: f64) -> C64 {
        C64::new(re, im)
    }

    fn assert_c64_approx(a: C64, b: C64, tol: f64) {
        assert!((a - b).norm() < tol, "expected {b}, got {a}");
    }

    fn assert_abcd_approx(a: &[C64; 4], b: &[C64; 4], tol: f64) {
        for (x, y) in a.iter().zip(b.iter()) {
            assert_c64_approx(*x, *y, tol);
        }
    }

    fn make_reciprocal(a: C64, b: C64, c: C64) -> [C64; 4] {
        let d = (C64::new(1.0, 0.0) + b * c) / a;
        [a, b, c, d]
    }

    #[test]
    fn test_abcd_s_roundtrip_reciprocal() {
        let z0 = 50.0;
        let abcd = make_reciprocal(c(1.0, 0.0), c(0.0, 100.0), c(0.0, 0.0));
        let abcd2 = s_to_abcd(&abcd_to_s(&abcd, z0), z0);
        assert_abcd_approx(&abcd, &abcd2, 1e-10);
    }

    #[test]
    fn test_abcd_s_roundtrip_reciprocal_complex() {
        let z0 = 50.0;
        let abcd = make_reciprocal(c(1.2, 0.3), c(10.0, 5.0), c(0.001, 0.002));
        let abcd2 = s_to_abcd(&abcd_to_s(&abcd, z0), z0);
        assert_abcd_approx(&abcd, &abcd2, 1e-10);
    }

    // For a reciprocal network, S12 must equal S21.
    #[test]
    fn test_reciprocal_s12_equals_s21() {
        let z0 = 50.0;
        let abcd = [c(1.0, 0.0), c(0.0, 100.0), c(0.0, 0.0), c(1.0, 0.0)];
        let [_, s12, s21, _] = abcd_to_s(&abcd, z0);
        assert_c64_approx(s12, s21, 1e-10);
    }

    // Known values: series resistor R in series arm.
    // ABCD = [[1, R], [0, 1]], z0 = 50
    // S11 = R / (R + 2*z0), S21 = 2*z0 / (R + 2*z0)
    #[test]
    fn test_series_resistor_known_values() {
        let z0 = 50.0;
        let r = 50.0;
        let abcd = [c(1.0, 0.0), c(r, 0.0), c(0.0, 0.0), c(1.0, 0.0)];
        let [s11, s12, s21, s22] = abcd_to_s(&abcd, z0);
        let denom = r + 2.0 * z0; // 150
        assert_c64_approx(s11, c(r / denom, 0.0), 1e-10); // 1/3
        assert_c64_approx(s21, c(2.0 * z0 / denom, 0.0), 1e-10); // 2/3
        assert_c64_approx(s12, s21, 1e-10); // reciprocal
        assert_c64_approx(s22, s11, 1e-10); // symmetric
    }
}
