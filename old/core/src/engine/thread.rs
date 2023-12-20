use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::{
    engine::{Engine, EngineControl, EngineLimit, Info, OptionValue},
    Board, Move,
};
use crossbeam_channel::{Receiver, Sender};

enum Cmd {
    SetOption(String, OptionValue),
    SetBoard(Board),
    MakeMove(Move),
    Go {
        time_left: Option<Duration>,
        limits: EngineLimit,
    },
}

pub enum Response {
    Info(Info),
    Done(Option<Move>),
}

struct ThreadControllerInner {
    quit: AtomicBool,
    sender: Sender<Response>,
}

pub struct ThreadController(Arc<ThreadControllerInner>);

impl Default for ThreadController {
    fn default() -> Self {
        ThreadController(Arc::new(ThreadControllerInner {
            quit: AtomicBool::new(false),
            sender: crossbeam_channel::bounded(0).0,
        }))
    }
}

impl EngineControl for ThreadController {
    fn should_stop(&self) -> bool {
        self.0.quit.load(Ordering::Relaxed)
    }

    fn info(&self, info: Info) {
        self.0.sender.send(Response::Info(info)).ok();
    }
}

pub struct EngineThread {
    reciever: Receiver<Response>,
    cmd_send: Sender<Cmd>,
    controller: Arc<ThreadControllerInner>,
}

/// A struct for running an engine on a different thread.
impl EngineThread {
    pub fn new<E>(mut engine: E) -> Self
    where
        E: Engine<ThreadController> + Send + 'static,
    {
        let (sender, reciever) = crossbeam_channel::bounded(8);
        let (cmd_send, cmd_recv) = crossbeam_channel::bounded(8);

        let controller = Arc::new(ThreadControllerInner {
            quit: AtomicBool::new(false),
            sender,
        });

        let controller_move = controller.clone();
        std::thread::spawn(move || {
            while let Ok(x) = cmd_recv.recv() {
                match x {
                    Cmd::SetBoard(b) => {
                        engine.set_board(b);
                    }
                    Cmd::MakeMove(m) => {
                        engine.make_move(m);
                    }
                    Cmd::SetOption(name, value) => engine.set_option(name, value),
                    Cmd::Go { limits, time_left } => {
                        let res =
                            engine.go(ThreadController(controller_move.clone()), time_left, limits);
                        controller_move.sender.send(Response::Done(res)).ok();
                    }
                }
            }
        });

        EngineThread {
            reciever,
            controller,
            cmd_send,
        }
    }

    pub fn start(&self, time_left: Option<Duration>, limits: EngineLimit) {
        self.controller.quit.store(false, Ordering::Release);
        self.cmd_send.send(Cmd::Go { time_left, limits }).unwrap();
    }

    pub fn stop(&self) {
        self.controller.quit.store(true, Ordering::Relaxed);
    }

    pub fn recv(&self) -> &Receiver<Response> {
        &self.reciever
    }

    pub fn set_option(&self, name: String, value: OptionValue) {
        self.cmd_send.send(Cmd::SetOption(name, value)).unwrap();
    }

    pub fn set_board(&self, b: Board) {
        self.cmd_send.send(Cmd::SetBoard(b)).unwrap();
    }

    pub fn make_move(&self, m: Move) {
        self.cmd_send.send(Cmd::MakeMove(m)).unwrap();
    }
}
