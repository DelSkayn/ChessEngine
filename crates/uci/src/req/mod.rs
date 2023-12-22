use crate::r#move::UciMove;
use common::board::Board;
use core::fmt;
use nom::Finish;
use std::str::FromStr;

mod parse;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Request {
    Uci,
    Debug(bool),
    IsReady,
    UciNewGame,
    SetOption {
        name: String,
        value: Option<OptionValue>,
    },
    Position {
        fen: Option<Box<Board>>,
        moves: Vec<UciMove>,
    },
    Go(GoRequest),
    Stop,
    PonderHit,
    Quit,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct GoRequest {
    pub searchmoves: Option<Vec<UciMove>>,
    pub ponder: bool,
    pub wtime: Option<i64>,
    pub btime: Option<i64>,
    pub winc: Option<i32>,
    pub binc: Option<i32>,
    pub moves_to_go: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u64>,
    pub mate: Option<u32>,
    pub movetime: Option<u64>,
    pub infinite: bool,
}

impl fmt::Display for GoRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(x) = self.searchmoves.as_ref() {
            write!(f, " searchmoves")?;
            for m in x.iter() {
                write!(f, " {m}")?;
            }
        }
        if self.ponder {
            write!(f, " ponder")?;
        }
        if let Some(wtime) = self.wtime {
            write!(f, " wtime {wtime}")?;
        }
        if let Some(btime) = self.btime {
            write!(f, " btime {btime}")?;
        }
        if let Some(winc) = self.winc {
            write!(f, " winc {winc}")?;
        }
        if let Some(binc) = self.binc {
            write!(f, " binc {binc}")?;
        }
        if let Some(m) = self.moves_to_go {
            write!(f, " movestogo {m}")?;
        }
        if let Some(d) = self.depth {
            write!(f, " depth {d}")?;
        }
        if let Some(n) = self.nodes {
            write!(f, " nodes {n}")?;
        }
        if let Some(m) = self.mate {
            write!(f, " mate {m}")?;
        }
        if let Some(m) = self.movetime {
            write!(f, " movetime {m}")?;
        }
        if self.infinite {
            write!(f, " infinite")?;
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum OptionValue {
    String(String),
    Spin(i64),
    Check(bool),
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Request::Uci => write!(f, "uci"),
            Request::Debug(x) => {
                write!(f, "debug ")?;
                if *x {
                    write!(f, "on")
                } else {
                    write!(f, "off")
                }
            }
            Request::IsReady => write!(f, "isready"),
            Request::UciNewGame => write!(f, "ucinewgame"),
            Request::SetOption { name, value } => {
                write!(f, "setoption name {}", name)?;
                match value {
                    Some(OptionValue::Spin(x)) => write!(f, " value {}", x),
                    Some(OptionValue::String(x)) => write!(f, " value {}", x),
                    Some(OptionValue::Check(x)) => write!(f, " value {}", x),
                    None => Ok(()),
                }
            }
            Request::Position { fen, moves } => {
                write!(f, "position ")?;
                if let Some(x) = fen {
                    write!(f, "fen {}", x.to_fen())?;
                } else {
                    write!(f, "startposition")?;
                }
                if moves.is_empty() {
                    Ok(())
                } else {
                    write!(f, " moves")?;
                    for m in moves.iter() {
                        write!(f, " {m}")?;
                    }
                    Ok(())
                }
            }
            Request::Go(x) => write!(f, "go {x}"),
            Request::Stop => write!(f, "stop"),
            Request::PonderHit => write!(f, "ponderhit"),
            Request::Quit => write!(f, "quit"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UciRequestError {
    at: String,
}

impl fmt::Display for UciRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn floor_char_boundary(i: &str, mut offset: usize) -> usize {
            while offset != 0 && !i.is_char_boundary(offset) {
                offset -= 1;
            }
            offset
        }

        write!(f, "failed to parse UCI request at: ")?;
        let boundry = floor_char_boundary(&self.at, self.at.len().min(40));
        write!(f, "{}", &self.at[..boundry])?;
        if boundry < self.at.len() {
            write!(f, "...")?;
        }
        Ok(())
    }
}

impl Request {
    pub fn from_line(l: &str) -> Result<Self, UciRequestError> {
        match parse::parse(l).finish() {
            Ok((_, x)) => Ok(x),
            Err(e) => Err(UciRequestError {
                at: e.input.to_owned(),
            }),
        }
    }
}

impl FromStr for Request {
    type Err = UciRequestError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_line(s)
    }
}
