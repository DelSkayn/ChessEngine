use std::time::{Duration, Instant};

use crate::{board::RenderBoard, player::Player};
use common::{
    board::Board,
    misc::{DrawCause, Outcome, WinCause},
    Player as ChessPlayer,
};
use ggez::{
    audio::{SoundSource, Source},
    event::{EventHandler, MouseButton},
    graphics::{Canvas, Color, Image, Rect},
    input::keyboard::KeyInput,
    winit::event::VirtualKeyCode,
    Context, GameError, GameResult,
};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};

#[derive(Eq, PartialEq, Debug)]
pub enum PlayedMove {
    Didnt,
    Move,
    Castle,
}

#[derive(Eq, PartialEq, Debug)]
pub enum State {
    Playing { start: Instant },
    Waiting(Instant),
    Paused,
    Finished(Outcome),
}

pub struct Chess {
    board: RenderBoard,
    move_gen: MoveGenerator,
    piece_sprite: Image,
    castle_sound: Source,
    move_sound: Source,
    play_move: PlayedMove,
    white: Box<dyn Player>,
    black: Box<dyn Player>,
    time_taken: Vec<Duration>,
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
            time_taken: Vec::new(),
            move_gen: MoveGenerator::new(),
            black,
            set_coords: None,
            pause: None,
            state: State::Playing {
                start: Instant::now(),
            },
        }
    }

    pub fn set_timelimit(&mut self, duration: Duration) {
        self.board.white_time = Some(duration);
        self.board.black_time = Some(duration);
    }

    pub fn set_increment(&mut self, duration: Duration) {
        self.board.increment = duration;
    }

    pub fn set_pause(&mut self, pause: Option<f32>) {
        self.pause = pause;
    }

    fn white_turn(&self) -> bool {
        self.board.board.state.player == ChessPlayer::White
    }

    fn undo_move(&mut self) {
        if self.board.current_move == 0 {
            return;
        }

        self.board.undo_move();
        let time = if self.board.board.state.player == ChessPlayer::White {
            self.board.white_time.as_mut()
        } else {
            self.board.black_time.as_mut()
        };

        if let Some(d) = time {
            *d += self.time_taken[self.board.current_move];
            *d = d.checked_sub(self.board.increment).unwrap();
        }
    }

    fn redo_move(&mut self) {
        if self.board.current_move == self.board.made_moves.len() {
            return;
        }
        self.board.redo_move();
        let time = if self.board.board.state.player == ChessPlayer::White {
            self.board.black_time.as_mut()
        } else {
            self.board.white_time.as_mut()
        };
        if let Some(d) = time {
            *d += self.board.increment;
            *d = d
                .checked_sub(self.time_taken[self.board.current_move - 1])
                .unwrap();
        }
    }

    fn start_turn(&mut self) {
        let mut buffer = InlineBuffer::new();
        let info = self
            .move_gen
            .gen_moves::<gen_type::All>(&self.board.board, &mut buffer);

        if buffer.is_empty() {
            if self.move_gen.checked_king(&self.board.board, &info) {
                let by = self.board.board.state.player.flip();
                println!("player {by:?} won by mate");
                self.state = State::Finished(Outcome::Won {
                    by,
                    cause: WinCause::Mate,
                })
            } else {
                println!("game resulted in stale mate.");
                self.state = State::Finished(Outcome::Drawn(DrawCause::Stalemate));
            }
            return;
        }

        if self.board.board.state.player == ChessPlayer::White {
            self.white.start_turn(&self.board);
        } else {
            self.black.start_turn(&self.board);
        }

        if let Some(p) = self.pause {
            self.state = State::Waiting(Instant::now() + Duration::from_secs_f32(p));
        } else {
            self.state = State::Playing {
                start: Instant::now(),
            }
        }
    }
}

impl EventHandler for Chess {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        match self.state {
            State::Playing { start } => {
                if self.play_move == PlayedMove::Didnt {
                    self.play_move = if self.white_turn() {
                        self.white.update(&mut self.board)
                    } else {
                        self.black.update(&mut self.board)
                    };
                }

                match self.play_move {
                    PlayedMove::Didnt => {}
                    PlayedMove::Castle => {
                        self.castle_sound.play(ctx)?;
                    }
                    PlayedMove::Move => {
                        println!("MOVE");
                        self.move_sound.play(ctx)?;
                    }
                }

                if self.play_move != PlayedMove::Didnt {
                    let time = if self.board.board.state.player == ChessPlayer::White {
                        self.board.black_time.as_mut()
                    } else {
                        self.board.white_time.as_mut()
                    };

                    let elapsed = start.elapsed();
                    self.time_taken.truncate(self.board.current_move - 1);
                    self.time_taken.push(elapsed);

                    if let Some(d) = time {
                        *d += self.board.increment;
                        if let Some(n) = d.checked_sub(elapsed) {
                            *d = n;
                        } else {
                            let by = self.board.board.state.player;
                            println!("player {by:?} won by timeout");
                            self.state = State::Finished(Outcome::Won {
                                by,
                                cause: WinCause::Timeout,
                            });
                            self.play_move = PlayedMove::Didnt;
                            return Ok(());
                        }
                    };

                    println!("FEN: {}", self.board.board.to_fen());
                    println!(
                        "WHITE TIME: {:?}",
                        self.board.white_time.unwrap_or_default()
                    );
                    println!(
                        "BLACK TIME: {:?}",
                        self.board.black_time.unwrap_or_default()
                    );
                    self.start_turn();
                } else if self.board.board.state.player == ChessPlayer::White {
                    if let Some(d) = self.board.white_time {
                        if start.elapsed() > d {
                            println!("player {:?} won by timeout", ChessPlayer::Black);
                            self.state = State::Finished(Outcome::Won {
                                by: ChessPlayer::Black,
                                cause: WinCause::Timeout,
                            });
                            self.white.stop();
                        }
                    }
                } else if let Some(d) = self.board.black_time {
                    if start.elapsed() > d {
                        println!("player {:?} won by timeout", ChessPlayer::White);
                        self.state = State::Finished(Outcome::Won {
                            by: ChessPlayer::Black,
                            cause: WinCause::Timeout,
                        });
                        self.black.stop();
                    }
                }

                self.play_move = PlayedMove::Didnt;
            }
            State::Waiting(x) => {
                if x < Instant::now() {
                    self.state = State::Playing {
                        start: Instant::now(),
                    }
                }
            }
            State::Paused => {}
            State::Finished(_) => {}
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut canvas = Canvas::from_frame(ctx, Color::from_rgb_u32(0x282828));

        if let Some(coords) = self.set_coords.take() {
            canvas.set_screen_coordinates(coords);
        }

        let coords = canvas.screen_coordinates().unwrap_or_default();

        let mut board_coords = coords;
        board_coords.w /= 2.0;
        board_coords.x = board_coords.w;

        board_coords.w -= 60.0;
        board_coords.x += 30.0;

        self.board
            .draw(ctx, &mut canvas, board_coords, &self.piece_sprite)?;

        // Draw code here...
        canvas.finish(ctx)
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        input: KeyInput,
        repeat: bool,
    ) -> Result<(), GameError> {
        if input.keycode == Some(VirtualKeyCode::P) {
            if self.state == State::Paused {
                self.state = State::Playing {
                    start: Instant::now(),
                };
                self.start_turn();
            } else {
                self.state = State::Paused;

                if self.white_turn() {
                    self.white.stop();
                } else {
                    self.black.stop();
                }
            }
            return Ok(());
        }

        if self.state == State::Paused {
            if repeat {
                return Ok(());
            }
            if input.keycode == Some(VirtualKeyCode::Left)
                || input.keycode == Some(VirtualKeyCode::U)
            {
                self.undo_move()
            }
            if input.keycode == Some(VirtualKeyCode::Right)
                || input.keycode == Some(VirtualKeyCode::R)
            {
                self.redo_move()
            }
            return Ok(());
        }

        if matches!(self.state, State::Finished(_)) {
            return Ok(());
        }

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
        if matches!(self.state, State::Paused | State::Finished(_)) {
            return Ok(());
        }

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
        if matches!(self.state, State::Paused | State::Finished(_)) {
            return Ok(());
        }

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
        if matches!(self.state, State::Paused | State::Finished(_)) {
            return Ok(());
        }
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
