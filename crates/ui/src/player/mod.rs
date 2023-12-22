use crate::{board::RenderBoard, game::PlayedMove};
use common::{Move, Square};
use ggez::{event::MouseButton, input::keyboard::KeyInput, winit::event::VirtualKeyCode};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};

mod uci;
pub use uci::UciPlayer;

pub trait Player {
    fn update(&mut self, _board: &mut RenderBoard) -> PlayedMove {
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, _board: &RenderBoard) {}

    fn key_down(&mut self, _board: &mut RenderBoard, _key: KeyInput) {}

    fn mouse_button_down_event(
        &mut self,
        _button: MouseButton,
        _x: f32,
        _y: f32,
        _board: &mut RenderBoard,
    ) {
    }

    fn mouse_motion_event(
        &mut self,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
        _board: &mut RenderBoard,
    ) {
    }

    fn mouse_button_up_event(
        &mut self,
        _button: MouseButton,
        _x: f32,
        _y: f32,
        _board: &mut RenderBoard,
    ) -> PlayedMove {
        PlayedMove::Didnt
    }
}

pub struct NullPlayer;

impl Player for NullPlayer {}

pub struct MousePlayer {
    move_gen: MoveGenerator,
    possible_moves: InlineBuffer,
    holding: Option<Square>,
    dragging: bool,
}

impl MousePlayer {
    pub fn new() -> Self {
        MousePlayer {
            move_gen: MoveGenerator::new(),
            possible_moves: InlineBuffer::new(),
            holding: None,
            dragging: false,
        }
    }
}

impl Player for MousePlayer {
    fn start_turn(&mut self, board: &RenderBoard) {
        self.possible_moves.clear();
        self.move_gen
            .gen_moves::<gen_type::All>(&board.board, &mut self.possible_moves);
        print!("possible moves:");
        for m in self.possible_moves.iter() {
            print!("{},", m);
        }
        println!();
    }

    fn key_down(&mut self, board: &mut RenderBoard, key: KeyInput) {
        if dbg!(key.keycode) == Some(VirtualKeyCode::Left) {
            board.undo_move();
            self.possible_moves.clear();
            self.move_gen
                .gen_moves::<gen_type::All>(&board.board, &mut self.possible_moves);
        }
    }

    fn mouse_motion_event(
        &mut self,
        _x: f32,
        _y: f32,
        _dx: f32,
        _dy: f32,
        board: &mut RenderBoard,
    ) {
        if let Some(x) = self.holding {
            board.drag(x);
            self.dragging = true;
        }
    }

    fn mouse_button_down_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32,
        board: &mut RenderBoard,
    ) {
        if button == MouseButton::Left {
            if let Some(x) = board.square([x, y]) {
                if board
                    .on(x)
                    .map(|x| x.player() == board.board.state.player)
                    .unwrap_or(false)
                {
                    board.select(x);
                    let mut moves = Vec::new();
                    for m in self.possible_moves.iter() {
                        if m.from() == x {
                            moves.push(m.to())
                        }
                    }
                    board.set_possible(moves);
                    self.holding = Some(x);
                }
            }
        } else if button == MouseButton::Right {
            board.clear_select();
        }
    }

    fn mouse_button_up_event(
        &mut self,
        button: MouseButton,
        x: f32,
        y: f32,
        board: &mut RenderBoard,
    ) -> PlayedMove {
        if button != MouseButton::Left {
            return PlayedMove::Didnt;
        }

        if self.dragging {
            board.clear_drag();
            if let Some(from) = self.holding.take() {
                if let Some(to) = board.square([x, y]) {
                    for m in self.possible_moves.as_slice().iter().copied() {
                        if m.from() == from && m.to() == to {
                            board.highlight(from, to);
                            board.make_move(m);
                            assert!(board.board.is_valid(), "{:?}", board.board);
                            if m.ty() == Move::TYPE_CASTLE {
                                return PlayedMove::Castle;
                            } else {
                                return PlayedMove::Move;
                            }
                        }
                    }
                }
            }
        } else {
            board.clear_drag();
            self.holding = None;
        }
        PlayedMove::Didnt
    }
}
