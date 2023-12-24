use common::{board::Board, Move, Player, SquareContent};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use uci::{
    engine::{self, Engine, RunContext, SearchResult},
    req::GoRequest,
};

pub struct CCCPEngine {
    move_gen: MoveGenerator,
    position: Board,
}

impl Engine for CCCPEngine {
    const NAME: &'static str = concat!("CCCP Engine ", env!("CARGO_PKG_VERSION"));
    const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

    fn new() -> Self {
        CCCPEngine {
            move_gen: MoveGenerator::new(),
            position: Board::start_position(),
        }
    }

    fn position(&mut self, board: Board, moves: &[uci::UciMove]) {
        self.position = board;

        for m in moves {
            let mut buffer = InlineBuffer::new();
            self.move_gen
                .gen_moves::<gen_type::All>(&self.position, &mut buffer);
            let Some(r#move) = m.to_move(buffer.as_slice()) else {
                break;
            };
            self.position.make_move(r#move);
            buffer.clear();
        }
    }

    fn go(&mut self, _settings: &GoRequest, _context: RunContext<'_>) -> SearchResult {
        let mut buffer = InlineBuffer::new();
        self.move_gen
            .gen_moves::<gen_type::All>(&self.position, &mut buffer);

        let mut board = self.position.clone();

        // checkmate
        for m in buffer.iter() {
            let undo = board.make_move(m);
            let info = self.move_gen.gen_info(&board);
            if self.move_gen.check_mate(&board, &info) {
                return SearchResult {
                    r#move: m,
                    ponder: None,
                };
            }
            board.unmake_move(undo);
        }

        // check
        for m in buffer.iter() {
            let undo = board.make_move(m);
            let info = self.move_gen.gen_info(&board);
            if self.move_gen.checked_king(&board, &info) {
                return SearchResult {
                    r#move: m,
                    ponder: None,
                };
            }
            board.unmake_move(undo);
        }

        // capture
        let mut best = None;
        let mut score = 0;
        for m in buffer.iter() {
            match board.squares[m.to()] {
                SquareContent::WhiteQueen | SquareContent::BlackQueen => {
                    best = Some(m);
                    score = 9;
                }
                SquareContent::WhiteRook | SquareContent::BlackRook => {
                    if score < 5 {
                        best = Some(m);
                        score = 5;
                    }
                }
                SquareContent::WhiteBishop
                | SquareContent::BlackBishop
                | SquareContent::WhiteKnight
                | SquareContent::BlackKnight => {
                    if score < 3 {
                        best = Some(m);
                        score = 3;
                    }
                }
                SquareContent::WhitePawn | SquareContent::BlackPawn => {
                    if score < 1 {
                        best = Some(m);
                        score = 1;
                    }
                }
                _ => {}
            }
        }

        if let Some(best) = best {
            return SearchResult {
                r#move: best,
                ponder: None,
            };
        }

        // push
        let m = buffer
            .iter()
            .max_by_key(|x| {
                if board.state.player == Player::White {
                    x.to().rank() as i8 - x.from().rank() as i8
                } else {
                    x.from().rank() as i8 - x.to().rank() as i8
                }
            })
            .unwrap_or(Move::NULL);

        SearchResult {
            r#move: m,
            ponder: None,
        }
    }
}

fn main() {
    engine::run::<CCCPEngine>().unwrap()
}
