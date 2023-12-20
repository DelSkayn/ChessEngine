use std::{convert::TryInto, fmt, str::FromStr, time::Duration};

use thiserror::Error;

use crate::{
    board::MoveChain,
    gen::{gen_type, MoveGenerator},
    mov::Promotion,
    Board, Move, Square,
};
pub use crossbeam_channel;

#[derive(Error, Debug)]
pub enum UciParseError {
    #[error("Uci line was empty")]
    EmptyLine,
    #[error("Recieved invalid command: `{0}`")]
    InvalidCommand(String),
    #[error("Recieved unexpected input, expected on of {expected:?}, found `{found}`")]
    Unexpected {
        expected: &'static [&'static str],
        found: String,
    },
}

#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UciMove {
    from: Square,
    to: Square,
    promotion: Option<Promotion>,
}

impl UciMove {
    pub fn to_move<C: MoveChain>(&self, gen: &MoveGenerator, board: &Board<C>) -> Option<Move> {
        let mut moves = Vec::<Move>::new();
        gen.gen_moves::<gen_type::All, _, _>(board, &mut moves);
        moves.into_iter().find(|&m| {
            m.from() == self.from && m.to() == self.to && m.get_promotion() == self.promotion
        })
    }
}

#[derive(Debug)]
pub struct InvalidMove;

impl fmt::Display for InvalidMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid move")
    }
}

impl From<Move> for UciMove {
    fn from(value: Move) -> Self {
        UciMove {
            from: value.from(),
            to: value.to(),
            promotion: value.get_promotion(),
        }
    }
}

impl FromStr for UciMove {
    type Err = InvalidMove;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 4 {
            return Err(InvalidMove);
        }
        if !s.is_char_boundary(2) || !s.is_char_boundary(4) {
            return Err(InvalidMove);
        }
        let Some(from) = Square::from_name(&s[..2]) else {
            return Err(InvalidMove);
        };
        let Some(to) = Square::from_name(&s[2..4]) else {
            return Err(InvalidMove);
        };

        let promotion = if s.len() > 4 && s.is_char_boundary(5) {
            let m = match &s[4..5] {
                "q" => Promotion::Queen,
                "r" => Promotion::Rook,
                "k" => Promotion::Knight,
                "b" => Promotion::Bishop,
                _ => return Err(InvalidMove),
            };
            Some(m)
        } else {
            None
        };

        Ok(UciMove {
            from,
            to,
            promotion,
        })
    }
}

impl fmt::Display for UciMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.from, self.to)?;
        if let Some(p) = self.promotion {
            write!(f, "{}", p)?;
        }
        Ok(())
    }
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RegisterCmd {
    Name(String),
    Code(String),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StartOrFen {
    StartPosition,
    Fen(String),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PositionCmd {
    pub position: StartOrFen,
    pub moves: Vec<UciMove>,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GoCmd {
    SearchMoves(Vec<UciMove>),
    Ponder,
    WhiteTime(Duration),
    BlackTime(Duration),
    WhiteInc(Duration),
    BlackInc(Duration),
    MovesToGo(u32),
    Depth(u32),
    Nodes(u64),
    Mate(u32),
    MoveTime(u32),
    Infinite,
}

// Commands recieved from the gui
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Cmd {
    Uci,
    Debug(bool),
    IsReady,
    SetOption { name: String, value: Option<String> },
    NewGame,
    Position(PositionCmd),
    Go(Vec<GoCmd>),
    Stop,
    PonderHit,
    Quit,
}

impl fmt::Display for Cmd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Cmd::Uci => write!(f, "uci"),
            Cmd::Debug(on) => {
                if on {
                    write!(f, "debug on")
                } else {
                    write!(f, "debug off")
                }
            }
            Cmd::IsReady => write!(f, "isready"),
            Cmd::SetOption {
                ref name,
                ref value,
            } => {
                write!(f, "setoption name {}", name)?;
                if let Some(ref v) = *value {
                    write!(f, " value {}", v)?;
                }
                Ok(())
            }
            Cmd::NewGame => write!(f, "ucinewgame"),
            Cmd::Position(ref p) => {
                write!(f, "position")?;
                match p.position {
                    StartOrFen::Fen(ref x) => write!(f, " fen {}", x)?,
                    StartOrFen::StartPosition => write!(f, " startpos")?,
                }
                if !p.moves.is_empty() {
                    write!(f, " moves")?;
                    for m in &p.moves {
                        write!(f, "{m}")?
                    }
                }
                Ok(())
            }
            Cmd::Go(ref x) => {
                write!(f, "go")?;
                for cmd in x {
                    match cmd {
                        GoCmd::SearchMoves(ref m) => {
                            write!(f, " searchmoves")?;
                            for mov in m {
                                write!(f, " {mov}")?;
                            }
                        }
                        GoCmd::Ponder => {
                            write!(f, " ponder")?;
                        }
                        GoCmd::WhiteTime(x) => {
                            let t: u32 = x.as_millis().try_into().expect("time to large");
                            write!(f, " wtime {t}")?;
                        }
                        GoCmd::BlackTime(x) => {
                            let t: u32 = x.as_millis().try_into().expect("time to large");
                            write!(f, " btime {t}")?;
                        }
                        GoCmd::WhiteInc(x) => {
                            let t: i32 = x.as_millis().try_into().expect("time to large");
                            write!(f, " winc {t}")?;
                        }
                        GoCmd::BlackInc(x) => {
                            let t: i32 = x.as_millis().try_into().expect("time to large");
                            write!(f, " binc {t}")?;
                        }
                        GoCmd::MovesToGo(x) => {
                            write!(f, " movestogo {x}")?;
                        }
                        GoCmd::Depth(x) => {
                            write!(f, " depth {x}")?;
                        }
                        GoCmd::Nodes(x) => {
                            write!(f, " nodes {x}")?;
                        }
                        GoCmd::Mate(x) => {
                            write!(f, " mate {x}")?;
                        }
                        GoCmd::MoveTime(x) => {
                            write!(f, " movetime {x}")?;
                        }
                        GoCmd::Infinite => {
                            write!(f, " infinite")?;
                        }
                    }
                }
                Ok(())
            }
            Cmd::Stop => write!(f, "stop"),
            Cmd::PonderHit => write!(f, "ponderhit"),
            Cmd::Quit => write!(f, "quit"),
        }
    }
}

impl Cmd {
    /// Parses a line of UCI commands, returns none if the command was invalid.
    pub fn from_line(line: &str) -> Option<Self> {
        let mut parts = line.split_whitespace();
        let Some(first) = parts.next() else {
            return None;
        };

        match first {
            "uci" => Some(Cmd::Uci),
            "debug" => {
                let mut enabled = true;
                if let Some(x) = parts.next() {
                    if x == "off" {
                        enabled = false;
                    }
                }
                Some(Cmd::Debug(enabled))
            }
            "isready" => Some(Cmd::IsReady),
            "setoption" => {
                if let Some(v) = parts.next() {
                    if v == "name" {
                        if let Some(name) = parts.next() {
                            let mut value = None;
                            if let Some(v) = parts.next() {
                                if v == "value" {
                                    if let Some(v) = parts.next() {
                                        value = Some(v.to_owned())
                                    }
                                }
                            };
                            return Some(Cmd::SetOption {
                                name: name.to_owned(),
                                value,
                            });
                        }
                    }
                }
                None
            }
            "ucinewgame" => Some(Cmd::NewGame),
            "position" => {
                let position = if let Some(x) = parts.next() {
                    if x == "startpos" {
                        StartOrFen::StartPosition
                    } else if x == "fen" {
                        let mut fen_string = String::new();
                        for i in 0..6 {
                            let Some(part) = parts.next() else {
                                return None;
                            };
                            fen_string.push_str(part);
                            if i != 5 {
                                fen_string.push(' ');
                            }
                        }
                        StartOrFen::Fen(fen_string)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };

                let moves = if let Some(x) = parts.next() {
                    if x == "moves" {
                        let mut moves = Vec::new();
                        for p in parts {
                            if let Ok(x) = UciMove::from_str(p) {
                                moves.push(x);
                            } else {
                                return None;
                            }
                        }
                        moves
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                Some(Cmd::Position(PositionCmd { position, moves }))
            }
            "go" => {
                let mut cmd = Vec::new();
                let mut cur = parts.next();
                while let Some(x) = cur {
                    match x {
                        "searchmoves" => {
                            let mut moves = Vec::new();
                            cur = parts.next();
                            while let Some(x) = cur {
                                if let Ok(m) = UciMove::from_str(x) {
                                    moves.push(m);
                                    cur = parts.next();
                                } else {
                                    break;
                                }
                            }
                            cmd.push(GoCmd::SearchMoves(moves));
                            continue;
                        }
                        "ponder" => {
                            cmd.push(GoCmd::Ponder);
                        }
                        "wtime" => {
                            let Some(time) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            let time = Duration::from_millis(time as u64);
                            cmd.push(GoCmd::WhiteTime(time));
                        }
                        "btime" => {
                            let Some(time) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            let time = Duration::from_millis(time as u64);
                            cmd.push(GoCmd::BlackTime(time));
                        }
                        "winc" => {
                            let Some(time) = parts.next().and_then(|x| i32::from_str(x).ok())
                            else {
                                return None;
                            };
                            if time > 0 {
                                let time = Duration::from_millis(time as u64);
                                cmd.push(GoCmd::WhiteInc(time));
                            }
                        }
                        "binc" => {
                            let Some(time) = parts.next().and_then(|x| i32::from_str(x).ok())
                            else {
                                return None;
                            };
                            if time > 0 {
                                let time = Duration::from_millis(time as u64);
                                cmd.push(GoCmd::BlackInc(time));
                            }
                        }
                        "movestogo" => {
                            let Some(moves) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            cmd.push(GoCmd::MovesToGo(moves));
                        }
                        "depth" => {
                            let Some(depth) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            cmd.push(GoCmd::Depth(depth));
                        }
                        "nodes" => {
                            let Some(nodes) = parts.next().and_then(|x| u64::from_str(x).ok())
                            else {
                                return None;
                            };
                            cmd.push(GoCmd::Nodes(nodes));
                        }
                        "mate" => {
                            let Some(depth) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            cmd.push(GoCmd::Mate(depth));
                        }
                        "movetime" => {
                            let Some(time) = parts.next().and_then(|x| u32::from_str(x).ok())
                            else {
                                return None;
                            };
                            cmd.push(GoCmd::MoveTime(time));
                        }
                        "infinite" => cmd.push(GoCmd::Infinite),
                        _ => return None,
                    }
                    cur = parts.next()
                }
                Some(Cmd::Go(cmd))
            }
            "stop" => Some(Cmd::Stop),
            "ponderhit" => Some(Cmd::PonderHit),
            "quit" => Some(Cmd::Quit),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Id {
    Name(String),
    Author(String),
    Version(Version),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ScoreKind {
    Cp,
    Mate,
    Lowerbound,
    UpperBound,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InfoMsg {
    Depth(u32),
    SelDepth(u32),
    Time(u32),
    Pv(Vec<UciMove>),
    MultiPv { order: u32, moves: Vec<UciMove> },
    Score { value: i32, kind: ScoreKind },
    CurrMove(UciMove),
    CurrMoveNumber(u32),
    HashFull(u16),
    Nps(u64),
    Nodes(u64),
    TbHits(u64),
    CpuLoad(u16),
    String(String),
    Refutation(Vec<UciMove>),
    CurrLine(Vec<UciMove>),
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OptionMsgType {
    Check {
        default: bool,
    },
    Spin {
        default: i64,
        min: Option<i64>,
        max: Option<i64>,
    },
    Combo {
        /// Options values
        options: Vec<String>,
        /// Index into options
        default: usize,
    },
    String {
        default: String,
    },
    Button,
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OptionMsg {
    pub name: String,
    pub r#type: OptionMsgType,
}

// Messages from the engine to the GUI
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Msg {
    Id(Id),
    UciOk,
    ReadyOk,
    BestMove {
        r#move: UciMove,
        ponder: Option<UciMove>,
    },
    Info(Vec<InfoMsg>),
    Option(OptionMsg),
}

impl fmt::Display for Msg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Msg::Id(Id::Name(ref x)) => write!(f, "id name {x}"),
            Msg::Id(Id::Author(ref x)) => write!(f, "id author {x}"),
            Msg::Id(Id::Version(ref x)) => {
                write!(f, "id version {} {} {}", x.major, x.minor, x.patch)
            }
            Msg::UciOk => write!(f, "uciok"),
            Msg::ReadyOk => write!(f, "readyok"),
            Msg::BestMove {
                ref r#move,
                ref ponder,
            } => {
                write!(f, "bestmove {}", r#move)?;
                if let Some(ponder) = ponder {
                    write!(f, " ponder {ponder}")?;
                }
                Ok(())
            }
            Msg::Info(ref x) => {
                write!(f, "info")?;
                for info in x {
                    match info {
                        InfoMsg::Depth(x) => write!(f, " depth {x}")?,
                        InfoMsg::SelDepth(x) => write!(f, " seldepth {x}")?,
                        InfoMsg::Time(x) => write!(f, " time {x}")?,
                        InfoMsg::Nodes(x) => write!(f, " nodes {x}")?,
                        InfoMsg::Pv(x) => {
                            write!(f, " pv")?;
                            for m in x {
                                write!(f, " {m}")?;
                            }
                        }
                        InfoMsg::MultiPv { moves, order } => {
                            write!(f, " multipv {order}")?;
                            for m in moves {
                                write!(f, " {m}")?;
                            }
                        }
                        InfoMsg::Score { value, kind } => {
                            let kind = match kind {
                                ScoreKind::Cp => "cp",
                                ScoreKind::Mate => "mate",
                                ScoreKind::Lowerbound => "lowerbound",
                                ScoreKind::UpperBound => "lowerbound",
                            };
                            write!(f, " score {kind} {value}")?;
                        }
                        InfoMsg::CurrMove(x) => write!(f, " currmove {x}")?,
                        InfoMsg::CurrMoveNumber(x) => write!(f, " currmovenumber {x}")?,
                        InfoMsg::HashFull(x) => write!(f, " hashfull {x}")?,
                        InfoMsg::Nps(x) => write!(f, " nps {x}")?,
                        InfoMsg::TbHits(x) => write!(f, " tbhits {x}")?,
                        InfoMsg::CpuLoad(x) => write!(f, " cpuload {x}")?,
                        InfoMsg::String(x) => write!(f, " string {x}")?,
                        InfoMsg::Refutation(x) => {
                            write!(f, " refutation")?;
                            for m in x {
                                write!(f, " {m}")?;
                            }
                        }
                        InfoMsg::CurrLine(x) => {
                            write!(f, " currline")?;
                            for m in x {
                                write!(f, " {m}")?;
                            }
                        }
                    }
                }
                Ok(())
            }
            Msg::Option(ref x) => {
                write!(f, "option name {}", x.name)?;
                match x.r#type {
                    OptionMsgType::String { ref default } => {
                        write!(f, " type string default {default}")
                    }
                    OptionMsgType::Button => write!(f, " type button"),
                    OptionMsgType::Check { default } => {
                        let default = if default { "true" } else { "false" };
                        write!(f, " type check default {default}")
                    }
                    OptionMsgType::Combo {
                        ref options,
                        default,
                    } => {
                        let default = &options[default];
                        write!(f, " type combo default {default}")?;
                        for o in options {
                            write!(f, "var {o}")?;
                        }
                        Ok(())
                    }
                    OptionMsgType::Spin { default, min, max } => {
                        write!(f, " type spin default {default}")?;
                        if let Some(min) = min {
                            write!(f, " min {min}")?;
                        }
                        if let Some(max) = max {
                            write!(f, " max {max}")?;
                        }
                        Ok(())
                    }
                }
            }
        }
    }
}

impl Msg {
    pub fn from_line(line: &str) -> Option<Msg> {
        let mut parts = line.split_whitespace();
        let msg = parts.next()?;
        match msg {
            "id" => {
                let Some(next) = parts.next() else {
                    return None;
                };
                match next {
                    "name" => Some(Msg::Id(Id::Name(parts.collect::<Vec<_>>().join(" ")))),
                    "author" => Some(Msg::Id(Id::Author(parts.collect::<Vec<_>>().join(" ")))),
                    "version" => {
                        let Some(major) = parts.next().and_then(|x| x.parse().ok()) else {
                            return None;
                        };
                        let Some(minor) = parts.next().and_then(|x| x.parse().ok()) else {
                            return None;
                        };
                        let Some(patch) = parts.next().and_then(|x| x.parse().ok()) else {
                            return None;
                        };
                        Some(Msg::Id(Id::Version(Version {
                            major,
                            minor,
                            patch,
                        })))
                    }
                    _ => None,
                }
            }
            "uciok" => Some(Msg::UciOk),
            "readyok" => Some(Msg::ReadyOk),
            "bestmove" => {
                let Some(m) = parts.next().and_then(|x| x.parse().ok()) else {
                    return None;
                };
                let mut ponder = None;
                if let Some(x) = parts.next() {
                    if x == "ponder" {
                        ponder = parts.next().and_then(|x| x.parse().ok());
                    }
                }
                Some(Msg::BestMove { r#move: m, ponder })
            }
            "info" => {
                let mut cur = parts.next();
                let mut info = Vec::new();
                while let Some(x) = cur {
                    match x {
                        "depth" => {
                            let d = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::Depth(d));
                        }
                        "seldepth" => {
                            let d = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::SelDepth(d));
                        }
                        "time" => {
                            let time = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::Time(time));
                        }
                        "nodes" => {
                            let nodes = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::Nodes(nodes));
                        }
                        "pv" => {
                            let mut moves = Vec::new();
                            cur = parts.next();
                            while let Some(x) = cur {
                                if let Ok(x) = UciMove::from_str(x) {
                                    moves.push(x);
                                    cur = parts.next();
                                } else {
                                    break;
                                }
                            }
                            info.push(InfoMsg::Pv(moves));
                            continue;
                        }
                        "multipv" => {
                            let order = parts.next().and_then(|x| x.parse().ok())?;
                            let mut moves = Vec::new();
                            cur = parts.next();
                            while let Some(x) = cur {
                                if let Ok(x) = UciMove::from_str(x) {
                                    moves.push(x);
                                    cur = parts.next();
                                } else {
                                    break;
                                }
                            }
                            info.push(InfoMsg::MultiPv { order, moves });
                            continue;
                        }
                        "score" => {
                            let kind = parts.next()?;
                            let kind = match kind {
                                "cp" => ScoreKind::Cp,
                                "mate" => ScoreKind::Mate,
                                "upperbound" => ScoreKind::UpperBound,
                                "lowerbound" => ScoreKind::Lowerbound,
                                _ => return None,
                            };
                            let value = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::Score { value, kind });
                        }
                        "curmove" => {
                            let m = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::CurrMove(m));
                        }
                        "curmovenumber" => {
                            let m = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::CurrMoveNumber(m));
                        }
                        "hashfull" => {
                            let full = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::HashFull(full));
                        }
                        "nps" => {
                            let nps = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::Nps(nps));
                        }
                        "tbhits" => {
                            let hits = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::TbHits(hits));
                        }
                        "cpuload" => {
                            let load = parts.next().and_then(|x| x.parse().ok())?;
                            info.push(InfoMsg::CpuLoad(load));
                        }
                        "string" => {
                            let s = parts.collect::<Vec<_>>().join(" ");
                            info.push(InfoMsg::String(s));
                            return Some(Msg::Info(info));
                        }
                        "refutation" => {
                            let mut moves = Vec::new();
                            cur = parts.next();
                            while let Some(x) = cur {
                                if let Ok(x) = UciMove::from_str(x) {
                                    moves.push(x);
                                    cur = parts.next();
                                } else {
                                    break;
                                }
                            }
                            info.push(InfoMsg::Refutation(moves));
                            continue;
                        }
                        "currline" => {
                            let mut moves = Vec::new();
                            cur = parts.next();
                            while let Some(x) = cur {
                                if let Ok(x) = UciMove::from_str(x) {
                                    moves.push(x);
                                    cur = parts.next();
                                } else {
                                    break;
                                }
                            }
                            info.push(InfoMsg::CurrLine(moves));
                            continue;
                        }
                        _ => return None,
                    }
                    cur = parts.next();
                }
                Some(Msg::Info(info))
            }
            "option" => {
                let name = parts.next()?;
                if name != "name" {
                    return None;
                }
                let mut name = String::new();
                loop {
                    if let Some(n) = parts.next() {
                        if n != "type" {
                            name.push_str(n);
                        } else {
                            break;
                        }
                    } else {
                        return None;
                    }
                }
                let ty = parts.next()?;
                let r#type = match ty {
                    "check" => {
                        let def = parts.next()?;
                        if def != "default" {
                            return None;
                        }
                        let def = parts.next()?;
                        let default = match def {
                            "true" => true,
                            "false" => false,
                            _ => return None,
                        };
                        OptionMsgType::Check { default }
                    }
                    "spin" => {
                        let mut default = None;
                        let mut max = None;
                        let mut min = None;
                        while let Some(x) = parts.next() {
                            match x {
                                "default" => {
                                    let d: i64 = parts.next().and_then(|x| x.parse().ok())?;
                                    default = Some(d);
                                }
                                "max" => {
                                    let m = parts.next().and_then(|x| x.parse().ok())?;
                                    max = Some(m);
                                }
                                "min" => {
                                    let m = parts.next().and_then(|x| x.parse().ok())?;
                                    min = Some(m);
                                }
                                _ => return None,
                            }
                        }
                        let default = default?;
                        OptionMsgType::Spin { default, max, min }
                    }
                    "combo" => {
                        let mut options = Vec::new();
                        let mut default = None;
                        while let Some(x) = parts.next() {
                            let option = parts.next()?;
                            if x == "default" {
                                default = Some(option);
                                continue;
                            }
                            if x != "var" {
                                return None;
                            }
                            options.push(option.to_owned());
                        }
                        let default = default?;
                        let default = options
                            .iter()
                            .enumerate()
                            .find(|(_, x)| *x == default)
                            .map(|(idx, _)| idx)?;
                        OptionMsgType::Combo { options, default }
                    }
                    "button" => OptionMsgType::Button,
                    "string" => {
                        let default = parts.next()?;
                        if default != "default" {
                            return None;
                        }
                        let default = parts.collect::<Vec<_>>().join(" ");
                        OptionMsgType::String { default }
                    }
                    _ => return None,
                };
                Some(Msg::Option(OptionMsg { name, r#type }))
            }
            _ => None,
        }
    }
}
