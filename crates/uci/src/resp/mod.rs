use crate::UciMove;
use core::fmt;
use nom::Finish;
use std::str::FromStr;

mod parse;

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Response {
    UciOk,
    ReadyOk,
    Id(ResponseId),
    BestMove {
        r#move: UciMove,
        ponder: Option<UciMove>,
    },
    Info(Vec<ResponseInfo>),
    Option(ResponseOption),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UciResponseError {
    at: String,
}

impl fmt::Display for UciResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn floor_char_boundary(i: &str, mut offset: usize) -> usize {
            while offset != 0 && !i.is_char_boundary(offset) {
                offset -= 1;
            }
            offset
        }

        write!(f, "failed to parse UCI response at: ")?;
        let boundry = floor_char_boundary(&self.at, self.at.len().min(40));
        write!(f, "{}", &self.at[..boundry])?;
        if boundry < self.at.len() {
            write!(f, "...")?;
        }
        Ok(())
    }
}

impl Response {
    pub fn from_line(l: &str) -> Result<Self, UciResponseError> {
        match parse::parse(l).finish() {
            Ok((_, x)) => Ok(x),
            Err(e) => Err(UciResponseError {
                at: e.input.to_owned(),
            }),
        }
    }
}

impl FromStr for Response {
    type Err = UciResponseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_line(s)
    }
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Response::UciOk => write!(f, "uciok"),
            Response::ReadyOk => write!(f, "readyok"),
            Response::Id(x) => write!(f, "id {x}"),
            Response::BestMove { r#move, ponder } => {
                write!(f, "bestmove {move}")?;
                if let Some(p) = ponder {
                    write!(f, " ponder {p}")?;
                }
                Ok(())
            }
            Response::Info(info) => {
                write!(f, "info")?;
                for i in info {
                    write!(f, " {i}")?;
                }
                Ok(())
            }
            Response::Option(x) => write!(f, "option {x}"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ResponseId {
    Name(String),
    Author(String),
}

impl fmt::Display for ResponseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseId::Name(x) => write!(f, "name {x}"),
            ResponseId::Author(x) => write!(f, "author {x}"),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ResponseInfo {
    Depth(u32),
    SelectiveDepth(u32),
    Time(u64),
    Nodes(u64),
    Pv(Vec<UciMove>),
    MultiPv(u32),
    Score(ResponseScore),
    CurrMove(UciMove),
    CurrMoveNumber(u32),
    Hashfull(u16),
    Nps(u64),
    TbHits(u64),
    SbHits(u64),
    CpuLoad(u16),
    String(String),
    Refutation(Vec<UciMove>),
    CurrLine {
        cpu: Option<u32>,
        moves: Vec<UciMove>,
    },
}

impl fmt::Display for ResponseInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResponseInfo::Depth(x) => write!(f, "depth {x}"),
            ResponseInfo::SelectiveDepth(x) => write!(f, "seldepth {x}"),
            ResponseInfo::Time(x) => write!(f, "time {x}"),
            ResponseInfo::Nodes(x) => write!(f, "nodes {x}"),
            ResponseInfo::Pv(moves) => {
                write!(f, "pv")?;
                for m in moves {
                    write!(f, " {m}")?;
                }
                Ok(())
            }
            ResponseInfo::MultiPv(x) => write!(f, "multipv {x}"),
            ResponseInfo::Score(x) => write!(f, "score {x}"),
            ResponseInfo::CurrMove(x) => write!(f, "currmove {x}"),
            ResponseInfo::CurrMoveNumber(x) => write!(f, "currmovenumber {x}"),
            ResponseInfo::Hashfull(x) => write!(f, "hashfull {x}"),
            ResponseInfo::Nps(x) => write!(f, "nps {x}"),
            ResponseInfo::TbHits(x) => write!(f, "tbhits {x}"),
            ResponseInfo::SbHits(x) => write!(f, "sbhits {x}"),
            ResponseInfo::CpuLoad(x) => write!(f, "cpuload {x}"),
            ResponseInfo::String(x) => write!(f, "string {x}"),
            ResponseInfo::Refutation(moves) => {
                write!(f, "refutation")?;
                for m in moves {
                    write!(f, " {m}")?;
                }
                Ok(())
            }
            ResponseInfo::CurrLine { cpu, moves } => {
                write!(f, "currline")?;
                if let Some(cpu) = cpu {
                    write!(f, " {cpu}")?;
                }
                for m in moves {
                    write!(f, " {m}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ResponseScore {
    pub mate: Option<u32>,
    pub cp: Option<i64>,
    pub bound: ResponseBound,
}

impl fmt::Display for ResponseScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut wrote = false;
        if let Some(m) = self.mate {
            wrote = true;
            write!(f, "mate {m}")?;
        }

        if let Some(s) = self.cp {
            if wrote {
                write!(f, " ")?;
            }
            wrote = true;
            write!(f, "cp {s}")?;
        }
        match self.bound {
            ResponseBound::Exact => {}
            ResponseBound::Upperbound => {
                if wrote {
                    write!(f, " ")?;
                }
                write!(f, "upperbound")?;
            }
            ResponseBound::Lowerbound => {
                if wrote {
                    write!(f, " ")?;
                }
                write!(f, "lowerbound")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum ResponseBound {
    Exact,
    Upperbound,
    Lowerbound,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct ResponseOption {
    pub name: String,
    pub kind: OptionKind,
}

impl fmt::Display for ResponseOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "name {}", self.name)?;
        match self.kind {
            OptionKind::String { ref default } => {
                write!(f, " type string")?;
                if let Some(d) = default.as_ref() {
                    write!(f, " default {d}")?;
                }
                Ok(())
            }
            OptionKind::Spin { default, min, max } => {
                write!(f, " type spin")?;
                if let Some(d) = default {
                    write!(f, " default {d}")?;
                }
                if let Some(m) = min {
                    write!(f, " min {m}")?;
                }
                if let Some(m) = max {
                    write!(f, " max {m}")?;
                }
                Ok(())
            }
            OptionKind::Button => write!(f, " type button"),
            OptionKind::Check { default } => {
                write!(f, " type check")?;
                if let Some(d) = default {
                    write!(f, " default {d}")?;
                }
                Ok(())
            }
            OptionKind::Combo {
                ref options,
                ref default,
            } => {
                write!(f, " type combo")?;
                if let Some(d) = default.as_ref() {
                    write!(f, " default {d}")?;
                }
                for v in options {
                    write!(f, " var {v}")?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum OptionKind {
    String {
        default: Option<String>,
    },
    Spin {
        default: Option<i64>,
        min: Option<i64>,
        max: Option<i64>,
    },
    Button,
    Check {
        default: Option<bool>,
    },
    Combo {
        options: Vec<String>,
        default: Option<String>,
    },
}
