use common::{board::Board, Move};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use uci::{
    engine::{self, Engine, RunContext, SearchResult},
    req::GoRequest,
    UciMove,
};

pub struct LastEngine {
    move_gen: MoveGenerator,
    position: Board,
}

impl Engine for LastEngine {
    const NAME: &'static str = concat!("Last Engine ", env!("CARGO_PKG_VERSION"));
    const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

    fn new() -> Self {
        LastEngine {
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

        let mut moves: Vec<_> = buffer.as_slice().to_vec();
        moves.sort_by_key(|x| UciMove::from(*x).to_string());

        SearchResult {
            r#move: moves.last().copied().unwrap_or(Move::NULL),
            ponder: None,
        }
    }
}

fn main() {
    engine::run::<LastEngine>().unwrap()
}
