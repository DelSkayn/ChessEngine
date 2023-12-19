use std::{
    io::BufRead,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub use crossbeam_channel::bounded;
use crossbeam_channel::{Receiver, Sender};

use crate::{
    board::EndChain,
    gen::MoveGenerator,
    uci::{Cmd, Id, InfoMsg, Msg, OptionMsg, StartOrFen, UciMove, Version},
    Board, Move,
};

#[derive(Clone)]
pub struct EngineSignal {
    stop: Arc<AtomicBool>,
    info: Sender<Vec<InfoMsg>>,
}

impl EngineSignal {
    /// Whether the GUI has asked the engine to stop.
    pub fn should_stop(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }

    /// Send a info message to the gui.
    pub fn info(&self, msg: Vec<InfoMsg>) {
        self.info.send(msg).unwrap();
    }
}

pub struct EngineCommand {
    stop: Arc<AtomicBool>,
    pub info: Receiver<Vec<InfoMsg>>,
}

impl EngineCommand {
    fn create() -> (EngineCommand, EngineSignal) {
        let stop = Arc::new(AtomicBool::new(false));
        let (send, recv) = crossbeam_channel::bounded(64);
        (
            EngineCommand {
                stop: stop.clone(),
                info: recv,
            },
            EngineSignal { stop, info: send },
        )
    }

    pub fn reset(&self) {
        self.stop.store(false, Ordering::Release);
    }

    pub fn stop(&self) {
        self.stop.store(true, Ordering::Release);
    }
}

#[derive(Default)]
pub struct SearchConfig {
    /// If some restrict the search to the given moves.
    pub moves: Option<Vec<Move>>,
    pub ponder: bool,
    pub wtime: Option<Duration>,
    pub btime: Option<Duration>,
    pub winc: Option<Duration>,
    pub binc: Option<Duration>,
    pub movestogo: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
    pub movetime: Option<Duration>,
    pub infinite: bool,
}

/// The result of a search.
pub struct SearchResult {
    /// The best move found, can be none if there are no moves available.
    pub r#move: Option<Move>,
    /// The move the engine would like to ponder if there is one.
    pub ponder: Option<Move>,
}

/// A trait which chess engines must implement.
pub trait Engine {
    const NAME: &'static str;
    const AUTHOR: &'static str;

    fn version() -> Option<Version> {
        None
    }

    fn options() -> Vec<OptionMsg>;

    fn set_option(&mut self, name: String, value: Option<String>);

    fn new_position(&mut self, board: Board);

    fn make_move(&mut self, r#move: Move);

    fn search(&mut self, config: SearchConfig, signal: EngineSignal) -> SearchResult;
}

enum ThreadCmd {
    Position(Board),
    Move(Move),
    Option(String, Option<String>),
    Search(SearchConfig, EngineSignal, Sender<SearchResult>),
    Quit,
}

fn engine_thread<E: Engine>(mut engine: E, recv: Receiver<ThreadCmd>) {
    loop {
        match recv.recv().unwrap() {
            ThreadCmd::Position(pos) => {
                engine.new_position(pos);
            }
            ThreadCmd::Move(m) => {
                engine.make_move(m);
            }
            ThreadCmd::Search(config, signal, res) => {
                let r = engine.search(config, signal);
                res.send(r).unwrap()
            }
            ThreadCmd::Option(n, v) => {
                engine.set_option(n, v);
            }
            ThreadCmd::Quit => return,
        }
    }
}

struct PendingSearch {
    done: Receiver<SearchResult>,
    cmd: EngineCommand,
}

/// Struct which implements UCI protocol for chess engines implementing the Engine trait
pub struct UciEngine<E: Engine, F: FnMut(Msg)> {
    board: Board,
    engine_thread: JoinHandle<()>,
    cmd: Sender<ThreadCmd>,
    pending_search: Option<PendingSearch>,
    incomming: Receiver<Cmd>,
    cb: F,
    move_gen: MoveGenerator,
    _marker: PhantomData<E>,
}

impl<E: Engine + Send + 'static, F: Fn(Msg)> UciEngine<E, F> {
    pub fn new(e: E, incomming: Receiver<Cmd>, cb: F) -> Self {
        let (send, recv) = crossbeam_channel::bounded(2);
        let engine_thread = thread::spawn(|| engine_thread(e, recv));
        let move_gen = MoveGenerator::new();
        UciEngine {
            board: Board::empty(),
            engine_thread,
            cmd: send,
            pending_search: None,
            cb,
            incomming,
            move_gen,
            _marker: PhantomData,
        }
    }

    pub fn run(mut self) {
        loop {
            if let Some(pending) = self.pending_search.as_ref() {
                crossbeam_channel::select! {
                    recv(pending.done) -> done => {
                        let done = done.unwrap();
                        if let Some(r#move) = done.r#move {
                            let r#move = r#move.into();
                            let ponder = done.ponder.map(UciMove::from);
                            self.pending_search = None;
                            (self.cb)(Msg::BestMove{ r#move, ponder });
                        }
                    }
                    recv(pending.cmd.info) -> info => {
                        if let Ok(info) = info{
                            (self.cb)(Msg::Info(info));
                        }
                    }
                    recv(self.incomming) -> cmd => {
                        if self.handle_cmd(cmd.unwrap()) {
                           break
                        }
                    }
                }
            } else {
                let cmd = self.incomming.recv().unwrap();
                if self.handle_cmd(cmd) {
                    break;
                }
            }
        }
        self.engine_thread.join().unwrap();
    }

    fn handle_cmd(&mut self, cmd: Cmd) -> bool {
        match cmd {
            Cmd::Uci => {
                (self.cb)(Msg::Id(Id::Name(E::NAME.to_owned())));
                (self.cb)(Msg::Id(Id::Author(E::AUTHOR.to_owned())));
                if let Some(x) = E::version() {
                    (self.cb)(Msg::Id(Id::Version(x)));
                }

                for option in E::options() {
                    (self.cb)(Msg::Option(option));
                }

                (self.cb)(Msg::UciOk);
            }
            Cmd::Debug(_) => {}
            Cmd::IsReady => (self.cb)(Msg::ReadyOk),
            Cmd::SetOption { name, value } => {
                self.cmd.send(ThreadCmd::Option(name, value)).unwrap();
            }
            Cmd::NewGame => {}
            Cmd::Position(pos) => {
                self.board = match pos.position {
                    StartOrFen::StartPosition => Board::start_position(EndChain),
                    StartOrFen::Fen(ref x) => {
                        let Ok(board) = Board::from_fen(x, EndChain) else {
                            return false;
                        };
                        board
                    }
                };
                self.cmd
                    .send(ThreadCmd::Position(self.board.clone()))
                    .unwrap();
                for m in pos.moves {
                    let Some(m) = m.to_move(&self.move_gen, &self.board) else {
                        return false;
                    };
                    self.board.make_move(m);
                    self.cmd.send(ThreadCmd::Move(m)).unwrap();
                }
            }
            Cmd::Go(cfg) => {
                let mut config = SearchConfig::default();
                for c in cfg {
                    match c {
                        super::GoCmd::SearchMoves(m) => {
                            let mut moves = Vec::new();
                            for m in m {
                                let Some(m) = m.to_move(&self.move_gen, &self.board) else {
                                    return false;
                                };
                                self.board.make_move(m);
                                moves.push(m);
                            }
                            config.moves = Some(moves);
                        }
                        super::GoCmd::Ponder => {
                            config.ponder = true;
                        }
                        super::GoCmd::WhiteTime(t) => {
                            config.wtime = Some(t);
                        }
                        super::GoCmd::BlackTime(t) => {
                            config.btime = Some(t);
                        }
                        super::GoCmd::WhiteInc(t) => {
                            config.winc = Some(t);
                        }
                        super::GoCmd::BlackInc(t) => {
                            config.binc = Some(t);
                        }
                        super::GoCmd::MovesToGo(m) => {
                            config.movestogo = Some(m);
                        }
                        super::GoCmd::Depth(d) => config.depth = Some(d),
                        super::GoCmd::Nodes(n) => {
                            config.nodes = Some(n);
                        }
                        super::GoCmd::Mate(m) => {
                            config.mate = Some(m);
                        }
                        super::GoCmd::MoveTime(t) => {
                            config.movetime = Some(Duration::from_millis(t as u64));
                            config.infinite = false;
                        }
                        super::GoCmd::Infinite => {
                            config.infinite = true;
                            config.movetime = None;
                        }
                    }
                }
                let (cmd, sig) = EngineCommand::create();
                let (send, recv) = crossbeam_channel::bounded(0);
                self.cmd.send(ThreadCmd::Search(config, sig, send)).unwrap();
                self.pending_search = Some(PendingSearch { done: recv, cmd });
            }
            Cmd::Stop => {
                if let Some(ref cmd) = self.pending_search {
                    cmd.cmd.stop();
                }
            }
            Cmd::PonderHit => {}
            Cmd::Quit => {
                if let Some(ref pending) = self.pending_search {
                    pending.cmd.stop();
                }
                self.cmd.send(ThreadCmd::Quit).unwrap();
                return true;
            }
        }
        false
    }
}

fn spawn_stdin_thread(send: Sender<Cmd>) -> JoinHandle<()> {
    thread::spawn(move || {
        let i = std::io::stdin().lock();
        for line in i.lines() {
            let line = line.unwrap();
            if let Some(cmd) = Cmd::from_line(&line) {
                send.send(cmd).unwrap();
            } else {
                eprintln!("invalid command: {}", line);
            }
        }
    })
}

pub fn run<E: Engine + Send + 'static>(engine: E) {
    let (send, recv) = bounded(8);
    spawn_stdin_thread(send);
    let engine = UciEngine::new(engine, recv, |x| {
        println!("{}", x);
    });
    engine.run()
}
