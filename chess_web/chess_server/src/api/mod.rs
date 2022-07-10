use axum::{
    routing::{get, post},
    Router,
};

pub mod engine;
pub mod user;

pub fn serve() -> Router {
    Router::new()
        .route("/user", post(user::create))
        .route("/user", get(user::get))
        .route("/user/login", post(user::login))
}
