#![allow(dead_code)]

use ggez::{
    conf::WindowMode,
    event::{self, EventHandler, MouseButton},
    graphics::{self, Color, Image, Rect},
    timer, Context, ContextBuilder, GameResult,
};
use std::{env, path};

pub mod board;
use board::{Board, Move};

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
        .window_mode(WindowMode::default().resizable(true))
        .build()
        .expect("aieee, could not create ggez context!");

    println!("{}", graphics::renderer_info(&mut ctx).unwrap());
    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object to
    // use when setting your game up.
    let my_game = Chess::new(&mut ctx, dbg!(board));

    // Run!
    event::run(ctx, event_loop, my_game)
}

struct Chess {
    moves: Vec<Move>,
    start_board: Board,
    board: Board,
    piece_sprite: Image,
}

impl Chess {
    pub fn new(ctx: &mut Context, board: Board) -> Chess {
        Chess {
            moves: Vec::new(),
            start_board: board,
            board,
            piece_sprite: Image::new(ctx, "/pieces.png").unwrap(),
        }
    }
}

impl EventHandler for Chess {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        while timer::check_update_time(ctx, 30) {
            if self.moves.is_empty() {
                self.moves = self.start_board.gen_moves();
            }
            self.board = self.start_board.make_move(self.moves.pop().unwrap());
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb(0x28, 0x28, 0x28));

        let coords = graphics::screen_coordinates(&ctx);
        self.board.draw(ctx, coords, &self.piece_sprite)?;

        // Draw code here...
        graphics::present(ctx)
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.board = self.board.flip();
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(
            ctx,
            Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            },
        )
        .unwrap();
        println!("resized!: {}, {}", width, height);
    }
}
