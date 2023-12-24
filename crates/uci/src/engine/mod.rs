use crate::{
    req::{GoRequest, OptionValue},
    resp::{OptionKind, ResponseInfo},
    UciMove,
};
use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
};

mod run;
use common::{board::Board, Move};
pub use run::run;

static STOP: AtomicBool = AtomicBool::new(true);

#[derive(Clone)]
pub struct RunContext<'a> {
    sender: Sender<GoInfo>,
    marker: PhantomData<&'a ()>,
}

enum GoInfo {
    Info(ResponseInfo),
    BestMove {
        r#move: UciMove,
        ponder: Option<UciMove>,
    },
}

impl RunContext<'_> {
    pub fn should_stop() -> bool {
        STOP.load(Ordering::Relaxed)
    }

    pub fn force_run() {
        STOP.store(false, Ordering::Release)
    }

    pub fn info(&self, info: ResponseInfo) {
        let _ = self.sender.send(GoInfo::Info(info));
    }
}

pub struct SearchResult {
    pub r#move: Move,
    pub ponder: Option<Move>,
}

pub trait Engine {
    const NAME: &'static str;
    const AUTHOR: &'static str;

    fn new() -> Self;

    /// The options that the engine has.
    fn options(&self) -> HashMap<String, OptionKind> {
        HashMap::new()
    }

    fn set_option(&mut self, _name: &str, _value: Option<OptionValue>) -> bool {
        true
    }

    fn new_game(&mut self) {}

    fn position(&mut self, board: Board, moves: &[UciMove]);

    fn go(&mut self, settings: &GoRequest, context: RunContext<'_>) -> SearchResult;
}
