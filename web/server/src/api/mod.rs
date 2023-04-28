use axum::{
    body::Body,
    extract::DefaultBodyLimit,
    routing::{delete, get, post},
    Router,
};

use crate::ServerState;

pub mod engine;
pub mod game;
pub mod position;
pub mod user;

pub fn serve() -> Router<ServerState, Body> {
    Router::new()
        .route("/user", post(user::create))
        .route("/user", get(user::get))
        .route("/user/login", post(user::login))
        .route("/engine", post(engine::create))
        .route("/engine", get(engine::get))
        .route("/engine", delete(engine::delete))
        .route("/game/subscribe", get(game::subscribe))
        .route("/game", post(game::create))
        .route("/game", get(game::get))
        .route("/position", get(position::get))
        .route("/position", post(position::create))
        .layer(DefaultBodyLimit::disable())
}
