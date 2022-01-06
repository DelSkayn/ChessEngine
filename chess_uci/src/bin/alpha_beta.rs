use anyhow::Result;
use chess_uci::Uci;

fn main() -> Result<()> {
    Uci::new(chess_alpha_beta_2::AlphaBeta::new()).start()
}
