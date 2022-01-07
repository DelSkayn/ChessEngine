use anyhow::Result;
use hyper::{body::HttpBody, Body};
use serde::{de::DeserializeOwned, Deserialize};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
enum Event {
    Challenge { challenge: Challenge },
    ChallengeCanceled { challenge: Challenge },
    ChallengeDeclinded { challenge: Challenge },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum ChallengeColor {
    Random,
    White,
    Black,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Challenge {
    id: String,
    url: String,
    status: ChallengeStatus,
    challenger: User,
    dest_user: User,
    variant: Variant,
    rated: bool,
    time_control: TimeControl,
    color: ChallengeColor,
    speed: String,
}

#[derive(Deserialize, Debug)]
pub struct TimeControl {
    r#type: String,
    limit: u64,
    increment: u64,
    show: String,
}

#[derive(Deserialize, Debug)]
pub struct Variant {
    key: String,
    name: String,
    short: String,
}

#[derive(Deserialize, Debug)]
pub struct User {
    id: String,
    name: String,
    title: Option<String>,
    rating: u32,
    #[serde(default)]
    provisional: bool,
    online: bool,
    #[serde(default)]
    lag: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct ChallengeCompat {
    bot: bool,
    board: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChallengeStatus {
    Created,
    Canceled,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameClock {
    initial: u64,
    increment: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FullGame {
    id: String,
    rated: bool,
    variant: Variant,
    clock: GameClock,
    speed: String,
    white: User,
    black: User,
    created_at: u64,
    initial_fen: String,
    state: GameState,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GameStatus {
    Started,
    Resign,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum GameWinner {
    Black,
    White,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GameState {
    moves: String,
    wtime: u64,
    btime: u64,
    winc: u64,
    binc: u64,
    status: GameStatus,
    #[serde(default)]
    winner: Option<GameWinner>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum ChatRoom {
    Player,
    Spectator,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatLine {
    username: String,
    text: String,
    room: ChatRoom,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type")]
pub enum GameEvent {
    GameFull(FullGame),
    GameState(GameState),
    ChatLine(ChatLine),
}

pub struct ToNdJson<'a> {
    body: &'a mut Body,
    buffer: Vec<u8>,
}

impl<'a> ToNdJson<'a> {
    pub fn new(body: &'a mut Body) -> Self {
        ToNdJson {
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
                    return Ok(Some(serde_json::from_slice(&value)?));
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

pub async fn parse_incoming_events(body: &mut Body) -> Result<()> {
    let mut incomming = ToNdJson::new(body);
    while let Some(x) = incomming.next_event::<Event>().await.transpose() {
        info!("recieved event: {:?}", x);
    }
    info!("server quit");

    Ok(())
}
