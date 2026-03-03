use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Wideband LNA matching network optimizer")]
pub struct Cli {
    /// Path to transmission line fit coefficients (JSON)
    #[arg(short, long)]
    pub tline: PathBuf,

    /// Path to transistor target data (CSV)
    #[arg(short = 'T', long)]
    pub target: PathBuf,

    /// Number of width segments
    #[arg(short = 'N', long, default_value_t = 50)]
    pub segments: usize,

    /// Maximum return loss constraint (dB, e.g. -10)
    #[arg(short = 'G', long, default_value_t = -10.0)]
    pub gamma_max: f64,

    /// Total line length (m)
    #[arg(short = 'L', long, default_value_t = 0.14)]
    pub length: f64,

    #[arg(long, default_value_t = 0.2e-3)]
    pub min_w: f64,

    #[arg(long, default_value_t = 15e-3)]
    pub max_w: f64,

    /// Write JSON results to this file (runs evaluate on best solution)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}
