#[macro_use]
extern crate bitflags;

use ggez::{
    event::{self, EventHandler},
    graphics::{self, Color, Image},
    Context, ContextBuilder, GameResult,
};
use std::{env, path};

mod board;
use board::Board;

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
    //
    //
    let (mut ctx, event_loop) = ContextBuilder::new("Chess", "Mees Delzenne")
        .add_resource_path(resource_dir)
        .build()
        .expect("aieee, could not create ggez context!");

    println!("{}", graphics::renderer_info(&mut ctx).unwrap());
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = Chess::new(&mut ctx, board);

    // Run!
    event::run(ctx, event_loop, my_game)
}

struct Chess {
    board: Board,
    piece_sprite: Image,
}

impl Chess {
    pub fn new(ctx: &mut Context, board: Board) -> Chess {
        Chess {
            board,
            piece_sprite: Image::new(ctx, "/pieces.png").unwrap(),
        }
    }
}

impl EventHandler for Chess {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(0x28, 0x28, 0x28));

        let coords = graphics::screen_coordinates(&ctx);
        self.board.draw(ctx, coords, &self.piece_sprite)?;

        // Draw code here...
        graphics::present(ctx)
    }
}
