use std::{
    fmt::{self, Display},
    future::Future,
    path::Path,
};

use anyhow::{bail, Context, Result};
use hyper_tls::HttpsConnector;
use tokio::{fs::File, io::AsyncReadExt};

use crate::{
    events::{Challenge, Event, FromNdJson},
    game::Game,
    Client, AUTHORITY, SCHEME,
};
use hyper::{
    header::{HeaderValue, AUTHORIZATION},
    Body, Client as BaseClient, Request, Uri,
};

#[derive(Clone)]
pub enum DeclineReason {
    Generic,
    Later,
    TooFast,
    TooSlow,
    TimeControl,
    Rated,
    Casual,
}

impl Display for DeclineReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DeclineReason::Generic => write!(f, "generic"),
            DeclineReason::Later => write!(f, "later"),
            DeclineReason::TooFast => write!(f, "tooFast"),
            DeclineReason::TooSlow => write!(f, "tooSlow"),
            DeclineReason::TimeControl => write!(f, "timeControl"),
            DeclineReason::Rated => write!(f, "rated"),
            DeclineReason::Casual => write!(f, "Casual"),
        }
    }
}
pub struct Bot {
    client: Client,
    token: String,
    stream: FromNdJson,
}

impl Bot {
    pub async fn new(token_path: impl AsRef<Path>) -> Result<Self> {
        let https = HttpsConnector::new();
        let client = BaseClient::builder().build::<_, hyper::Body>(https);

        let mut token = String::new();
        info!("reading secret");
        File::open(token_path)
            .await
            .context("Could not open secret file at `./secrets/token.txt`")?
            .read_to_string(&mut token)
            .await
            .context("Failed to read token file")?;

        let token = token.trim().to_string();

        let uri = Uri::builder()
            .scheme(SCHEME)
            .authority(AUTHORITY)
            .path_and_query("/api/stream/event")
            .build()?;

        let req = Request::get(uri)
            //.version(Version::HTTP_2)
            .header(
                hyper::header::AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {}", token))?,
            )
            .body(Body::empty())
            .context("Failed to create request")?;

        let resp = client.request(req).await?;

        if !resp.status().is_success() {
            bail!("Request to lichess event stream failed");
        }

        info!("stream events request succeeded, waiting for events");

        let stream = FromNdJson::new(resp.into_body());

        Ok(Bot {
            client,
            token,
            stream,
        })
    }

    pub async fn next_event(&mut self) -> Option<Event> {
        loop {
            let e = self.stream.next_event().await;
            match e {
                Ok(e) => {
                    trace!("recieve event: {:?}", e);
                    return e;
                }
                Err(e) => {
                    error!("failed to parse event: {:?}", e);
                    continue;
                }
            };
        }
    }

    pub async fn decline_challenge(
        &self,
        challenge: &Challenge,
        reason: DeclineReason,
    ) -> Result<()> {
        let path = format!("/api/challenge/{}/decline?reason={}", challenge.id, reason);
        let uri = Uri::builder()
            .scheme(SCHEME)
            .authority(AUTHORITY)
            .path_and_query(path)
            .build()
            .unwrap();
        let resp = self
            .client
            .request(
                Request::post(uri)
                    .header(AUTHORIZATION, format!("Bearer {}", self.token))
                    .body(Body::empty())?,
            )
            .await?;

        crate::handle_failed_response(resp)
            .await
            .context("Decline request failed")?;

        info!(
            "decline challenge from `{}` with reason `{}`",
            challenge.challenger.name, reason
        );

        Ok(())
    }

    pub async fn accept_challenge(&self, challenge: &Challenge) -> Result<()> {
        let path = format!("/api/challenge/{}/accept", challenge.id);
        let uri = Uri::builder()
            .scheme(SCHEME)
            .authority(AUTHORITY)
            .path_and_query(path)
            .build()
            .unwrap();

        let resp = self
            .client
            .request(
                Request::post(uri)
                    .header(AUTHORIZATION, format!("Bearer {}", self.token))
                    .body(Body::empty())?,
            )
            .await?;

        crate::handle_failed_response(resp)
            .await
            .context("Accept request failed")?;

        info!("accepted challenge from `{}`", challenge.challenger.name);

        Ok(())
    }

    pub fn should_decline(&self, challenge: &Challenge) -> Option<DeclineReason> {
        let limit = challenge.time_control.limit.unwrap_or(u64::MAX);
        let increment = challenge.time_control.increment.unwrap_or(0);

        if limit < 60 {
            return Some(DeclineReason::TooFast);
        }
        if limit > 60 * 20 {
            return Some(DeclineReason::TooSlow);
        }
        if increment > 5 {
            return Some(DeclineReason::TooSlow);
        }
        None
    }

    pub fn spawn_game(
        &self,
        game_id: String,
        engine: impl AsRef<Path>,
    ) -> impl Future<Output = Result<Game>> {
        let path = engine.as_ref().to_path_buf();
        let client = self.client.clone();
        let token = self.token.clone();
        async move {
            Game::new(client, &path, game_id, token)
                .await
                .map_err(anyhow::Error::from)
        }
    }
}
