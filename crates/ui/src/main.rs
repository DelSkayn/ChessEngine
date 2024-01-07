use crate::game::Chess;
use anyhow::Result;
use clap::Parser;
use common::board::Board;
use ggez::{
    conf::{Backend, WindowMode, WindowSetup},
    ContextBuilder,
};
use player::{MousePlayer, Player, UciPlayer};
use std::{
    env,
    path::{self, PathBuf},
    time::Duration,
};

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
    #[arg(short, long)]
    pause: Option<f32>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let board = if let Some(fen) = args.fen {
        Board::from_fen(fen.as_str())?
    } else {
        Board::start_position()
    };

    let (mut ctx, event_loop) = ContextBuilder::new("chess ui", env!("CARGO_PKG_AUTHORS"))
        .add_resource_path(resource_dir)
        .window_mode(WindowMode::default().resizable(true))
        .window_setup(WindowSetup::default().title("chess"))
        .backend(Backend::Gl)
        .build()?;

    let white = if let Some(white) = args.white {
        Box::new(UciPlayer::new(&white)?) as Box<dyn Player>
    } else {
        Box::new(MousePlayer::new()) as Box<dyn Player>
    };
    let black = if let Some(black) = args.black {
        Box::new(UciPlayer::new(&black)?) as Box<dyn Player>
    } else {
        Box::new(MousePlayer::new()) as Box<dyn Player>
    };

    let mut game = Chess::new(&mut ctx, board, white, black);
    game.set_pause(args.pause);
    if let Some(limit) = args.time {
        game.set_timelimit(Duration::from_secs_f64(limit * 60.0))
    }
    if let Some(increment) = args.increment {
        game.set_increment(Duration::from_secs_f64(increment))
    }

    ggez::event::run(ctx, event_loop, game)
}
