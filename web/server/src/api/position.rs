use axum::{extract::State, Json};
use chess_core::{board::EndChain, Board};
use common::position;

use crate::{
    error::{Error, ErrorKind},
    session::AdminSession,
    Pool,
};

pub async fn create(
    State(db): State<Pool>,
    _ses: AdminSession,
    Json(req): Json<position::CreateReq>,
) -> Result<Json<position::CreateRes>, Error> {
    if let Err(_) = Board::from_fen(&req.fen, EndChain) {
        return Err(Error::from(ErrorKind::BadRequest).context("invalid fen string"));
    }

    let id = sqlx::query_scalar!(
        r#"insert into "position" (fen, name) values ($1,$2) returning "position_id" "#,
        req.fen,
        req.name,
    )
    .fetch_one(&db)
    .await?;

    return Ok(Json(position::CreateRes::Ok { id }));
}

pub async fn get(State(db): State<Pool>) -> Result<Json<Vec<position::Position>>, Error> {
    let res = sqlx::query_as!(
        position::Position,
        r#"select position_id as id, name, fen from "position" where name is not null "#,
    )
    .fetch_all(&db)
    .await?;

    return Ok(Json(res));
}
