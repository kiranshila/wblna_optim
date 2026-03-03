use crate::{
    complex::C64,
    noise::{noise_temperature, passive_nf, system_nf},
    target::Target,
    tline::{TLine, abcd_to_s},
};

pub struct Results {
    pub mean_te_k: f64,
    pub freqs_ghz: Vec<f64>,
    pub te_k: Vec<f64>,
    pub te_amp_min_k: Vec<f64>,
    pub te_imn_k: Vec<f64>,
    pub return_loss_db: Vec<f64>,
}

pub fn eval_full(tline: &TLine, widths: &[f64], length: f64, target: &Target, z0: f64) -> Results {
    let delta = length / widths.len() as f64;
    let mut freqs_ghz = Vec::with_capacity(target.points.len());
    let mut te_k = Vec::with_capacity(target.points.len());
    let mut te_amp_min_k = Vec::with_capacity(target.points.len());
    let mut te_imn_k = Vec::with_capacity(target.points.len());
    let mut return_loss_db = Vec::with_capacity(target.points.len());

    for pt in target.points.iter() {
        let abcd_imn = tline.cascade(widths, pt.freq, delta);
        let s_imn = abcd_to_s(&abcd_imn, z0);

        // system noise temperature
        let te = noise_temperature(system_nf(&pt.noise, &s_imn, C64::ZERO, z0));

        // amplifier minimum noise temperature (T_min)
        let te_min = noise_temperature(pt.noise.f_min);

        // IMN noise temperature alone (passive_nf with Γs=0)
        let te_imn = noise_temperature(passive_nf(&s_imn, C64::ZERO, 290.0));

        // return loss: 20 log10 |S11|
        let rl_db = 20.0 * s_imn[0].abs2().sqrt().log10();

        freqs_ghz.push(pt.freq / 1e9);
        te_k.push(te);
        te_amp_min_k.push(te_min);
        te_imn_k.push(te_imn);
        return_loss_db.push(rl_db);
    }

    let mean_te_k = te_k.iter().sum::<f64>() / te_k.len() as f64;

    Results {
        mean_te_k,
        freqs_ghz,
        te_k,
        te_amp_min_k,
        te_imn_k,
        return_loss_db,
    }
}
