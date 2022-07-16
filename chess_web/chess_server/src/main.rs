#![allow(dead_code)]

use axum::{http::StatusCode, response::IntoResponse, routing::get_service, Extension, Router};
use sqlx::postgres::Postgres;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use std::io;

mod api;
//mod error;
mod error;
mod session;

type Db = Postgres;
type Pool = sqlx::Pool<Db>;

#[derive(Clone)]
pub struct Context {
    pub db: Pool,
    pub secret: String,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();

    let connect_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let db: Pool = sqlx::Pool::connect(&connect_url).await.unwrap();

    sqlx::migrate!().run(&db).await.unwrap();

    session::init_clean(db.clone());

    let context = Context {
        db,
        secret: std::env::var("SECRET").expect("SECRET not set"),
    };

    let app = Router::new()
        .nest("/api/v1", api::serve())
        .fallback(get_service(ServeDir::new("./static")).handle_error(handle_static_error))
        .layer(
            ServiceBuilder::new()
                .layer(tower_http::trace::TraceLayer::new_for_http())
                .layer(Extension(context)),
        );

    let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR not set");
    info!("starting server on {server_addr}");

    axum::Server::bind(&server_addr.parse().expect("invalid server address"))
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_static_error(e: io::Error) -> impl IntoResponse {
    match e.kind() {
        io::ErrorKind::NotFound => {
            (StatusCode::NOT_FOUND, "Could not find request file").into_response()
        }
        _ => (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong").into_response(),
    }
}
