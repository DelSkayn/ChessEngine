use crate::{board::RenderBoard, game::PlayedMove};
use engine::{Move, MoveGenerator, Square,Piece};
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
    holding: Option<Square>,
    dragging: bool,
}

impl MousePlayer {
    pub fn new() -> Self {
        MousePlayer {
            move_gen: MoveGenerator::new(),
            possible_moves: Vec::new(),
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
        println!("possible moves: {:?}", self.possible_moves)
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
                    .map(|x| x.white() == board.board.white_turn())
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
                    for m in self.possible_moves.iter().copied() {
                        match m {
                            Move::Quiet{ from, to, .. } => {
                                if from == b_from && to == b_to {
                                    board.highlight(from, to);
                                    board.make_move(m);
                                    return PlayedMove::Move;
                                }
                            }
                            Move::Capture{ from, to, .. } => {
                                if from == b_from && to == b_to {
                                    board.highlight(from, to);
                                    board.make_move(m);
                                    return PlayedMove::Move;
                                }
                            }
                            Move::Castle{ king } => {
                                if board.board.state.black_turn{
                                    if b_from == Square::E8 && b_to == Square::G8 && king{
                                        board.make_move(m);
                                        board.highlight(b_from, b_to);
                                        return PlayedMove::Move;
                                    }
                                    if b_from == Square::E8 && b_to == Square::C8 && !king{
                                        board.make_move(m);
                                        board.highlight(b_from, b_to);
                                        return PlayedMove::Move;
                                    }
                                }else{
                                    if b_from == Square::E1 && b_to == Square::G1 && king{
                                        board.make_move(m);
                                        board.highlight(b_from, b_to);
                                        return PlayedMove::Move;
                                    }
                                    if b_from == Square::E1 && b_to == Square::C1 && !king{
                                        board.make_move(m);
                                        board.highlight(b_from, b_to);
                                        return PlayedMove::Move;
                                    }
                                }
                            }
                            Move::Promote{promote,from,to,..} => {
                                if from == b_from && to == b_to && (promote == Piece::WhiteQueen || promote == Piece::BlackQueen) {
                                    board.highlight(from, to);
                                    board.make_move(m);
                                    return PlayedMove::Move;
                                }
                            }
                            Move::PromoteCapture{promote,from,to,..} => {
                                if from == b_from && to == b_to && (promote == Piece::WhiteQueen || promote == Piece::BlackQueen) {
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
