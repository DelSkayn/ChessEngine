use std::time::Duration;

use chess_core::{uci::InfoMsg, Move, Player};
use serde::{Deserialize, Serialize};

use crate::engine::Engine;

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum WonBy {
    Mate,
    Timeout,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum NoContestReason {
    Canceled,
    EngineCrashed,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug)]
pub enum Outcome {
    Won(Player, WonBy),
    Drawn,
    NoContest(NoContestReason),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Event {
    StartGame {
        position: String,
        white: Engine,
        black: Engine,
    },
    Move(Move),
    Time {
        white: Duration,
        black: Duration,
    },
    Eval(Player, InfoMsg),
    GameEnded {
        outcome: Outcome,
        updated_elo_white: f64,
        elo_gain_white: f64,
        updated_elo_black: f64,
        elo_gain_black: f64,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Scheduled {
    pub white: Engine,
    pub black: Engine,
    pub time: Duration,
    pub increment: Option<Duration>,
    pub position: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ScheduleReq {
    pub white: i32,
    pub black: i32,
    pub time: Duration,
    pub increment: Option<Duration>,
    pub position: i32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum ScheduleRes {
    Ok,
    Err {
        error: String,
        context: Option<Vec<String>>,
    },
}
