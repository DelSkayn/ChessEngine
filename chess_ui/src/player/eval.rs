use super::Player;
use crate::{board::RenderBoard, game::PlayedMove};
use chess_core::{
    board::{Board, EndChain},
    engine::{Engine, EngineControl, EngineLimit, EngineThread, Info, Response, ThreadController},
    Move,
};
use crossbeam_channel::TryRecvError;
use std::time::{Duration, Instant};

pub struct ThreadedEval {
    board: Board,
    time: Option<Instant>,
    best_move: Option<Move>,
    manager: EngineThread,
    search_time: Duration,
}

impl ThreadedEval {
    pub fn new<E: Engine<ThreadController> + Send>(search_time: f32, e: E) -> Self {
        ThreadedEval {
            board: Board::start_position(EndChain),
            search_time: Duration::from_secs_f32(search_time),
            time: None,
            best_move: None,
            manager: EngineThread::new(e),
        }
    }

    fn handle_info(info: Info) {
        match info {
            Info::Depth(x) => print!("{}: ", x),
            Info::BestMove { mov, value } => print!("{} = {}", mov, value),
            Info::Nodes(x) => print!(" ({} nodes)", x),
            Info::TransHit(x) => println!(" ({} trans_hit)", x),
            Info::Pv(x) => {
                print!("PV: ");
                x.iter().for_each(|x| print!("{} ", x));
                println!();
            }
            _ => {}
        }
    }
}

impl Player for ThreadedEval {
    fn update(&mut self, board: &mut RenderBoard) -> PlayedMove {
        if self.time.unwrap().elapsed() > self.search_time {
            self.manager.stop();
        }

        match self.manager.recv().try_recv() {
            Ok(x) => match x {
                Response::Info(x) => Self::handle_info(x),
                Response::Done(x) => {
                    if let Some(x) = x {
                        println!("MOVE: {}", x);
                        self.time.take();
                        board.make_move(x);
                        self.board.make_move(x);
                        self.manager.make_move(x);
                        if x.ty() == Move::TYPE_CASTLE {
                            return PlayedMove::Castle;
                        } else {
                            return PlayedMove::Move;
                        }
                    }
                }
            },
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => panic!("Engine thread disconnected!"),
        }

        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &RenderBoard) {
        self.time = Some(Instant::now());
        if let Some(x) = board.made_moves.last() {
            let mut cmp_board = board.board.clone();
            cmp_board.unmake_move(*x);
            if cmp_board.is_equal(&self.board) {
                self.board.make_move(x.mov);
                self.manager.make_move(x.mov);
            } else {
                self.board.copy_position(&board.board);
                self.manager.set_board(board.board.clone());
            }
        } else {
            self.board.copy_position(&board.board);
            self.manager.set_board(board.board.clone());
        }
        self.manager
            .start(Some(self.search_time), EngineLimit::none());
    }
}
