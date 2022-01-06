#![allow(dead_code)]

use anyhow::{Context, Result};
use hyper::{header::HeaderValue, Body, Client, Request, Uri};
use hyper_tls::HttpsConnector;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::info;

#[macro_use]
extern crate tracing;

mod events;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("NNYBot starting!");

    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);

    let mut token = String::new();
    info!("reading secret");
    File::open("./secrets/token.txt")
        .await?
        .read_to_string(&mut token)
        .await?;

    let token = token.trim().to_string();

    let uri: Uri = "https://lichess.org/api/stream/event".parse()?;

    let req = Request::get(uri)
        //.version(Version::HTTP_2)
        .header(
            hyper::header::AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        )
        .body(Body::empty())
        .context("Failed to create request")?;

    let mut resp = client.request(req).await?;

    info!("stream events request succeeded, waiting for events");

    events::parse_incoming_events(resp.body_mut()).await?;

    Ok(())
}
