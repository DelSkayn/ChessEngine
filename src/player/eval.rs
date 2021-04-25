use super::Player;
use crate::{board::RenderBoard, game::PlayedMove};
use engine::{
    eval::{BestMove, Eval},
    hash::Hasher,
    Board, Move,
};
use std::{
    sync::{Arc,atomic::{Ordering,AtomicBool},mpsc::{self, Receiver, Sender}},
    thread,
    time::{Duration, Instant},
};

pub struct EvalCmd {
    board: Board,
    sender: Sender<Option<BestMove>>,
}

pub struct ThreadedEval {
    time: Option<Instant>,
    sender: Sender<EvalCmd>,
    reciever: Option<Receiver<Option<BestMove>>>,
    best_move: Option<Move>,
    value: i32,
    stop: Arc<AtomicBool>
}

impl ThreadedEval {
    pub fn new(hasher: Hasher) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let (sender, recv) = mpsc::channel();
        thread::spawn(|| Self::run(hasher, recv,stop_clone));
        ThreadedEval {
            time: None,
            sender,
            reciever: None,
            best_move: None,
            value: 0,
            stop,
        }
    }

    fn run(hasher: Hasher, channel: Receiver<EvalCmd>,stop: Arc<AtomicBool>) {
        let mut eval = Eval::new(hasher, 1 << 12,stop);

        for cmd in channel {
            let board = cmd.board;
            let sender = cmd.sender;
            println!("running!");
            eval.eval(&board, &mut |b: Option<BestMove>| {
                if let Some(b) = b.as_ref() {
                    if let Some(mov) = b.mov{
                        println!(
                            "{}:{}\t={}\t(nodes: {}, hits:{}, cut_offs:{})",
                            b.depth, mov, b.value, b.nodes_evaluated, b.table_hits, b.cut_offs
                        );
                    }
                }

                sender.send(b).is_ok()
            });

            println!("finished!");
        }
    }
}

impl Player for ThreadedEval {
    fn update(&mut self, board: &mut RenderBoard) -> PlayedMove {
        while let Some(x) = self.reciever.as_ref().unwrap().try_recv().ok() {
            if let Some(x) = x {
                self.best_move = x.mov;
                self.value = x.value;
            }
        }

        if self.time.unwrap().elapsed() > Duration::from_secs(1)
            || self.best_move.is_some() && self.value == Eval::CHECK_VALUE
        {
            if let Some(mov) = self.best_move.take() {
                self.reciever.take();
                self.stop.store(true,Ordering::Relaxed);
                board.make_move(mov);
                return PlayedMove::Move;
            }
        }
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &RenderBoard) {
        let (sender, recv) = mpsc::channel();
        self.time = Some(Instant::now());
        self.stop.store(false,Ordering::Release);
        self.reciever = Some(recv);
        self.sender
            .send(EvalCmd {
                board: board.board.clone(),
                sender,
            })
            .unwrap();
    }
}
