use common::{board::Board, Move};
use move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use rand::seq::SliceRandom;
use uci::{
    engine::{self, Engine, RunContext, SearchResult},
    req::GoRequest,
};

pub struct RandomEngine {
    move_gen: MoveGenerator,
    position: Board,
}

impl Engine for RandomEngine {
    const NAME: &'static str = concat!("Random Engine ", env!("CARGO_PKG_VERSION"));
    const AUTHOR: &'static str = env!("CARGO_PKG_AUTHORS");

    fn new() -> Self {
        RandomEngine {
            move_gen: MoveGenerator::new(),
            position: Board::start_position(),
        }
    }

    fn options(&self) -> std::collections::HashMap<String, uci::resp::OptionKind> {
        std::collections::HashMap::new()
    }

    fn set_option(&mut self, _name: &str, _value: Option<uci::req::OptionValue>) -> bool {
        true
    }

    fn new_game(&mut self) {}

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

        let pick = buffer
            .as_slice()
            .choose(&mut rand::thread_rng())
            .copied()
            .unwrap_or(Move::NULL);

        SearchResult {
            r#move: pick,
            ponder: None,
        }
    }
}

fn main() {
    engine::run::<RandomEngine>().unwrap()
}
