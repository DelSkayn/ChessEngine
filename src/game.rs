use engine::{hash::Hasher, Board};
use ggez::{
    audio::{SoundSource, Source},
    event::{EventHandler, MouseButton},
    graphics::{self, Color, Image, Rect},
    Context, GameResult,
};

use crate::{player::Player, RenderBoard};

#[derive(Eq, PartialEq, Debug)]
pub enum PlayedMove {
    Didnt,
    Move,
    Castle,
}

pub struct Chess {
    board: RenderBoard,
    piece_sprite: Image,
    castle_sound: Source,
    move_sound: Source,
    play_move: PlayedMove,
    white: Box<dyn Player>,
    black: Box<dyn Player>,
}

impl Chess {
    pub fn new(
        ctx: &mut Context,
        board: Board,
        hasher: Hasher,
        mut white: Box<dyn Player>,
        mut black: Box<dyn Player>,
    ) -> Chess {
        let board = RenderBoard::new(board, hasher);
        if board.board.white_turn() {
            white.start_turn(&board);
        } else {
            black.start_turn(&board);
        }
        Chess {
            piece_sprite: Image::new(ctx, "/pieces.png").unwrap(),
            castle_sound: Source::new(ctx, "/castle.ogg").unwrap(),
            move_sound: Source::new(ctx, "/move.ogg").unwrap(),
            play_move: PlayedMove::Didnt,
            white,
            board,
            black,
        }
    }

    fn white_turn(&self) -> bool {
        self.board.board.white_turn()
    }
}

impl EventHandler for Chess {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.play_move {
            PlayedMove::Didnt => {}
            PlayedMove::Castle => {
                self.castle_sound.play(ctx)?;
                self.play_move = PlayedMove::Didnt
            }
            PlayedMove::Move => {
                println!("MOVE");
                self.move_sound.play(ctx)?;
                self.play_move = PlayedMove::Didnt
            }
        }

        self.play_move = if self.white_turn() {
            self.white.update(&mut self.board)
        } else {
            self.black.update(&mut self.board)
        };

        if self.play_move != PlayedMove::Didnt {
            println!("FEN: {}", self.board.board.to_fen());
            if self.white_turn() {
                self.white.start_turn(&self.board);
            } else {
                self.black.start_turn(&self.board);
            }
        }

        match self.play_move {
            PlayedMove::Didnt => {}
            PlayedMove::Castle => {
                self.castle_sound.play(ctx)?;
                self.play_move = PlayedMove::Didnt
            }
            PlayedMove::Move => {
                println!("MOVE");
                self.move_sound.play(ctx)?;
                self.play_move = PlayedMove::Didnt
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::from_rgb_u32(0x282828));

        let coords = graphics::screen_coordinates(&ctx);
        self.board.draw(ctx, coords, &self.piece_sprite)?;

        // Draw code here...
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if self.white_turn() {
            self.white
                .mouse_button_down_event(button, x, y, &mut self.board);
        } else {
            self.black
                .mouse_button_down_event(button, x, y, &mut self.board);
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        self.play_move = if self.white_turn() {
            self.white
                .mouse_button_up_event(button, x, y, &mut self.board)
        } else {
            self.black
                .mouse_button_up_event(button, x, y, &mut self.board)
        };

        if self.play_move != PlayedMove::Didnt {
            println!("FEN: {}", self.board.board.to_fen());
            if self.white_turn() {
                self.white.start_turn(&self.board);
            } else {
                self.black.start_turn(&self.board);
            }
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, dx: f32, dy: f32) {
        if self.white_turn() {
            self.white.mouse_motion_event(x, y, dx, dy, &mut self.board);
        } else {
            self.black.mouse_motion_event(x, y, dx, dy, &mut self.board);
        }
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
