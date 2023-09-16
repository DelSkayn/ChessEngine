use chess_core::{board::Board, hash::Hasher, Player as PlayerColor};
use ggez::{
    audio::{SoundSource, Source},
    event::{EventHandler, MouseButton},
    graphics::{self, Canvas, Color, Image, Rect},
    input::keyboard::KeyInput,
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
    resized: Option<Rect>,
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
        match board.board.state.player {
            PlayerColor::White => white.start_turn(&board),
            PlayerColor::Black => black.start_turn(&board),
        }
        Chess {
            piece_sprite: Image::from_path(ctx, "/pieces.png").unwrap(),
            castle_sound: Source::new(ctx, "/castle.ogg").unwrap(),
            move_sound: Source::new(ctx, "/move.ogg").unwrap(),
            play_move: PlayedMove::Didnt,
            white,
            board,
            black,
            resized: None,
        }
    }

    fn white_turn(&self) -> bool {
        self.board.board.state.player == PlayerColor::White
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
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        if let Some(x) = self.resized.take() {
            canvas.set_screen_coordinates(x);
        }
        let Some(coords) = canvas.screen_coordinates() else {
            canvas.finish(ctx)?;
            return Ok(());
        };
        self.board
            .draw(ctx, &mut canvas, coords, &self.piece_sprite)?;

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        _input: KeyInput,
        _repeat: bool,
    ) -> GameResult<()> {
        let Some(keycode) = _input.keycode else {
            return Ok(());
        };
        if self.white_turn() {
            self.white.key_down(&mut self.board, keycode);
        } else {
            self.black.key_down(&mut self.board, keycode);
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult<()> {
        if self.white_turn() {
            self.white
                .mouse_button_down_event(button, x, y, &mut self.board);
        } else {
            self.black
                .mouse_button_down_event(button, x, y, &mut self.board);
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult<()> {
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
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        dx: f32,
        dy: f32,
    ) -> GameResult<()> {
        if self.white_turn() {
            self.white.mouse_motion_event(x, y, dx, dy, &mut self.board);
        } else {
            self.black.mouse_motion_event(x, y, dx, dy, &mut self.board);
        }
        Ok(())
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) -> GameResult<()> {
        self.resized = Some(Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        });
        println!("resized!: {}, {}", width, height);
        Ok(())
    }
}
