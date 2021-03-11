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
    best: Mutex<Option<BestMove>>,
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
            best: Mutex::new(None),
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
                    *context.best.lock().unwrap() = Some(b)
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
            let mov = self
                .context
                .best
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .mov
                .unwrap();
            board.make_move(mov);
            return PlayedMove::Move;
        }
        PlayedMove::Didnt
    }

    fn start_turn(&mut self, board: &RenderBoard) {
        *self.context.best.lock().unwrap() = None;
        self.sender.send(board.board).unwrap();
        self.time = Some(Instant::now());
    }
}
