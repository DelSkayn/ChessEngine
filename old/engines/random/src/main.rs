use chess::{
    board::EndChain,
    gen::{gen_type, MoveGenerator},
    uci::engine::{self, Engine, SearchResult},
    Board,
};
use rand::Rng;

struct RandomEngine {
    board: Board,
    gen: MoveGenerator,
}

impl Engine for RandomEngine {
    const NAME: &'static str = "random";
    const AUTHOR: &'static str = "Mees Delzenne";

    fn options() -> Vec<chess::uci::OptionMsg> {
        Vec::new()
    }

    fn set_option(&mut self, _name: String, _value: Option<String>) {}

    fn new_position(&mut self, board: chess::Board) {
        self.board = board;
    }

    fn make_move(&mut self, r#move: chess::Move) {
        self.board.make_move(r#move);
    }

    fn search(
        &mut self,
        _config: chess::uci::engine::SearchConfig,
        _signal: chess::uci::engine::EngineSignal,
    ) -> chess::uci::engine::SearchResult {
        let mut moves = Vec::new();
        self.gen
            .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves);
        if !moves.is_empty() {
            let idx = rand::thread_rng().gen_range(0..moves.len());
            let m = moves[idx];
            SearchResult {
                r#move: Some(m),
                ponder: None,
            }
        } else {
            SearchResult {
                r#move: None,
                ponder: None,
            }
        }
    }
}

fn main() {
    engine::run(RandomEngine {
        board: Board::start_position(EndChain),
        gen: MoveGenerator::new(),
    })
}
