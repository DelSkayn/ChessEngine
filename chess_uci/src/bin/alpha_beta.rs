use anyhow::Result;
use chess_uci::Uci;

fn main() -> Result<()> {
    Uci::new(chess_alpha_beta::AlphaBeta::new()).start()
}
