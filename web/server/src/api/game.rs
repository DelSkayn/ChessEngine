use std::sync::Arc;

use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::Response,
    Form, Json,
};
use common::game;
use tracing::{error, info};

use crate::{
    engine::{
        colosseum::{Colosseum, GameSubscription},
        ScheduledEngine, ScheduledGame,
    },
    error::{Error, ErrorContext, ResultExt},
    session::AdminSession,
    Pool,
};

pub async fn subscribe(State(colo): State<Arc<Colosseum>>, ws: WebSocketUpgrade) -> Response {
    let subscribe = colo.subscribe().await;
    info!("recieved subscription");
    ws.on_upgrade(|socket| handle_game_socket(subscribe, socket))
}

async fn handle_game_socket(mut sub: GameSubscription, mut socket: WebSocket) {
    info!("entering socket loop");
    for m in sub.catch_up {
        let Ok(m) = serde_json::to_string(&m).log_error() else { return };
        if let Err(e) = socket.send(axum::extract::ws::Message::Text(m)).await {
            error!("error sending message: {e}");
            return;
        }
    }

    loop {
        let Ok(m) = sub.events.recv().await else {
            return
        };
        let Ok(m) = serde_json::to_string(&m).log_error() else { return };
        if let Err(e) = socket.send(axum::extract::ws::Message::Text(m)).await {
            error!("error sending message: {e}");
            return;
        }
    }
}

pub async fn create(
    State(db): State<Pool>,
    State(colo): State<Arc<Colosseum>>,
    _admin: AdminSession,
    data: Form<Vec<game::ScheduleReq>>,
) -> Result<Json<game::ScheduleRes>, Error> {
    let mut games = Vec::new();
    for d in data.0 {
        let g = create_scheduled_game(&d, &db).await?;
        games.push(g);
    }

    colo.schedule_games(games).await;

    Ok(Json(game::ScheduleRes::Ok))
}

async fn create_scheduled_game(
    schedule: &game::ScheduleReq,
    db: &Pool,
) -> Result<ScheduledGame, Error> {
    let Some(position) = sqlx::query_scalar!(
        r#"select fen from "position" where "position_id" = $1"#,
        schedule.position,
    )
    .fetch_optional(db)
    .await? else {
        let err = Error::from(crate::error::ErrorKind::BadRequest)
            .context("no such position");
        return Err(err);
    };

    let white = load_engine(schedule.white, db)
        .await
        .context("failed to load white engine")?;

    let black = load_engine(schedule.black, db)
        .await
        .context("failed to load white engine")?;

    Ok(ScheduledGame {
        position,
        white,
        black,
        time: schedule.time,
        increment: schedule.increment,
    })
}

async fn load_engine(id: i32, db: &Pool) -> Result<ScheduledEngine, Error> {
    let Some(engine) = sqlx::query_as!(
        ScheduledEngine,
        r#"select engine_id as id, name, author, description, elo, games_played, engine_file from "engine" where engine_id = $1 "#,
        id
    )
    .fetch_optional(db)
    .await? else {
        let err = Error::from(crate::error::ErrorKind::BadRequest)
            .context("no such engine");
        return Err(err);
    };

    Ok(engine)
}

pub async fn get(State(colo): State<Arc<Colosseum>>) -> Json<Vec<game::Scheduled>> {
    let game = colo
        .get_scheduled_games()
        .await
        .into_iter()
        .map(game::Scheduled::from)
        .collect();
    Json(game)
}
