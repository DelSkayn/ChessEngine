use std::{path::Path, process::Stdio, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use chess_core::{
    board::EndChain,
    gen::{gen_type, MoveGenerator},
    Board, Square,
};
use hyper::{
    body,
    header::{AUTHORIZATION, CONTENT_TYPE},
    Body, Request, Uri,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use crate::{
    events::{ChatRoom, FromNdJson, GameEvent, GameState, GameStatus, Player},
    Client, AUTHORITY, SCHEME,
};

pub struct Game {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    client: Client,
    game_id: String,
    token: String,
    stream: Option<FromNdJson>,
    color: Player,
    board: Board,
    move_gen: MoveGenerator,
}

impl Game {
    async fn start_uci(stdin: &mut ChildStdin, stdout: &mut BufReader<ChildStdout>) {
        let mut stdout = BufReader::new(stdout);
        stdin.write_all(b"uci\n").await.unwrap();
        let mut line_buffer = String::new();
        loop {
            line_buffer.clear();
            stdout.read_line(&mut line_buffer).await.unwrap();
            if line_buffer.trim() == "uciok" {
                return;
            }
        }
    }

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
        match Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .context("Could not run engine command")
        {
            Ok(mut child) => {
                let stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| anyhow!("Missing stdin from child process"))
                    .unwrap();

                let stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| anyhow!("Missing stdin from child process"))
                    .unwrap();

                Ok(Game {
                    child,
                    stdin,
                    stdout: BufReader::new(stdout),
                    client,
                    game_id,
                    token,
                    stream: None,
                    color: Player::White,
                    board: Board::start_position(EndChain),
                    move_gen: MoveGenerator::new(),
                })
            }

            Err(e) => {
                Game::abort_game(
                    &[
                        "I seem to be unable to start my brain",
                        "Sorry I will have to abort.",
                    ],
                    &client,
                    &game_id,
                    &token,
                )
                .await;
                return Err(e);
            }
        }
    }

    pub async fn start_engine(&mut self) -> Result<()> {
        tokio::time::timeout(
            Duration::from_secs(2),
            Self::start_uci(&mut self.stdin, &mut self.stdout),
        )
        .await
        .context("Uci startup timedout")
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
            ()
        })
        .ok();
    }

    pub async fn abort(&self, texts: &[&str]) {
        Self::abort_game(texts, &self.client, &self.game_id, &self.token).await
    }

    pub async fn abort_game(texts: &[&str], client: &Client, game_id: &str, token: &str) {
        async {
            for text in texts.iter() {
                Game::send_message_game(&client, &game_id, &token, text, ChatRoom::Player).await;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;

            let path = format!("/api/bot/game/{}/abort?", game_id,);
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

    pub async fn quit(mut self) {
        async {
            self.stdin
                .write_all(b"quit\n")
                .await
                .context("Failed to write to process stdin")
                .map_err(|e| error!("Failed to stop engine process: {:?}", e))
                .ok();

            match tokio::time::timeout(Duration::from_secs(1), self.child.wait()).await {
                Ok(x) => {
                    x.context("Failed to wait on child")?;
                }
                Err(_) => {
                    warn!("Engine process failed to stop, killing process");
                    self.child.kill().await?;
                }
            };
            Result::<(), anyhow::Error>::Ok(())
        }
        .await
        .map_err(|e| error!("Failed to stop engine process: {:?}", e))
        .ok();
    }

    pub async fn set_position(&mut self, fen: &str, state: &GameState) -> Result<()> {
        let mut pos_string = String::new();
        pos_string.push_str("position ");
        if fen == "startpos" {
            pos_string.push_str("startpos");
            self.board = Board::start_position(EndChain);
        } else {
            pos_string.push_str("fen");
            pos_string.push_str(fen);
            pos_string.push(' ');
            self.board = Board::from_fen(fen, EndChain)?;
        }
        if !state.moves.is_empty() {
            pos_string.push_str(" moves ");
            pos_string.push_str(&state.moves);

            let mut moves = Vec::new();

            for m in state.moves.split_whitespace() {
                moves.clear();
                self.move_gen
                    .gen_moves::<gen_type::All, _, _>(&self.board, &mut moves);
                let from =
                    Square::from_name(&m[..2]).ok_or_else(|| anyhow!("invalid move name"))?;
                let to = Square::from_name(&m[2..4]).ok_or_else(|| anyhow!("invalid move name"))?;

                let m = moves
                    .iter()
                    .copied()
                    .find(|m| m.from() == from && m.to() == to)
                    .ok_or_else(|| anyhow!("Could not find move"))?;

                self.board.make_move(m);
            }
        }
        pos_string.push('\n');
        self.stdin.write_all(pos_string.as_bytes()).await?;
        Ok(())
    }

    pub async fn make_move(&self, m: &str) -> Result<()> {
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

    pub async fn go(&mut self, state: &GameState) -> Result<()> {
        let cmd = format!("go wtime {} btime {}\n", state.wtime, state.btime);

        self.stdin.write_all(cmd.as_bytes()).await?;
        let mut buffer = String::new();
        loop {
            buffer.clear();
            self.stdout.read_line(&mut buffer).await?;
            trace!("engine: {}", buffer);
            if buffer.starts_with("bestmove") {
                let m = buffer
                    .split_whitespace()
                    .skip(1)
                    .next()
                    .ok_or_else(|| anyhow!("Missing move after `bestmove` command"))?;

                self.make_move(m).await?;
                return Ok(());
            }
        }
    }

    pub async fn start(mut self) {
        self.send_message("Good luck, have fun!", ChatRoom::Player)
            .await;

        let err: Result<()> = async {
            let mut fen = "startpos".to_string();
            loop {
                let ev = self
                    .stream
                    .as_mut()
                    .unwrap()
                    .next_event::<GameEvent>()
                    .await?;

                let ev = if let Some(ev) = ev {
                    ev
                } else {
                    bail!("Game stream ended");
                };

                match ev {
                    GameEvent::GameFull(game) => {
                        self.color = if game.white.id == "nnybot" {
                            Player::White
                        } else {
                            Player::Black
                        };
                        fen = game.initial_fen;

                        self.set_position(&fen, &game.state).await?;
                        if self.board.state.player == self.color.into() {
                            self.go(&game.state).await?;
                        }
                    }
                    GameEvent::GameState(state) => {
                        if state.status != GameStatus::Started {
                            return Ok(());
                        }
                        self.set_position(&fen, &state).await?;
                        if self.board.state.player == self.color.into() {
                            self.go(&state).await?;
                        }
                    }
                    _ => {}
                }
            }
        }
        .await;

        match err {
            Ok(()) => {
                self.send_message("Good game, well played!", ChatRoom::Player)
                    .await;
            }
            Err(e) => {
                error!("Error while playing a game: {:?}", e);
            }
        }

        self.quit().await;
    }
}
