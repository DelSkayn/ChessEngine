use std::time::{Duration, Instant};

use crate::{board::RenderBoard, player::Player};
use common::{board::Board, Player as ChessPlayer};
use ggez::{
    audio::{SoundSource, Source},
    event::{EventHandler, MouseButton},
    graphics::{Canvas, Color, Image, Rect},
    input::keyboard::KeyInput,
    Context, GameError, GameResult,
};

#[derive(Eq, PartialEq, Debug)]
pub enum PlayedMove {
    Didnt,
    Move,
    Castle,
}

pub enum State {
    Playing,
    Waiting(Instant),
}

pub struct Chess {
    board: RenderBoard,
    piece_sprite: Image,
    castle_sound: Source,
    move_sound: Source,
    play_move: PlayedMove,
    white: Box<dyn Player>,
    black: Box<dyn Player>,
    set_coords: Option<Rect>,
    pause: Option<f32>,
    state: State,
}

impl Chess {
    pub fn new(
        ctx: &mut Context,
        board: Board,
        mut white: Box<dyn Player>,
        mut black: Box<dyn Player>,
    ) -> Chess {
        let board = RenderBoard::new(board);
        match board.board.state.player {
            ChessPlayer::White => white.start_turn(&board),
            ChessPlayer::Black => black.start_turn(&board),
        }
        Chess {
            piece_sprite: Image::from_path(ctx, "/pieces.png").unwrap(),
            castle_sound: Source::new(ctx, "/castle.ogg").unwrap(),
            move_sound: Source::new(ctx, "/move.ogg").unwrap(),
            play_move: PlayedMove::Didnt,
            white,
            board,
            black,
            set_coords: None,
            pause: None,
            state: State::Playing,
        }
    }

    pub fn set_pause(&mut self, pause: Option<f32>) {
        self.pause = pause;
    }

    fn white_turn(&self) -> bool {
        self.board.board.state.player == ChessPlayer::White
    }
}

impl EventHandler for Chess {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.state {
            State::Playing => {
                if self.play_move == PlayedMove::Didnt {
                    self.play_move = if self.white_turn() {
                        self.white.update(&mut self.board)
                    } else {
                        self.black.update(&mut self.board)
                    };
                }

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

                if let Some(pause) = self.pause {
                    self.state = State::Waiting(Instant::now() + Duration::from_secs_f32(pause))
                }
            }
            State::Waiting(x) => {
                if x < Instant::now() {
                    self.state = State::Playing
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb_u32(0x282828));

        if let Some(coords) = self.set_coords.take() {
            canvas.set_screen_coordinates(coords);
        }

        let coords = canvas.screen_coordinates().unwrap_or_default();
        self.board
            .draw(ctx, &mut canvas, coords, &self.piece_sprite)?;

        // Draw code here...
        canvas.finish(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        _repeat: bool,
    ) -> Result<(), GameError> {
        if self.white_turn() {
            self.white.key_down(&mut self.board, input);
        } else {
            self.black.key_down(&mut self.board, input);
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> Result<(), GameError> {
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
    ) -> Result<(), GameError> {
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
    ) -> Result<(), GameError> {
        if self.white_turn() {
            self.white.mouse_motion_event(x, y, dx, dy, &mut self.board);
        } else {
            self.black.mouse_motion_event(x, y, dx, dy, &mut self.board);
        }
        Ok(())
    }

    fn resize_event(
        &mut self,
        _ctx: &mut Context,
        width: f32,
        height: f32,
    ) -> Result<(), GameError> {
        self.set_coords = Some(Rect {
            x: 0.0,
            y: 0.0,
            w: width,
            h: height,
        });
        println!("resized!: {}, {}", width, height);
        Ok(())
    }
}
