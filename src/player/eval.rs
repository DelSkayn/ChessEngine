use super::Player;
use crate::{board::RenderBoard, game::PlayedMove};
use engine::{
    eval::{BestMove, Buffers, Eval},
    Board,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

pub struct Context {
    run: AtomicBool,
    best: Mutex<BestMove>,
}

pub struct ThreadedEval {
    time: Option<Instant>,
    context: Arc<Context>,
    sender: Sender<Board>,
}

impl ThreadedEval {
    pub fn new() -> Self {
        let context = Arc::new(Context {
            run: AtomicBool::new(true),
            best: Mutex::new(Default::default()),
        });
        let (sender, recv) = mpsc::channel();

        let ctx = context.clone();
        thread::spawn(|| Self::run(ctx, recv));

        ThreadedEval {
            time: None,
            context,
            sender,
        }
    }

    fn run(context: Arc<Context>, channel: Receiver<Board>) {
        let mut eval = Eval::new();
        let mut buffers = Buffers::default();

        for b in channel {
            println!("running!");
            eval.eval(&b, &mut buffers, &mut |b: Option<BestMove>| {
                if let Some(b) = b {
                    println!(
                        "{}:{:?}={} ({})",
                        b.depth, b.mov, b.value, b.nodes_evaluated
                    );
                    let mut l = context.best.lock().unwrap();
                    if let Some(x) = b.mov {
                        l.mov = Some(x);
                    }
                    l.depth = b.depth;
                    l.value = b.value;
                    l.nodes_evaluated = b.nodes_evaluated;
                }
                context.run.load(Ordering::Acquire)
            });

            println!("finished!");
            context.run.store(true, Ordering::Release);
        }
    }
}

impl Player for ThreadedEval {
    fn update(&mut self, board: &mut RenderBoard) -> PlayedMove {
        if self.time.unwrap().elapsed() > Duration::from_secs(1) {
            self.context.run.store(false, Ordering::Release);
            if let Some(mov) = self.context.best.lock().unwrap().mov {
                board.make_move(mov);
                return PlayedMove::Move;
            }
        }
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &RenderBoard) {
        self.context.best.lock().unwrap().mov = None;
        self.sender.send(board.board.clone()).unwrap();
        self.time = Some(Instant::now());
    }
}
