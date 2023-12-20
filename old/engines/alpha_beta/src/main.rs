use alpha_beta::AlphaBeta;
use chess_core::uci;

fn main() {
    uci::engine::run(AlphaBeta::new())
}
