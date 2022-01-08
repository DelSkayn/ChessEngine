use std::{path::Path, process::Stdio, time::Duration};

use anyhow::{anyhow, bail, Context, Result};
use hyper::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    Body, Request, Uri,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};

use crate::{
    events::{ChatRoom, FromNdJson},
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
        let path = format!("/api/board/game/stream/{}", game_id);
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

        let req = client
            .request(req)
            .await
            .context("Failed to request stream")?;

        if !req.status().is_success() {
            bail!("Stream request return unsuccessfull status code");
        }

        Ok(FromNdJson::new(req.into_body()))
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

    /*



        let child: Result<Child> = match child {
            Ok(mut child) => {
                let stdin = child.stdin.as_mut().unwrap();
                let stdout = child.stdout.as_mut().unwrap();
                let timeout =
                    tokio::time::timeout(Duration::from_secs(2), Self::start_uci(stdin, stdout))
                        .await
                        .context("Uci startup timedout");
                match timeout {
                    Err(e) => {
                        child
                            .kill()
                            .await
                            .map_err(|e| {
                                error!("could not kill child: {}", e);
                                ()
                            })
                            .ok();
                        Err(e.into())
                    }
                    Ok(()) => Ok(child),
                }
            }
            Err(e) => {


            },
        };

        let mut child = match child {
            Ok(child) => child,
            Err(e) => {
                // Give lichess some time to catch up.
                tokio::time::sleep(Duration::from_secs(1)).await;
                Game::send_message_game(
                    &client,
                    &game_id,
                    &token,
                    "Sorry I will have to abort.",
                    ChatRoom::Player,
                )
                .await;

                Game::abort_game(&client, &game_id, &token).await?;
                return Err(e.into());
            }
        };

        let stream = match Self::start_game_stream(&client, &game_id, &token)
            .await
            .context("Engine UCI protocol timedout")
        {
            Ok(x) => x,
            Err(e) => {
                child
                    .kill()
                    .await
                    .map_err(|e| {
                        error!("could not kill child: {}", e);
                        ()
                    })
                    .ok();

                Game::abort_game(
                    &client,
                    &[
                        "I seem to be having problems connecting",
                        "Sorry I will have to abort.",
                    ],
                    &game_id,
                    &token,
                )
                .await?;
                return Err(e);
            }
        };

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
            client,
            stdout: BufReader::new(stdout),
            stdin,
            child,
            game_id,
            token,
            stream,
        })
    }

    */

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

    pub async fn start(self) {
        self.quit().await;
    }
}
