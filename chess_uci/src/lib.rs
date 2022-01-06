//! Utilities for implementing the UCI protocol
#![allow(dead_code)]

use std::{
    collections::HashMap,
    fmt,
    io::{self, BufRead},
    time::Duration,
};

use anyhow::{anyhow, bail, ensure, Result};
use chess_core::{
    board::{Board, EndChain},
    engine::{Engine, EngineLimit, EngineThread, Info, OptionKind, Response, ThreadController},
    gen::{gen_type, MoveGenerator},
    Move, Player, Square,
};
use crossbeam_channel::select;

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

/// An generic implementation of the UCI protocol.
pub struct Uci {
    board: Board,
    debug_mode: bool,
    manager: EngineThread,
    options: HashMap<String, OptionKind>,
    name: &'static str,
    author: &'static str,
    running: bool,
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
    pub fn new<E: Engine<ThreadController> + Send>(engine: E) -> Self {
        let options = engine.options();
        Uci {
            board: Board::start_position(EndChain),
            debug_mode: false,
            manager: EngineThread::new(engine),
            options,
            name: E::NAME,
            author: E::AUTHOR,
            running: true,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let (io_send, io_recv) = crossbeam_channel::bounded(8);

        std::thread::spawn(move || {
            let stdin = io::stdin();
            let mut handle = stdin.lock();
            loop {
                let mut buffer = String::new();
                handle.read_line(&mut buffer).unwrap();
                io_send.send(buffer).ok();
            }
        });

        let line = io_recv.recv()?;

        ensure!(
            line.trim() == "uci",
            "Protocol did not start with 'uci' command"
        );

        print!("id name ");
        println!("{}", self.name);
        print!("id author ");
        println!("{}", self.author);

        println!("uciok");
        while self.running {
            select! {
                recv(io_recv) -> line => self.handle_line(line?)?,
                recv(self.manager.recv()) -> resp => self.handle_response(resp?)?,
            }
        }
        Ok(())
    }

    fn handle_response(&mut self, resp: Response) -> Result<()> {
        match resp {
            Response::Info(info) => match info {
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
            },
            Response::Done(x) => {
                if let Some(m) = x {
                    println!("bestmove {}", UciMove(m))
                }
            }
        }
        Ok(())
    }

    fn handle_line(&mut self, line: String) -> Result<()> {
        let (command, rest) = split_once(line.trim());

        match command {
            "isready" => {
                println!("readyok");
            }
            "debug" => match rest {
                "on" => self.debug_mode = true,
                "off" => self.debug_mode = false,
                _ => bail!("misformed command"),
            },
            "go" => self.parse_go(rest)?,
            "stop" => self.manager.stop(),
            "ucinewgame" => {}
            "position" => self.parse_position(rest)?,
            "quit" => {
                self.running = false;
            }
            "" => {}
            _ => {
                println!("invalid command");
            }
        }

        Ok(())
    }

    pub fn parse_go(&self, arg: &str) -> Result<()> {
        let mut iter = arg.split_whitespace();
        let mut time_limit = None;
        let mut limits = EngineLimit::none();
        while let Some(cmd) = iter.next() {
            match cmd {
                "wtime" => {
                    let time = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing time number"))?
                        .parse()?;
                    if self.board.state.player == Player::White {
                        time_limit = Some(Duration::from_millis(time));
                    }
                }
                "btime" => {
                    let time = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing time number"))?
                        .parse()?;
                    if self.board.state.player == Player::Black {
                        time_limit = Some(Duration::from_millis(time));
                    }
                }
                "depth" => {
                    let depth = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing depth number"))?
                        .parse()?;
                    limits.depth = Some(depth);
                }
                "nodes" => {
                    let nodes = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing nodes number"))?
                        .parse()?;
                    limits.nodes = Some(nodes);
                }
                "movetime" => {
                    let time = iter
                        .next()
                        .ok_or_else(|| anyhow!("missing nodes number"))?
                        .parse()?;
                    limits.time = Some(Duration::from_millis(time));
                }
                "infinite" => {
                    self.manager.start(None, EngineLimit::none());
                    return Ok(());
                }
                _ => {}
            }
        }
        self.manager.start(time_limit, limits);
        Ok(())
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
