use crate::complex::C64;

/// IEEE Reference Temperature (K)
pub const T0: f64 = 290.0;

#[derive(Debug, Clone)]
pub struct NoiseParams {
    pub f_min: f64,
    pub r_n: f64,
    pub gamma_opt: C64,
}

impl NoiseParams {
    /// Noise figure of an active device given source reflection Γs.
    /// r_n is the noise resistance (ohms), z0 is the reference impedance.
    pub fn nf(&self, gamma_s: C64, z0: f64) -> f64 {
        self.f_min
            + (4.0 * self.r_n * (gamma_s - self.gamma_opt).abs2())
                / (z0 * (1.0 - gamma_s.abs2()) * (1.0 + self.gamma_opt).abs2())
    }
}

/// Available gain of a two-port given source reflection Γs.
/// S-param layout: [S11, S12, S21, S22].
pub fn available_gain(s: &[C64; 4], gamma_s: C64) -> f64 {
    let gamma_o = s[3] + s[1] * s[2] * gamma_s / (1.0 - s[0] * gamma_s);
    let num = s[2].abs2() * (1.0 - gamma_s.abs2());
    let den = (1.0 - s[0] * gamma_s).abs2() * (1.0 - gamma_o.abs2());
    num / den
}

/// Noise figure of a passive two-port at physical temperature `t_phys` (K),
/// given its S-parameters and source reflection Γs.
pub fn passive_nf(s: &[C64; 4], gamma_s: C64, t_phys: f64) -> f64 {
    let ga = available_gain(s, gamma_s);
    1.0 + ((1.0 - ga) / ga) * (t_phys / T0)
}

/// System noise figure of a passive IMN cascaded with an active device.
///
/// Computes F_sys = NF_amp / G_a_IMN using an algebraically simplified
/// form that cancels the (1 - |Γ_s_amp|²) singularity present in both
/// NF_amp (denominator) and G_a (numerator). The only remaining
/// singularity is 1/|S21_IMN|², corresponding to a physically opaque
/// matching network.
///
/// S-param layout: [S11, S12, S21, S22].
pub fn system_nf(noise: &NoiseParams, s_imn: &[C64; 4], gamma_s: C64, z0: f64) -> f64 {
    // Source reflection seen by the amplifier
    let gamma_sa = s_imn[3] + s_imn[1] * s_imn[2] * gamma_s / (1.0 - s_imn[0] * gamma_s);

    // NF_amp × (1 - |Γ_sa|²):  cancels the (1-|Γ_sa|²) denominator in NF
    let nf_x_mismatch = noise.f_min * (1.0 - gamma_sa.abs2())
        + 4.0 * noise.r_n * (gamma_sa - noise.gamma_opt).abs2()
            / (z0 * (1.0 + noise.gamma_opt).abs2());

    // G_a × (1 - |Γ_sa|²):  removes the output mismatch factor from G_a
    //   = |S21|² (1 - |Γ_s|²) / |1 - S11 Γ_s|²
    let ga_x_mismatch =
        s_imn[2].abs2() * (1.0 - gamma_s.abs2()) / (1.0 - s_imn[0] * gamma_s).abs2();

    // F_sys = (NF × (1-|Γ_sa|²)) / (G_a × (1-|Γ_sa|²))
    nf_x_mismatch / ga_x_mismatch
}

/// Convert noise figure (linear) to noise temperature (K).
pub fn noise_temperature(nf: f64) -> f64 {
    T0 * (nf - 1.0)
}
