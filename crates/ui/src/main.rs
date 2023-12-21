use clap::Parser;
use common::board::Board;
use ggez::{
    conf::{Backend, WindowMode, WindowSetup},
    ContextBuilder,
};
use player::MousePlayer;
use std::{
    env,
    path::{self, PathBuf},
};

use crate::game::Chess;

mod board;
mod game;
mod player;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Fen string of the start position.
    #[arg(short, long)]
    fen: Option<String>,
    /// Path to the white engine.
    #[arg(short, long)]
    white: Option<PathBuf>,
    /// Path to the black engine.
    #[arg(short, long)]
    black: Option<PathBuf>,
    /// Time increment in minutes.
    #[arg(short, long)]
    time: Option<f64>,
    /// Time increment in seconds
    #[arg(short, long)]
    increment: Option<f64>,
}

fn main() -> Result<(), String> {
    let args = Args::parse();

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let board = if let Some(fen) = args.fen {
        Board::from_fen(fen.as_str()).map_err(|x| x.to_string())?
    } else {
        Board::start_position()
    };

    let (mut ctx, event_loop) = ContextBuilder::new("chess ui", env!("CARGO_PKG_AUTHORS"))
        .add_resource_path(resource_dir)
        .window_mode(WindowMode::default().resizable(true))
        .window_setup(WindowSetup::default().title("chess"))
        .backend(Backend::Gl)
        .build()
        .map_err(|x| x.to_string())?;

    let game = Chess::new(
        &mut ctx,
        board,
        Box::new(MousePlayer::new()),
        Box::new(MousePlayer::new()),
    );

    ggez::event::run(ctx, event_loop, game)
}
