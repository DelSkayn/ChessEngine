use std::fmt::{self, Display};

use anyhow::{Context, Result};
use hyper::{body::HttpBody, Body};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum Event {
    Challenge { challenge: Challenge },
    ChallengeCanceled { challenge: Challenge },
    ChallengeDeclined { challenge: Challenge },
    GameStart { game: Game },
    GameFinish { game: Game },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub compat: ChallengeCompat,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeColor {
    Random,
    White,
    Black,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    pub id: String,
    pub url: String,
    pub status: ChallengeStatus,
    pub challenger: User,
    pub dest_user: User,
    pub variant: Variant,
    pub rated: bool,
    pub time_control: TimeControl,
    pub color: ChallengeColor,
    pub speed: String,
}

#[derive(Deserialize, Debug)]
pub struct TimeControl {
    pub r#type: String,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub increment: Option<u64>,
    #[serde(default)]
    pub show: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Variant {
    pub key: String,
    pub name: String,
    pub short: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub title: Option<String>,
    pub rating: u32,
    #[serde(default)]
    pub provisional: bool,
    pub online: Option<bool>,
    #[serde(default)]
    pub lag: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct ChallengeCompat {
    pub bot: bool,
    pub board: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeStatus {
    Created,
    Canceled,
    Declined,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameClock {
    pub initial: u64,
    pub increment: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FullGame {
    pub id: String,
    pub rated: bool,
    pub variant: Variant,
    pub clock: Option<GameClock>,
    pub speed: String,
    pub white: User,
    pub black: User,
    pub created_at: u64,
    pub initial_fen: String,
    pub state: GameState,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Started,
    Resign,
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum Player {
    Black,
    White,
}

impl From<Player> for chess_core::Player {
    fn from(c: Player) -> Self {
        match c {
            Player::Black => chess_core::Player::Black,
            Player::White => chess_core::Player::White,
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    pub moves: String,
    pub wtime: u64,
    pub btime: u64,
    pub winc: u64,
    pub binc: u64,
    pub status: GameStatus,
    #[serde(default)]
    pub winner: Option<Player>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChatRoom {
    Player,
    Spectator,
}

impl Display for ChatRoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ChatRoom::Player => write!(f, "player"),
            ChatRoom::Spectator => write!(f, "spectator"),
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatLine {
    pub username: String,
    pub text: String,
    pub room: ChatRoom,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GameEvent {
    GameFull(FullGame),
    GameState(GameState),
    ChatLine(ChatLine),
}

pub struct FromNdJson {
    body: Body,
    buffer: Vec<u8>,
}

impl FromNdJson {
    pub fn new(body: Body) -> Self {
        FromNdJson {
            body,
            buffer: Vec::new(),
        }
    }

    pub async fn next_event<T: DeserializeOwned>(&mut self) -> Result<Option<T>> {
        loop {
            if let Some(x) = self
                .buffer
                .iter()
                .copied()
                .enumerate()
                .find(|x| x.1 == b'\n')
                .map(|x| x.0)
            {
                let mut value = self.buffer.split_off(x + 1);
                std::mem::swap(&mut value, &mut self.buffer);
                if value.len() > 1 {
                    trace!("recieved json: {:?}", std::str::from_utf8(&value));
                    return Ok(Some(serde_json::from_slice(&value).with_context(|| {
                        format!(
                            "Could not parse string: `{}`",
                            std::str::from_utf8(&value).unwrap_or("invalid utf8")
                        )
                    })?));
                }
            }
            if let Some(x) = self.body.data().await {
                self.buffer.extend_from_slice(&x?);
            } else {
                return Ok(None);
            }
        }
    }
}
