#![allow(dead_code)]

use engine::Board;
use ggez::{
    conf::{WindowMode, WindowSetup},
    event, graphics, ContextBuilder,
};
use std::{env, path};

mod board;
mod game;
use board::RenderBoard;
mod player;
use player::{MousePlayer, RandomPlayer, ThreadedEval};

fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let mut args = env::args();
    args.next();

    let board = if let Some(x) = args.next() {
        Board::from_fen(&x).unwrap()
    } else {
        Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    };

    // Make a Context.
    let (mut ctx, event_loop) = ContextBuilder::new("Chess", "Mees Delzenne")
        .add_resource_path(resource_dir)
        .window_mode(WindowMode::default().resizable(true))
        .window_setup(WindowSetup::default().title("devapp"))
        .build()
        .expect("aieee, could not create ggez context!");

    println!("{}", graphics::renderer_info(&mut ctx).unwrap());
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = game::Chess::new(
        &mut ctx,
        dbg!(board),
        true,
        Box::new(RandomPlayer::new()),
        Box::new(RandomPlayer::new()),
        //Box::new(MousePlayer::new(true)),
        //Box::new(MousePlayer::new(false)),
        //Box::new(ThreadedEval::new()),
    );

    // Run!
    event::run(ctx, event_loop, my_game)
}
