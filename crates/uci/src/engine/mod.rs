use common::board::Board;
use common::Move;

use crate::req::{GoRequest, OptionValue};
use crate::resp::{OptionKind, ResponseId, ResponseInfo, ResponseOption};
use crate::{req, Request, Response, UciMove};
use core::fmt;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender};
use std::{collections::HashMap, io};

static STOP: AtomicBool = AtomicBool::new(true);

enum EngineError {
    Io(io::Error),
    Parse(req::UciRequestError),
}

impl From<io::Error> for EngineError {
    fn from(value: io::Error) -> Self {
        EngineError::Io(value)
    }
}

impl From<req::UciRequestError> for EngineError {
    fn from(value: req::UciRequestError) -> Self {
        EngineError::Parse(value)
    }
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EngineError::Io(e) => {
                write!(f, "Io error: {e}")
            }
            EngineError::Parse(e) => {
                write!(f, "Parse error: {e}")
            }
        }
    }
}

#[derive(Clone)]
pub struct RunContext<'a> {
    sender: Sender<GoInfo>,
    marker: PhantomData<&'a ()>,
}

impl RunContext<'_> {
    pub fn should_stop() -> bool {
        STOP.load(Ordering::Relaxed)
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
    fn options(&self) -> HashMap<String, OptionKind>;

    fn set_option(&mut self, name: &str, value: Option<OptionValue>) -> bool;

    fn new_game(&mut self);

    fn position(&mut self, board: Board, moves: &[UciMove]);

    fn go(&mut self, settings: &GoRequest, context: RunContext<'_>) -> SearchResult;
}

enum GoInfo {
    Info(ResponseInfo),
    BestMove {
        r#move: UciMove,
        ponder: Option<UciMove>,
    },
}

enum EngineCmd {
    Options(SyncSender<HashMap<String, OptionKind>>),
    SetOption {
        name: String,
        value: Option<OptionValue>,
    },
    Sync(SyncSender<()>),
    Position {
        board: Board,
        moves: Vec<UciMove>,
    },
    Go {
        request: GoRequest,
        sender: Sender<GoInfo>,
    },
}

fn engine_thread<E: Engine>(recv: Receiver<EngineCmd>) {
    let mut engine = E::new();
    for next in recv.iter() {
        match next {
            EngineCmd::Options(x) => {
                let _ = x.send(engine.options());
            }
            EngineCmd::SetOption { name, value } => {
                engine.set_option(&name, value);
            }
            EngineCmd::Sync(x) => {
                let _ = x.send(());
            }
            EngineCmd::Position { board, moves } => engine.position(board, &moves),
            EngineCmd::Go { request, sender } => {
                let send_clone = sender.clone();
                let ctx = RunContext {
                    sender,
                    marker: PhantomData,
                };
                let r#move = engine.go(&request, ctx);
                let _ = send_clone.send(GoInfo::BestMove {
                    r#move: r#move.r#move.into(),
                    ponder: r#move.ponder.map(|x| x.into()),
                });
            }
        }
    }
}

fn print_thread(recv: Receiver<GoInfo>) {
    for r in recv.iter() {
        match r {
            GoInfo::Info(x) => {
                println!("{}", Response::Info(vec![x]));
            }
            GoInfo::BestMove { r#move, ponder } => {
                println!("{}", Response::BestMove { r#move, ponder })
            }
        }
    }
}

pub fn run<E: Engine>() -> Result<(), io::Error> {
    let (send, recv) = mpsc::channel();

    std::thread::spawn(move || {
        engine_thread::<E>(recv);
    });

    loop {
        let cmd = match read_command() {
            Ok(Some(x)) => x,
            Ok(None) => break,
            Err(EngineError::Parse(x)) => {
                println!("invalid command: {x}");
                continue;
            }
            Err(EngineError::Io(e)) => return Err(e),
        };
        match cmd {
            Request::Uci => {
                let (opt_send, opt_recv) = mpsc::sync_channel(0);

                send.send(EngineCmd::Options(opt_send)).unwrap();

                println!("{}", Response::Id(ResponseId::Name(E::NAME.to_owned())));
                println!("{}", Response::Id(ResponseId::Author(E::AUTHOR.to_owned())));

                println!();

                let options = opt_recv.recv().unwrap();

                for (name, opt) in options {
                    println!("{}", Response::Option(ResponseOption { name, kind: opt }));
                }

                println!();

                println!("{}", Response::UciOk);
            }
            Request::Debug(_) => {}
            Request::IsReady => {
                println!("{}", Response::ReadyOk);
            }
            Request::UciNewGame => {}
            Request::SetOption { name, value } => {
                send.send(EngineCmd::SetOption { name, value }).unwrap();
            }
            Request::Position { fen, moves } => {
                let board = fen.map(|x| *x).unwrap_or_else(Board::start_position);
                send.send(EngineCmd::Position { board, moves }).unwrap();
            }
            Request::Go(x) => {
                let (sender, recv) = mpsc::channel();
                std::thread::spawn(|| print_thread(recv));

                send.send(EngineCmd::Go { request: x, sender }).unwrap();
                STOP.store(false, Ordering::Release);
            }
            Request::Stop => {
                STOP.store(true, Ordering::Release);
                let (sync_sender, recv) = mpsc::sync_channel(0);
                send.send(EngineCmd::Sync(sync_sender)).unwrap();
                let _ = recv.recv();
            }
            Request::PonderHit => {}
            Request::Quit => return Ok(()),
        }
    }

    Ok(())
}

fn read_command() -> Result<Option<Request>, EngineError> {
    let mut buffer = String::new();
    let read = std::io::stdin().read_line(&mut buffer)?;
    if read == 0 {
        return Ok(None);
    }

    let res = buffer.trim().parse()?;
    Ok(Some(res))
}
