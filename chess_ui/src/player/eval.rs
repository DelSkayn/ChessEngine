use super::Player;
use crate::{board::RenderBoard, game::PlayedMove};
use chess_core::{
    engine::{Info, ShouldRun, Engine},
    uci::ThreadManger,
    Move,
};
use std::time::{Duration, Instant};

pub struct ThreadedEval {
    time: Option<Instant>,
    best_move: Option<Move>,
    manager: ThreadManger,
    search_time: f32,
}

impl ThreadedEval {
    pub fn new<E: Engine + Send>(search_time: f32, e: E) -> Self {
        let manager = ThreadManger::new(e, Self::handle_info);
        ThreadedEval {
            search_time,
            time: None,
            best_move: None,
            manager,
        }
    }

    fn handle_info(info: Info) -> ShouldRun {
        match info {
            Info::Depth(x) => print!("{}: ", x),
            Info::BestMove { mov, value } => print!("{} = {}", mov, value),
            Info::Nodes(x) => println!(" ({} nodes)", x),
            _ => {}
        }
        ShouldRun::Continue
    }
}

impl Player for ThreadedEval {
    fn update(&mut self, board: &mut RenderBoard) -> PlayedMove {
        if self.time.unwrap().elapsed() > Duration::from_secs_f32(self.search_time) {
            if let Some(x) = self.manager.stop() {
                self.time.take();
                board.make_move(x);
                if x.ty() == Move::TYPE_CASTLE {
                    return PlayedMove::Castle;
                } else {
                    return PlayedMove::Move;
                }
            }
            return PlayedMove::Didnt;
        }
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &RenderBoard) {
        self.time = Some(Instant::now());
        self.manager.set_board(board.board.clone());
        self.manager.start();
    }
}
