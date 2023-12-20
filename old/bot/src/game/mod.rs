use std::{path::Path, time::Duration};

use anyhow::{bail, Context, Result};

use futures_util::{future::Either, pin_mut};
use hyper::{
    body,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Body, Request, Uri,
};

use crate::{
    events::{ChatRoom, FromNdJson, GameEvent, GameStatus, Player},
    Client, AUTHORITY, SCHEME,
};

use self::engine::Engine;

mod engine;

pub struct Game {
    client: Client,
    game_id: String,
    token: String,
    stream: Option<FromNdJson>,
    color: Player,
    engine: engine::Engine,
}

impl Game {
    async fn start_game_stream(client: &Client, game_id: &str, token: &str) -> Result<FromNdJson> {
        let path = format!("/api/bot/game/stream/{}", game_id);
        let uri = Uri::builder()
            .scheme(SCHEME)
            .authority(AUTHORITY)
            .path_and_query(path)
            .build()
            .context("Failed to build uri")?;

        let req = Request::get(uri)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .body(Body::empty())
            .context("Could create request")?;

        let resp = client
            .request(req)
            .await
            .context("Failed to request stream")?;

        if !resp.status().is_success() {
            match body::to_bytes(resp.into_body()).await {
                Ok(x) => {
                    bail!(
                        "Game stream request did not succeeded, body:`{}`",
                        std::str::from_utf8(&x).unwrap_or("Invalid utf-8")
                    );
                }
                Err(e) => {
                    bail!(
                        "game stream request did not succeeded, could not read body: {}",
                        e
                    )
                }
            }
        }

        Ok(FromNdJson::new(resp.into_body()))
    }

    pub async fn new(client: Client, path: &Path, game_id: String, token: String) -> Result<Self> {
        let engine = match Engine::new(path) {
            Ok(x) => x,
            Err(e) => {
                Game::giveup_game(
                    &[
                        "I seem to be unable to start my brain",
                        "Sorry I will have to abort.",
                    ],
                    &client,
                    &game_id,
                    &token,
                    true,
                )
                .await;
                return Err(e);
            }
        };

        Ok(Game {
            client,
            game_id,
            token,
            stream: None,
            color: Player::White,
            engine,
        })
    }

    pub async fn start_engine(&mut self) -> Result<()> {
        self.engine.start().await
    }

    pub async fn connect_stream(&mut self) -> Result<()> {
        self.stream =
            Some(Self::start_game_stream(&self.client, &self.game_id, &self.token).await?);
        Ok(())
    }

    pub async fn send_message(&self, message: &str, room: ChatRoom) {
        Game::send_message_game(&self.client, &self.game_id, &self.token, message, room).await;
    }

    pub async fn send_message_game(
        client: &Client,
        game_id: &str,
        token: &str,
        message: &str,
        room: ChatRoom,
    ) {
        async {
            let path = format!("/api/bot/game/{}/chat", game_id,);

            let body = format!("room={}&text={}", room, urlencoding::encode(message));

            let body: Result<String, std::io::Error> = Ok(body);

            let stream = futures_util::stream::once(futures_util::future::ready(body));

            let url = Uri::builder()
                .scheme(SCHEME)
                .authority(AUTHORITY)
                .path_and_query(path)
                .build()?;

            let req = Request::post(url)
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::wrap_stream(stream))?;

            let resp = client.request(req).await?;

            trace!("send message to game `{}` : `{}`", game_id, message);
            crate::handle_failed_response(resp)
                .await
                .context("Failed to send chat message")
        }
        .await
        .map_err(|e| {
            error!("Could not send message: `{:?}`", e);
        })
        .ok();
    }

    pub async fn giveup(&self, texts: &[&str], abort: bool) {
        Self::giveup_game(texts, &self.client, &self.game_id, &self.token, abort).await
    }

    pub async fn giveup_game(
        texts: &[&str],
        client: &Client,
        game_id: &str,
        token: &str,
        abort: bool,
    ) {
        async {
            for text in texts.iter() {
                Game::send_message_game(client, game_id, token, text, ChatRoom::Player).await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;

            let path = if abort {
                format!("/api/bot/game/{}/abort", game_id)
            } else {
                format!("/api/bot/game/{}/resign", game_id)
            };
            let url = Uri::builder()
                .scheme(SCHEME)
                .authority(AUTHORITY)
                .path_and_query(path)
                .build()?;

            let req = Request::post(url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(Body::empty())?;

            let resp = client.request(req).await?;

            crate::handle_failed_response(resp)
                .await
                .context("Failed to abort game")
        }
        .await
        .map_err(|e| error!("Failed to abort game: {:?}", e))
        .ok();
    }

    pub async fn make_move(&self, m: &str) -> Result<()> {
        dbg!(m);
        let path = format!("/api/bot/game/{}/move/{}", self.game_id, m);
        let uri = Uri::builder()
            .scheme(SCHEME)
            .authority(AUTHORITY)
            .path_and_query(path)
            .build()
            .context("Failed to build uri")?;

        let req = Request::post(uri)
            .header(AUTHORIZATION, format!("Bearer {}", &self.token))
            .body(Body::empty())
            .context("Failed to creatue request")?;

        let resp = self
            .client
            .request(req)
            .await
            .context("Move making request failed")?;

        crate::handle_failed_response(resp).await?;

        Ok(())
    }

    pub async fn start(mut self) {
        self.send_message("Good luck, have fun!", ChatRoom::Player)
            .await;

        let err: Result<GameStatus> = async {
            let mut fen = "startpos".to_string();
            loop {
                let ev = if self.engine.is_running() {
                    let stream_future = self.stream.as_mut().unwrap().next_event::<GameEvent>();

                    let engine_future = self.engine.get_move();
                    pin_mut!(stream_future);
                    pin_mut!(engine_future);

                    match futures_util::future::select(stream_future, engine_future).await {
                        Either::Left(x) => Either::Left(x.0),
                        Either::Right(x) => Either::Right(x.0),
                    }
                } else {
                    Either::Left(
                        self.stream
                            .as_mut()
                            .unwrap()
                            .next_event::<GameEvent>()
                            .await,
                    )
                };

                match ev {
                    Either::Left(Ok(None)) => bail!("Game stream ended"),
                    Either::Left(Ok(Some(GameEvent::GameFull(game)))) => {
                        self.color = if game.white.id == "nnybot" {
                            Player::White
                        } else {
                            Player::Black
                        };
                        fen = game.initial_fen;

                        self.engine.set_position(&fen, &game.state.moves).await?;
                        if self.engine.cur_player() == self.color.into() {
                            self.engine.go(game.state.wtime, game.state.btime).await?;
                        }
                    }
                    Either::Left(Ok(Some(GameEvent::GameState(state)))) => {
                        if state.status != GameStatus::Started {
                            return Ok(state.status);
                        }
                        self.engine.set_position(&fen, &state.moves).await?;
                        if self.engine.cur_player() == self.color.into() {
                            self.engine.go(state.wtime, state.btime).await?;
                        }
                    }
                    Either::Left(Ok(Some(_))) => {}
                    Either::Left(Err(e)) => {
                        return Err(e);
                    }
                    Either::Right(Err(e)) => {
                        return Err(e);
                    }
                    Either::Right(Ok(m)) => {
                        self.make_move(&m).await?;
                    }
                }
            }
        }
        .await;

        match err {
            Ok(status) => {
                if status != GameStatus::Aborted {
                    self.send_message("Good game, well played!", ChatRoom::Player)
                        .await;
                }
            }
            Err(e) => {
                error!("Error while playing a game: {:?}", e);
                self.giveup(
                    &[
                        "I have encountered some errors I can't recover from.",
                        "I will have to resign..",
                    ],
                    false,
                )
                .await;
            }
        }

        self.quit().await;
    }

    pub async fn quit(self) {
        self.engine.quit().await
    }
}
