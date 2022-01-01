use anyhow::Result;
use chess_alpha_beta_2::AlphaBeta;
use chess_core::uci::Uci;

fn main() -> Result<()> {
    Uci::new(AlphaBeta::new()).start()
}
