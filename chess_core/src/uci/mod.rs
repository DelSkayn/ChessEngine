//! Utilities for implementing the UCI protocol

use crate::{
    board::{Board, EndChain},
    engine::{Engine, Info, OptionKind, OptionValue, ShouldRun},
    gen::{gen_type, MoveGenerator},
    Move, Square,
};

use anyhow::{anyhow, bail, ensure, Result};

use std::{
    collections::HashMap,
    fmt,
    io::{self, BufRead},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
};

enum Cmd {
    SetOption(String, OptionValue),
    SetBoard(Board),
    MakeMove(Move),
    Go,
}

enum Response {
    Done(Option<Move>),
}

struct UciMove(Move);

impl fmt::Display for UciMove {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.0.from(), self.0.to())?;
        if self.0.ty() == Move::TYPE_PROMOTION {
            match self.0.promotion_piece() {
                Move::PROMOTION_QUEEN => {
                    write!(f, "q")?;
                }
                Move::PROMOTION_ROOK => {
                    write!(f, "r")?;
                }
                Move::PROMOTION_BISHOP => {
                    write!(f, "b")?;
                }
                Move::PROMOTION_KNIGHT => {
                    write!(f, "n")?;
                }
                _ => unreachable!(),
            }
        }
        Ok(())
    }
}

pub struct ThreadManger {
    send: Sender<Cmd>,
    recv: Receiver<Response>,
    handle: Arc<AtomicBool>,
}

/// A struct for running an engine on a different thread.
impl ThreadManger {
    pub fn new<E, F>(mut engine: E, mut on_info: F) -> Self
    where
        E: Engine + Send + 'static,
        F: FnMut(Info) -> ShouldRun + Send + 'static,
    {
        let (send, recv) = mpsc::channel();
        let (send_b, recv_b) = mpsc::channel();

        let handle = Arc::new(AtomicBool::new(false));
        let handle_clone = handle.clone();

        thread::spawn(move || {
            while let Some(cmd) = recv_b.recv().ok() {
                match cmd {
                    Cmd::SetOption(name, value) => {
                        engine.set_option(name, value);
                    }
                    Cmd::SetBoard(x) => {
                        engine.set_board(x);
                    }
                    Cmd::MakeMove(m) => {
                        engine.make_move(m);
                    }
                    Cmd::Go => {
                        let new_on_info = |info| {
                            let mut res = on_info(info);

                            if !handle_clone.load(Ordering::Acquire) {
                                res = ShouldRun::Stop
                            }
                            res
                        };

                        let m = engine.go(new_on_info, || {
                            if handle_clone.load(Ordering::Acquire) {
                                ShouldRun::Continue
                            } else {
                                ShouldRun::Stop
                            }
                        });
                        send.send(Response::Done(m)).unwrap();
                    }
                }
            }
        });

        ThreadManger {
            send: send_b,
            recv,
            handle,
        }
    }

    pub fn start(&self) {
        self.handle.store(true, Ordering::Release);
        self.send.send(Cmd::Go).unwrap();
    }

    pub fn stop(&self) -> Option<Move> {
        self.handle.store(false, Ordering::Release);
        match self.recv.recv().unwrap() {
            Response::Done(x) => x,
        }
    }

    pub fn set_option(&self, name: String, value: OptionValue) {
        self.send.send(Cmd::SetOption(name, value)).unwrap();
    }

    pub fn set_board(&self, b: Board) {
        self.send.send(Cmd::SetBoard(b)).unwrap();
    }

    pub fn make_move(&self, m: Move) {
        self.send.send(Cmd::MakeMove(m)).unwrap();
    }
}

/// An generic implementation of the UCI protocol.
pub struct Uci {
    board: Board,
    debug_mode: bool,
    hash_size: usize,
    manager: ThreadManger,
    options: HashMap<String, OptionKind>,
    name: &'static str,
    author: &'static str,
}

pub fn split_once(s: &str) -> (&str, &str) {
    if let Some(at) = s.find(" ") {
        let (a, b) = s.split_at(at);
        (a, &b[1..])
    } else {
        (s, "")
    }
}

impl Uci {
    pub fn new<E: Engine + Send>(engine: E) -> Self {
        let options = engine.options();
        Uci {
            board: Board::start_position(EndChain),
            debug_mode: false,
            hash_size: 16,
            manager: ThreadManger::new(engine, Self::handle_info),
            options,
            name: E::NAME,
            author: E::AUTHOR,
        }
    }

    fn handle_info(info: Info) -> ShouldRun {
        match info {
            Info::BestMove { value, .. } => println!("info score cp {}", value),
            Info::Round => {}
            Info::Depth(x) => println!("info depth {}", x),
            Info::Nodes(x) => println!("info nodes {}", x),
            Info::NodesPerSec(x) => println!("info nps {}", x),
            Info::TransHit(x) => println!("info tbhits {}", x),
            Info::Pv(x) => {
                print!("info pv ");
                x.iter().for_each(|x| {
                    let m = UciMove(*x);
                    print!("{} ", m);
                });
                println!()
            }
            Info::Debug(x) => {
                print!("debug ");
                println!("{}", x);
            }
        }
        ShouldRun::Continue
    }

    pub fn start(&mut self) -> Result<()> {
        let stdin = io::stdin();
        let mut buffer = String::new();
        let mut handle = stdin.lock();

        handle.read_line(&mut buffer)?;
        let line = buffer.trim();

        ensure!(line == "uci", "Protocol did not start with 'uci' command");

        print!("id name ");
        println!("{}", self.name);
        print!("id author ");
        println!("{}", self.author);

        println!("uciok");
        loop {
            buffer.clear();
            handle.read_line(&mut buffer)?;
            let (command, rest) = split_once(buffer.trim());

            match command {
                "isready" => println!("readyok"),
                "debug" => match rest {
                    "on" => self.debug_mode = true,
                    "off" => self.debug_mode = false,
                    _ => bail!("misformed command"),
                },
                "setoption" => {
                    let (name, rest) = split_once(rest);
                    ensure!(name == "name", "invalid command");
                    let (option, rest) = split_once(rest);
                    match option {
                        "Hash" => {
                            let (value, rest) = split_once(rest);
                            ensure!(value == "value", "misformed command");
                            self.hash_size = rest.parse()?;
                        }
                        _ => {
                            bail!("invalid option");
                        }
                    }
                }
                "go" => self.manager.start(),
                "stop" => {
                    if let Some(x) = self.manager.stop() {
                        println!("bestmove {}", UciMove(x));
                    } else {
                        bail!("no move found by engine!");
                    }
                }
                "ucinewgame" => {}
                "position" => self.parse_position(rest)?,
                "quit" => return Ok(()),
                "" => {}
                _ => println!("invalid command"),
            }
        }
    }

    pub fn parse_position(&self, arg: &str) -> Result<()> {
        let mut board;

        let rem = match arg.split_once(" ") {
            Some(("startpos", rem)) => {
                board = Board::start_position(EndChain);
                self.manager.set_board(board.clone());
                rem
            }
            Some(("fen", rem)) => match rem.find("moves") {
                Some(x) => {
                    board = Board::from_fen(&rem[..x], EndChain)?;
                    self.manager.set_board(board.clone());
                    &rem[x..]
                }
                None => {
                    board = Board::from_fen(rem, EndChain)?;
                    self.manager.set_board(board.clone());
                    return Ok(());
                }
            },
            None => {
                if arg.starts_with("startpos") {
                    board = Board::start_position(EndChain);
                    self.manager.set_board(board.clone());
                    return Ok(());
                } else {
                    println!("invalid command");
                    return Ok(());
                }
            }
            _ => bail!("invalid position command"),
        };

        let mut iterator = rem.split_whitespace();
        ensure!(iterator.next() == Some("moves"));

        let move_gen = MoveGenerator::new();
        let mut move_buffer = Vec::new();

        for m in iterator {
            let from = Square::from_name(&m[0..2]).ok_or_else(|| anyhow!("invalid square"))?;
            let to = Square::from_name(&m[2..4]).ok_or_else(|| anyhow!("invalid square"))?;
            move_gen.gen_moves::<gen_type::All, _, _>(&board, &mut move_buffer);
            let m = move_buffer
                .iter()
                .copied()
                .find(|m| m.to() == to && m.from() == from)
                .ok_or_else(|| anyhow!("invalid move"))?;
            board.make_move(m);
            self.manager.make_move(m);
        }
        Ok(())
    }
}
