use crate::{complex::C64, noise::NoiseParams};
use anyhow::Result;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
struct Row {
    freq: f64,
    rn: f64,
    nfmin: f64,
    sopt_r: f64,
    sopt_i: f64,
    s11_r: f64,
    s11_i: f64,
    s12_r: f64,
    s12_i: f64,
    s21_r: f64,
    s21_i: f64,
    s22_r: f64,
    s22_i: f64,
}

/// Transistor data at a single frequency point.
#[derive(Debug, Clone)]
pub struct TargetPoint {
    pub freq: f64,
    pub noise: NoiseParams,
    /// S-parameters stored row-major: [S11, S12, S21, S22].
    pub s: [C64; 4],
}

/// Frequency-dependent transistor model loaded from CSV.
#[derive(Debug, Clone)]
pub struct Target {
    pub points: Vec<TargetPoint>,
}

impl Target {
    pub fn load(path: &Path) -> Result<Self> {
        let mut rdr = csv::Reader::from_path(path)?;
        let mut points = Vec::new();
        for result in rdr.deserialize() {
            let r: Row = result?;
            points.push(TargetPoint {
                freq: r.freq,
                noise: NoiseParams {
                    f_min: 10.0_f64.powf(r.nfmin / 10.0), // CSV is in dB
                    r_n: r.rn,
                    gamma_opt: C64::new(r.sopt_r, r.sopt_i),
                },
                s: [
                    C64::new(r.s11_r, r.s11_i),
                    C64::new(r.s12_r, r.s12_i),
                    C64::new(r.s21_r, r.s21_i),
                    C64::new(r.s22_r, r.s22_i),
                ],
            });
        }
        Ok(Target { points })
    }
}
