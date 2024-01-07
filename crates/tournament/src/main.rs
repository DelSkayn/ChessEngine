use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// A path to a file containing opening positions as FEN line-by-line.
    #[arg(short, long)]
    opening_book: Option<PathBuf>,
    /// Time limit in minutes.
    #[arg(short, long, default_value_t = 1.0)]
    time: f64,
    /// Time increment in seconds
    #[arg(short, long, default_value_t = 1.0)]
    increment: f64,
    /// The number of games to run
    #[arg(short, long)]
    games: Option<usize>,
    /// The engine binaries to run a tournament over.
    engines: Vec<PathBuf>,
}

fn main() {
    let _args = Args::parse();
}
