use std::path::Path;

use anyhow::Result;
use clap::Parser;
use wblna_optim::{
    cli::Cli, cost::OptParams, opt::adam, postprocess::eval_full, target::Target, tline::TLine,
};

/// Generate a linear taper from max to min in log-normalized space
fn initial_x(x: &mut [f64], min_w: f64, max_w: f64) {
    let ln_min = min_w.ln();
    let ln_max = max_w.ln();
    let n = x.len();
    if n == 0 {
        return;
    }
    if n == 1 {
        x[0] = (ln_min + ln_max) * 0.5;
        return;
    }
    for (i, xi) in x.iter_mut().enumerate() {
        *xi = ln_max + (ln_min - ln_max) * i as f64 / (n - 1) as f64;
    }
}

fn main() -> Result<()> {
    // Grab the CLI params and read out the tline data and target
    let cli = Cli::parse();
    let tline = TLine::load(&cli.tline)?;
    let target = Target::load(&cli.target)?;

    // Setup the optimization params and initial guess
    let p = OptParams {
        tline,
        target,
        length: cli.length,
        z0: 50.,
    };
    let mut x = vec![0.; cli.segments];
    initial_x(&mut x, cli.min_w, cli.max_w);

    // Run the optimizer
    let obj = adam(&p, &mut x, cli.min_w, cli.max_w, 1e-2, 5000, 1e-7);

    // Print result noise
    eprintln!("final obj={obj:.6}");

    // Save data
    let widths: Vec<f64> = x.iter().map(|&u| u.exp()).collect();
    let result = eval_full(&p.tline, &widths, p.length, &p.target, p.z0);

    let delta_mm = cli.length * 1e3 / widths.len() as f64;
    let positions_mm: Vec<f64> = (0..widths.len()).map(|i| i as f64 * delta_mm).collect();
    let widths_mm: Vec<f64> = widths.iter().map(|&w| w * 1e3).collect();

    let json = serde_json::json!({
        "params": { "gamma_max_db": -10.0 },  // TODO: wire up from CLI
        "positions_mm": positions_mm,
        "widths_mm": widths_mm,
        "freqs_ghz": result.freqs_ghz,
        "return_loss_db": result.return_loss_db,
        "te_k": result.te_k,
        "te_amp_min_k": result.te_amp_min_k,
        "te_imn_k": result.te_imn_k,
        "mean_te_k": result.mean_te_k,
    });

    let out_path = cli.output.as_deref().unwrap_or(Path::new("results.json"));
    std::fs::write(out_path, serde_json::to_string_pretty(&json)?)?;
    eprintln!("wrote {}", out_path.display());

    Ok(())
}
