use crate::complex::C64;
use crate::noise::{noise_temperature, system_nf};
use crate::target::Target;
use crate::tline::{TLine, abcd_to_s};
use std::autodiff::*;

/// Mean noise temperature across all frequency points for a candidate design.
fn mean_te(tline: &TLine, widths: &[f64], length: f64, target: &Target, z0: f64) -> f64 {
    let n = target.points.len();
    let delta = length / widths.len() as f64;
    let mut acc = 0.0;

    for pt in target.points.iter() {
        let abcd_imn = tline.cascade(widths, pt.freq, delta);
        let s_imn = abcd_to_s(&abcd_imn, z0);
        acc += noise_temperature(system_nf(&pt.noise, &s_imn, C64::ZERO, z0));
    }

    acc / n as f64
}

#[derive(Clone)]
pub struct OptParams {
    pub tline: TLine,
    pub target: Target,
    pub length: f64,
    pub z0: f64,
}

#[autodiff_reverse(dcost, Duplicated, Duplicated, Active)]
pub fn cost(p: &OptParams, ln_widths: &[f64]) -> f64 {
    let widths: Vec<f64> = ln_widths.iter().map(|&u| u.exp()).collect();
    mean_te(&p.tline, &widths, p.length, &p.target, p.z0)
}
