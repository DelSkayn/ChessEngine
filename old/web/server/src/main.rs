#![allow(dead_code)]
//#![feature(trivial_bounds)]

use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Router};
use axum_macros::FromRef;
use base64::engine::fast_portable::FastPortable;
use engine::colosseum::Colosseum;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    opt::auth::Root,
    Surreal,
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use std::{io, sync::Arc};

mod api;
//mod error;
mod engine;
mod error;
mod session;

mod codec;
mod temp;

pub static BASE64_ENGINE: FastPortable = FastPortable::from(
    &base64::alphabet::URL_SAFE,
    base64::engine::fast_portable::PAD,
);

type Db = Surreal<Client>;

#[derive(Clone)]
pub struct Secret(pub String);

#[derive(Clone, FromRef)]
pub struct ServerState {
    pub db: Db,
    pub secret: Secret,
    pub colosseum: Arc<Colosseum>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let connect_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db = Surreal::new::<Ws>(connect_url).await?;

    db.signin(Root {
        username: "root",
        password: "root",
    })
    .await?;

    db.use_ns("chess").use_db("chess").await?;

    let state = ServerState {
        db,
        secret: Secret(std::env::var("SECRET").expect("SECRET not set")),
        colosseum: Arc::new(Colosseum::new()),
    };

    let serve_dir = ServeDir::new("static");
    let serve_dir = get_service(serve_dir).handle_error(handle_static_error);

    let app = Router::<ServerState, _>::new()
        .nest("/api/v1", api::serve())
        .fallback_service(serve_dir)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    info!("starting server on {server_addr}");

    tokio::select! {
        _ = axum::Server::bind(&server_addr.parse().expect("invalid server address"))
            .serve(app.into_make_service()) => {}
        _ = tokio::signal::ctrl_c() => {
            info!("quiting!")
        }
    }
    Ok(())
}

async fn handle_static_error(e: io::Error) -> impl IntoResponse {
    match e.kind() {
        io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "Could not find request file").into_response()
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response(),
    }
}
