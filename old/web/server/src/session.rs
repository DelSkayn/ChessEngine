use std::time::Duration;

use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization, HeaderMapExt},
    http::request::Parts,
};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sqlx::types::{time::OffsetDateTime, Uuid};
use surrealdb::sql::{Datetime, Thing};

type Hmacs = Hmac<Sha256>;

use crate::{
    error::{Error, ErrorContext, ErrorKind},
    Db, Pool, ServerState,
};

pub struct UserSession(pub i32);

#[async_trait]
impl FromRequestParts<ServerState> for UserSession {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let user = extract_user(parts, state).await?;
        Ok(UserSession(user.user_id))
    }
}

pub struct AdminSession(pub i32);

#[async_trait]
impl FromRequestParts<ServerState> for AdminSession {
    type Rejection = Error;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &ServerState,
    ) -> Result<Self, Self::Rejection> {
        let user = extract_user(parts, state).await?;
        if !user.is_admin {
            return ErrorKind::Unauthorized
                .wrap()
                .context("user is not an admin")?;
        }
        Ok(AdminSession(user.user_id))
    }
}

/*
pub fn init_clean(db: Db) {
    tokio::spawn(async move {
        loop {
            sqlx::query!(r#"delete from "session" where timestamp < now() - '1 day'::interval"#)
                .execute(&db)
                .await
                .ok();

            tokio::time::sleep(Duration::from_secs(60 * 60)).await;
        }
    });
}
*/

#[derive(Debug, Serialize)]
pub struct Session {
    stamp: Datetime,
    user: Thing,
}

#[derive(Debug, Serialize)]
pub struct SessionUpdate {
    stamp: Datetime,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    id: Thing,
}

pub async fn create(user: Thing, ctx: &ServerState) -> Result<String, Error> {
    let id = ctx
        .db
        .query("SELECT id FROM session WHERE user=$id")
        .bind(("id", user))
        .await?;

    let id = id.take::<Option<Record>>(0)?.map(|x| id);

    let id = if let Some(id) = id {
        let time = std::time::SystemTime::now();
        ctx.db
            .update("session")
            .merge(SessionUpdate {
                stamp: Datetime::default(),
            })
            .await?;

        id
    } else {
        let res: Record = ctx
            .db
            .create("session")
            .content(Session {
                stamp: Datetime::default(),
                user,
            })
            .await?;

        res.id
    };

    let mut mac = Hmacs::new_from_slice(ctx.secret.0.as_bytes()).expect("could not create hmac");
    mac.update(id.as_bytes());

    let result = mac.finalize();
    Ok(format!(
        "{}|{}",
        base64::encode(id.as_bytes()),
        base64::encode(result.into_bytes())
    ))
}

#[derive(sqlx::FromRow)]
struct SessionRecord {
    user_id: i32,
    timestamp: OffsetDateTime,
    is_admin: bool,
}

async fn extract_user(req: &mut Parts, state: &ServerState) -> Result<SessionRecord, Error> {
    let header = req
        .headers
        .typed_get::<Authorization<Bearer>>()
        .ok_or_else(|| {
            Error::from(ErrorKind::BadRequest).context("missing authorization header")
        })?;

    let token: &str = header.token();
    let (session_id, hmac_hash) = token.trim().split_once('|').ok_or_else(|| {
        Error::from(ErrorKind::BadRequest).context("invalid authorization header")
    })?;

    let session_bytes = base64::decode(session_id)
        .map_err(|_| Error::from(ErrorKind::BadRequest).context("invalid authorization header"))?;

    let mut hmac = Hmacs::new_from_slice(state.secret.0.as_bytes()).unwrap();
    hmac.update(&session_bytes);
    let hmac_bytes = base64::decode(hmac_hash).map_err(|_| ErrorKind::BadRequest)?;
    if hmac.verify_slice(&hmac_bytes).is_err() {
        ErrorKind::Unauthorized.wrap()?;
    }

    let session_id = Uuid::from_slice(&session_bytes)
        .map_err(|_| Error::from(ErrorKind::BadRequest).context("invalid authorization header"))?;

    let res = sqlx::query_as::<_, SessionRecord>(
        r#"select 
                user_id, timestamp, is_admin
            from "session_view" 
            where session_id = $1 "#,
    )
    .bind(session_id)
    .fetch_one(&state.db)
    .await;

    let res = match res {
        Err(sqlx::Error::RowNotFound) => {
            return Err(ErrorKind::Unauthorized).context("session invalid");
        }
        Err(e) => return Err(e.into()),
        Ok(x) => x,
    };

    let now = OffsetDateTime::now_utc();
    if res.timestamp < now - Duration::from_secs(60 * 60 * 24) {
        sqlx::query!(r#"delete from "session" where session_id = $1"#, session_id)
            .execute(&state.db)
            .await
            .ok();

        return Err(ErrorKind::Unauthorized).context("session expired");
    }

    if res.timestamp < now - Duration::from_secs(60 * 60) {
        sqlx::query!(
            r#"update "session" set timestamp = now() where session_id = $1"#,
            session_id
        )
        .execute(&state.db)
        .await
        .ok();
    }

    Ok(res)
}
