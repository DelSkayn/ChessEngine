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
    Go {
        searchmoves: Option<Vec<UciMove>>,
        ponder: bool,
        wtime: Option<i64>,
        btime: Option<i64>,
        winc: Option<i32>,
        binc: Option<i32>,
        moves_to_go: Option<u32>,
        depth: Option<u32>,
        nodes: Option<u64>,
        mate: Option<u32>,
        movetime: Option<u64>,
        infinite: bool,
    },
    Stop,
    PonderHit,
    Quit,
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
            Request::Go {
                searchmoves,
                ponder,
                wtime,
                btime,
                winc,
                binc,
                moves_to_go,
                depth,
                nodes,
                mate,
                movetime,
                infinite,
            } => {
                write!(f, "go")?;
                if let Some(x) = searchmoves {
                    write!(f, " searchmoves")?;
                    for m in x.iter() {
                        write!(f, " {m}")?;
                    }
                }
                if *ponder {
                    write!(f, " ponder")?;
                }
                if let Some(wtime) = wtime {
                    write!(f, " wtime {wtime}")?;
                }
                if let Some(btime) = btime {
                    write!(f, " btime {btime}")?;
                }
                if let Some(winc) = winc {
                    write!(f, " winc {winc}")?;
                }
                if let Some(binc) = binc {
                    write!(f, " binc {binc}")?;
                }
                if let Some(m) = moves_to_go {
                    write!(f, " movestogo {m}")?;
                }
                if let Some(d) = depth {
                    write!(f, " depth {d}")?;
                }
                if let Some(n) = nodes {
                    write!(f, " nodes {n}")?;
                }
                if let Some(m) = mate {
                    write!(f, " mate {m}")?;
                }
                if let Some(m) = movetime {
                    write!(f, " movetime {m}")?;
                }
                if *infinite {
                    write!(f, " infinite")?;
                }
                Ok(())
            }
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
