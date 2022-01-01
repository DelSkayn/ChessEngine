#![allow(dead_code)]
#![allow(unused_imports)]

use chess_alpha_beta::AlphaBeta;
use chess_core::{
    board2::{Board, EndChain},
    eval::Eval,
    hash::Hasher,
};
use chess_mcts::Mcts;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, graphics, ContextBuilder,
};
use std::{env, path};
use structopt::StructOpt;

mod board;
mod game;
use board::RenderBoard;
mod player;
use player::{MousePlayer, Player, RandomPlayer, ThreadedEval};

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(short, long)]
    self_play: bool,
    fen: Option<String>,
}

fn main() {
    let args = Opt::from_args();

    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let hasher = Hasher::new();

    let board = if let Some(x) = args.fen {
        Board::from_fen(&x, EndChain).unwrap()
    } else {
        Board::start_position(EndChain)
    };

    let white = Box::new(ThreadedEval::new(2.0, chess_alpha_beta_2::AlphaBeta::new()));
    let black: Box<dyn Player> = if args.self_play {
        let mut engine = Mcts::new();
        engine.retry_quites = true;
        Box::new(MousePlayer::new())
    } else {
        Box::new(ThreadedEval::new(2.0, chess_alpha_beta::AlphaBeta::new()))
    };

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("Chess", "Mees Delzenne")
        .add_resource_path(resource_dir)
        .window_mode(WindowMode::default().resizable(true))
        .window_setup(WindowSetup::default().title("devapp chess"))
        .build()
        .expect("aieee, could not create ggez context!");

    println!("{}", graphics::renderer_info(&mut ctx).unwrap());
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = game::Chess::new(&mut ctx, board, hasher, white, black);

    // Run!
    event::run(ctx, event_loop, my_game)
}
