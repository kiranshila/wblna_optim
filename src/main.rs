use anyhow::Result;
use clap::Parser;
use indicatif::MultiProgress;
use itertools::iproduct;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use wblna_optim::{
    cli::Cli,
    cost::OptParams,
    opt::adam,
    postprocess::{Results, eval_full},
    target::Target,
    tline::TLine,
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

struct SweepResult {
    tline_path: PathBuf,
    length: f64,
    obj: f64,
    widths: Vec<f64>,
    result: Results,
}

fn solve_one(
    tline_path: &PathBuf,
    tline: &TLine,
    target: &Target,
    length: f64,
    segments: usize,
    min_w: f64,
    max_w: f64,
    mp: &MultiProgress,
) -> SweepResult {
    let p = OptParams {
        tline: tline.clone(),
        target: target.clone(),
        length,
        z0: 50.,
    };
    let mut x = vec![0.; segments];
    initial_x(&mut x, min_w, max_w);

    let label = format!(
        "{:<25} L={:>4.0}mm",
        tline_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("?"),
        length * 1e3,
    );
    let obj = adam(&p, &mut x, min_w, max_w, 1e-2, 5000, 1e-7, mp, &label);

    let widths: Vec<f64> = x.iter().map(|&u| u.exp()).collect();
    let result = eval_full(&p.tline, &widths, length, &p.target, p.z0);
    SweepResult {
        tline_path: tline_path.clone(), // path stored independently
        length,
        obj,
        widths,
        result,
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let target = Target::load(&cli.target)?;

    // Build the grid
    let tlines: Vec<TLine> = cli
        .tline
        .iter()
        .map(|p| TLine::load(p))
        .collect::<Result<_>>()?;
    let lengths: Vec<f64> = (0..cli.length_steps)
        .map(|i| {
            cli.length_min
                + (cli.length_max - cli.length_min) * i as f64 / (cli.length_steps - 1) as f64
        })
        .collect();

    // Progress bar
    let mp = MultiProgress::new();

    // Parallel sweep
    let combos: Vec<(&PathBuf, &TLine, f64)> =
        iproduct!(cli.tline.iter().zip(tlines.iter()), &lengths)
            .map(|((path, tline), &l)| (path, tline, l))
            .collect();

    let mut results: Vec<SweepResult> = combos
        .par_iter()
        .map(|(path, tline, length)| {
            solve_one(
                path,
                tline,
                &target,
                *length,
                cli.segments,
                cli.min_w,
                cli.max_w,
                &mp,
            )
        })
        .collect();

    results.sort_by(|a, b| a.obj.partial_cmp(&b.obj).unwrap());

    // Serialize all results ranked by objective
    let json: Vec<_> = results
        .iter()
        .map(|r| {
            let delta_mm = r.length * 1e3 / r.widths.len() as f64;
            let positions_mm: Vec<f64> = (0..r.widths.len()).map(|i| i as f64 * delta_mm).collect();
            serde_json::json!({
                "tline": r.tline_path,
                "length_m": r.length,
                "obj": r.obj,
                "params": { "gamma_max_db": cli.gamma_max },
                "positions_mm": positions_mm,
                "widths_mm": r.widths.iter().map(|&w| w * 1e3).collect::<Vec<_>>(),
                "freqs_ghz": r.result.freqs_ghz,
                "return_loss_db": r.result.return_loss_db,
                "te_k": r.result.te_k,
                "te_amp_min_k": r.result.te_amp_min_k,
                "te_imn_k": r.result.te_imn_k,
                "mean_te_k": r.result.mean_te_k,
            })
        })
        .collect();

    let out_path = cli.output.as_deref().unwrap_or(Path::new("results.json"));
    std::fs::write(out_path, serde_json::to_string_pretty(&json)?)?;
    eprintln!("wrote {} results to {}", results.len(), out_path.display());
    Ok(())
}
