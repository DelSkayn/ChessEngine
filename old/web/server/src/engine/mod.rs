use std::{path::Path, time::Duration};

use crate::{codec::uci::EngineCodec, error::Error};

use chess_core::uci::{Cmd, Id, Msg, OptionMsg, Version};
use futures::{SinkExt, StreamExt};
use tokio_util::codec::Framed;

pub mod colosseum;
mod game;
mod sandbox;

use sandbox::Sandbox;
use tracing::warn;

#[derive(Clone)]
pub struct ScheduledEngine {
    pub id: i32,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub elo: f64,
    pub games_played: i32,
    pub engine_file: String,
}

impl From<ScheduledEngine> for common::engine::Engine {
    fn from(value: ScheduledEngine) -> Self {
        Self {
            id: value.id,
            elo: value.elo,
            name: value.name,
            author: value.author,
            description: value.description,
            games_played: value.games_played,
        }
    }
}

#[derive(Clone)]
pub struct ScheduledGame {
    pub white: ScheduledEngine,
    pub black: ScheduledEngine,
    pub time: Duration,
    pub increment: Option<Duration>,
    pub position: String,
}

impl From<ScheduledGame> for common::game::Scheduled {
    fn from(value: ScheduledGame) -> Self {
        Self {
            white: value.white.into(),
            black: value.black.into(),
            time: value.time,
            increment: value.increment,
            position: value.position,
        }
    }
}

#[derive(Debug)]
pub struct EngineInfo {
    pub name: String,
    pub author: Option<String>,
    pub version: Option<Version>,
    pub options: Vec<OptionMsg>,
}

pub async fn retrieve_engine_info(engine_path: &Path) -> Result<EngineInfo, Error> {
    let sandbox = Sandbox::from_executable_path(engine_path).await?;
    let mut sandbox = Framed::new(sandbox, EngineCodec);

    let mut author = None;
    let mut name = None;
    let mut version = None;
    let mut options = Vec::new();

    let do_init = async {
        sandbox.send(Cmd::Uci).await?;

        while let Some(x) = sandbox.next().await {
            let msg = match x {
                Ok(x) => x,
                Err(err) => {
                    warn!("error reading input from engine: {err}");
                    continue;
                }
            };
            match msg {
                Msg::UciOk => return Ok(()),
                Msg::Id(Id::Name(x)) => name = Some(x),
                Msg::Id(Id::Author(x)) => {
                    author = Some(x);
                }
                Msg::Id(Id::Version(x)) => {
                    version = Some(x);
                }
                Msg::Option(x) => {
                    options.push(x);
                }
                x => {
                    warn!("engine returned unexpected message: `{x}`");
                }
            }
        }
        warn!("engine quit unexpectedly");
        Err(Error::string("engine quit unexpectedly"))
    };

    match tokio::time::timeout(Duration::from_secs(2), do_init).await {
        Ok(Ok(_)) => {}
        Err(_) => {
            return Err(Error::string(
                "engine failed to initialize within time limit",
            ))
        }
        Ok(Err(x)) => return Err(x.context("failed to initialize engine")),
    }

    sandbox.send(Cmd::Quit).await.ok();
    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;

    if let Err(e) = sandbox.into_inner().stop().await {
        warn!("failed to stop sandbox: {e}");
    }

    let name = name.ok_or(Error::string("engine did not have a name"))?;
    Ok(EngineInfo {
        name,
        author,
        version,
        options,
    })
}
