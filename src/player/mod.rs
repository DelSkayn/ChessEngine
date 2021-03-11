use crate::{board::RenderBoard, game::PlayedMove};
use engine::{Move, MoveGenerator, Square};
use ggez::event::MouseButton;

mod eval;
pub use eval::ThreadedEval;
mod random;
pub use random::RandomPlayer;

pub trait Player {
    fn update(&mut self, _board: &mut RenderBoard) -> PlayedMove {
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, _board: &RenderBoard) {}

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
    possible_moves: Vec<Move>,
    white: bool,
    holding: Option<Square>,
    dragging: bool,
}

impl MousePlayer {
    pub fn new(white: bool) -> Self {
        MousePlayer {
            move_gen: MoveGenerator::new(),
            possible_moves: Vec::new(),
            white,
            holding: None,
            dragging: false,
        }
    }
}

impl Player for MousePlayer {
    fn start_turn(&mut self, board: &RenderBoard) {
        self.possible_moves.clear();
        self.move_gen
            .gen_moves(&board.board, &mut self.possible_moves);
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
                    .map(|x| x.white() == self.white)
                    .unwrap_or(false)
                {
                    board.select(x);
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
            if let Some(b_from) = self.holding.take() {
                if let Some(b_to) = board.square([x, y]) {
                    dbg!(&self.possible_moves);
                    for m in self.possible_moves.iter().copied() {
                        match m {
                            Move::Simple { from, to, .. } => {
                                if from == b_from && to == b_to {
                                    board.highlight(from, to);
                                    board.make_move(m);
                                    return PlayedMove::Move;
                                }
                            }
                            _ => {}
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
