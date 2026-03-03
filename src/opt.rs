use crate::cost::OptParams;
use crate::cost::dcost;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

pub fn adam(
    p: &OptParams,
    x: &mut [f64],
    min_w: f64,
    max_w: f64,
    lr: f64,
    max_iter: usize,
    tol: f64,
    mp: &MultiProgress,
    label: &str,
) -> f64 {
    let beta1 = 0.9_f64;
    let beta2 = 0.999_f64;
    let eps = 1e-8_f64;
    let ln_min = min_w.ln();
    let ln_max = max_w.ln();
    let mut m = vec![0.0; x.len()];
    let mut v = vec![0.0; x.len()];
    let mut dx = vec![0.0; x.len()];
    let mut p_shadow = p.clone();
    let mut obj = f64::MAX;

    let pb = mp.add(ProgressBar::new(max_iter as u64));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{prefix:.bold} {spinner:.green} [{bar:40.cyan/blue}] {pos:>4}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_prefix(label.to_string());

    let mut converged = false;

    for iter in 1..=max_iter {
        dx.fill(0.0);
        let prev_obj = obj;
        obj = dcost(p, &mut p_shadow, x, &mut dx, 1.0);
        let t = iter as f64;
        let bc1 = 1.0 - beta1.powf(t);
        let bc2 = 1.0 - beta2.powf(t);
        let mut grad_norm_sq = 0.0;
        for (((xi, mi), vi), &gi) in x
            .iter_mut()
            .zip(m.iter_mut())
            .zip(v.iter_mut())
            .zip(dx.iter())
        {
            *mi = beta1 * *mi + (1.0 - beta1) * gi;
            *vi = beta2 * *vi + (1.0 - beta2) * gi * gi;
            let m_hat = *mi / bc1;
            let v_hat = *vi / bc2;
            *xi = (*xi - lr * m_hat / (v_hat.sqrt() + eps)).clamp(ln_min, ln_max);
            grad_norm_sq += gi * gi;
        }
        let grad_norm = grad_norm_sq.sqrt();

        pb.set_position(iter as u64);
        pb.set_message(format!("{obj:10.6} |g|={grad_norm:8.4}"));

        if (prev_obj - obj).abs() < tol {
            converged = true;
            break;
        }
    }
    if converged {
        pb.finish_with_message(format!("✓ {obj:10.6}  converged"));
    } else {
        pb.finish_with_message(format!("✗ {obj:10.6}  iter_max"));
    }
    obj
}
