use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(
        short,
        long,
        default_value = "phi_3_5_mini_instruct-genie-w4a16-qualcomm/genie_config.json"
    )]
    pub config: PathBuf,
    #[arg(
        short,
        long,
        default_value = "Explain quantum physics in one sentence."
    )]
    pub prompt: String,
    #[arg(short, long, default_value = "512")]
    pub max_tokens: usize,
    #[arg(short, long)]
    pub verbose: bool,
    #[arg(long)]
    pub ort: bool,
    #[arg(short, long)]
    pub model: Option<PathBuf>,
    #[arg(long, default_value = "43")]
    pub soc_model: String,
}
