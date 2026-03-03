use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Wideband LNA matching network optimizer")]
pub struct Cli {
    /// Path(s) to transmission line fit coefficients (JSON)
    #[arg(short, long, num_args = 1..)]
    pub tline: Vec<PathBuf>,

    /// Path to transistor target data (CSV)
    #[arg(short = 'T', long)]
    pub target: PathBuf,

    /// Number of width segments
    #[arg(short = 'N', long, default_value_t = 250)]
    pub segments: usize,

    /// Maximum return loss constraint (dB, e.g. -10)
    #[arg(short = 'G', long, default_value_t = -10.0)]
    pub gamma_max: f64,

    #[arg(long, default_value_t = 0.08)]
    pub length_min: f64,

    #[arg(long, default_value_t = 0.20)]
    pub length_max: f64,

    #[arg(long, default_value_t = 10)]
    pub length_steps: usize,

    #[arg(long, default_value_t = 0.2e-3)]
    pub min_w: f64,

    #[arg(long, default_value_t = 15e-3)]
    pub max_w: f64,

    /// Write ranked JSON results to this file
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}
