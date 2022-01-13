#![allow(dead_code)]

use std::{path::Path, time::Duration};

use anyhow::{bail, Result};
use hyper::{body, client::HttpConnector, Body, Client as BaseClient, Response};
use hyper_tls::HttpsConnector;

use tracing::info;

use crate::bot::DeclineReason;

type Client = BaseClient<HttpsConnector<HttpConnector>, Body>;

#[macro_use]
extern crate tracing;

mod bot;
mod events;
mod game;

const SCHEME: &'static str = "https";
const AUTHORITY: &'static str = "lichess.org";

async fn handle_failed_response(resp: Response<Body>) -> Result<()> {
    if !resp.status().is_success() {
        match body::to_bytes(resp.into_body()).await {
            Ok(x) => {
                bail!(
                    "request did not succeeded, body:`{}`",
                    std::str::from_utf8(&x).unwrap_or("Invalid utf-8")
                );
            }
            Err(e) => {
                bail!("request did not succeeded, could not read body: {}", e)
            }
        }
    }
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("NNYBot starting!");
    let mut bot = bot::Bot::new("./secrets/token.txt").await?;

    //let uri: Uri = format!("{}api/stream/event", BASE_URL).parse()?;

    let mut accepting_game = false;
    let mut active_game: Option<String> = None;

    while let Some(e) = bot.next_event().await {
        match e {
            events::Event::Challenge { challenge } => {
                info!("recieved challenge from `{}`", challenge.challenger.name);
                if accepting_game || active_game.is_some() {
                    bot.decline_challenge(&challenge, DeclineReason::Later)
                        .await?;
                } else {
                    if let Some(reason) = bot.should_decline(&challenge) {
                        bot.decline_challenge(&challenge, reason).await?;
                    } else {
                        bot.accept_challenge(&challenge).await?;
                        accepting_game = true;
                    }
                }
            }
            events::Event::GameStart { game } => {
                let game_future = bot.spawn_game(game.id.clone(), Path::new("./engine"));

                tokio::spawn(async move {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    let mut game = match game_future.await {
                        Ok(game) => game,
                        Err(e) => {
                            error!("could not create game: {:?}", e);
                            return;
                        }
                    };

                    match game.start_engine().await {
                        Ok(()) => {}
                        Err(e) => {
                            error!("could not start engine : {:?}", e);
                            game.giveup(
                                &[
                                    "Sorry, I seem to be unable to start my brain",
                                    "I will have to abort",
                                ],
                                true,
                            )
                            .await;
                            game.quit().await;
                            return;
                        }
                    }

                    match game.connect_stream().await {
                        Ok(()) => {}
                        Err(e) => {
                            error!("could not start engine : {:?}", e);
                            game.giveup(
                                &[
                                    "Sorry, I seem to have problems connecting to the game",
                                    "I will have to abort",
                                ],
                                true,
                            )
                            .await;
                            game.quit().await;
                            return;
                        }
                    }

                    game.start().await;
                });
                active_game = Some(game.id);
                accepting_game = false;
            }
            events::Event::GameFinish { game } => {
                if Some(game.id) != active_game {
                    error!("Got a finish event for a game which the bot was not playing");
                } else {
                    info!("game finished!");
                    active_game = None;
                }
            }
            _ => {}
        }
    }

    error!("event stream ended!");

    Ok(())
}
