use clap::{command, Parser};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, author)]
pub struct Args {
    /// mels resolution
    #[arg(short, long, default_value_t = 80)]
    pub mels: usize,

    /// Number of times to greet
    #[arg(short, long, default_value_t = String::from("./"))]
    pub out_dir: String,
}
