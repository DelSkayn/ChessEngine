use anyhow::Result;
use chess_core::uci::Uci;
use chess_mcts::Mcts;

fn main() -> Result<()> {
    Uci::new(Mcts::new()).start()
}
