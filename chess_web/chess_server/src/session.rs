use std::time::Duration;

use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    Extension, TypedHeader,
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use sqlx::types::{time::OffsetDateTime, Uuid};

type Hmacs = Hmac<Sha256>;

use crate::{
    error::{ApiError, Error},
    Context, Pool,
};

pub struct UserSession(pub i32);

#[async_trait]
impl<B> FromRequest<B> for UserSession
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let user = extract_user(req).await?;
        Ok(UserSession(user.user_id))
    }
}

pub struct AdminSession(pub i32);

#[async_trait]
impl<B> FromRequest<B> for AdminSession
where
    B: Send,
{
    type Rejection = ApiError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let user = extract_user(req).await?;
        if !user.is_admin {
            return Err(Error::Unauthorized.into());
        }
        Ok(AdminSession(user.user_id))
    }
}

pub fn init_clean(db: Pool) {
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

pub async fn create(user_id: i32, ctx: &Context) -> Result<String, ApiError> {
    let id = sqlx::query_scalar!(
        r#"select session_id from "session" where user_id=$1"#,
        user_id
    )
    .fetch_optional(&ctx.db)
    .await?;

    let id = if let Some(id) = id {
        sqlx::query!(
            r#"update "session" set timestamp = now() where session_id = $1"#,
            id
        )
        .execute(&ctx.db)
        .await?;
        id
    } else {
        sqlx::query_scalar!(
            r#"insert into "session"(user_id) values ($1) returning session_id"#,
            user_id
        )
        .fetch_one(&ctx.db)
        .await?
    };

    let mut mac = Hmacs::new_from_slice(ctx.secret.as_bytes()).expect("could not create hmac");
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

async fn extract_user<B: Send>(req: &mut RequestParts<B>) -> Result<SessionRecord, ApiError> {
    let header = req
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| ApiError {
            error: Error::BadRequest,
            payload: Some("missing authorization header".to_string()),
        })?;

    let token: &str = header.0.token();
    let (session_id, hmac_hash) = token.trim().split_once('|').ok_or_else(|| ApiError {
        error: Error::BadRequest,
        payload: Some("invalid authorization header".to_string()),
    })?;

    let Extension(ctx) = req
        .extract::<Extension<Context>>()
        .await
        .expect("context layer missing");

    let session_bytes = base64::decode(session_id).map_err(|_| ApiError {
        error: Error::BadRequest,
        payload: Some("invalid authorization header".to_string()),
    })?;

    let mut hmac = Hmacs::new_from_slice(ctx.secret.as_bytes()).unwrap();
    hmac.update(&session_bytes);
    let hmac_bytes = base64::decode(hmac_hash).map_err(|_| Error::BadRequest)?;
    if hmac.verify_slice(&hmac_bytes).is_err() {
        return Err(Error::Unauthorized.into());
    }

    let session_id = Uuid::from_slice(&session_bytes).map_err(|_| ApiError {
        error: Error::BadRequest,
        payload: Some("invalid authorization header".to_string()),
    })?;

    let res = sqlx::query_as::<_, SessionRecord>(
        r#"select 
                user_id, timestamp, is_admin
            from "session_view" 
            where session_id = $1 "#,
    )
    .bind(session_id)
    .fetch_one(&ctx.db)
    .await;

    let res = match res {
        Err(sqlx::Error::RowNotFound) => {
            return Err(Error::Unauthorized.into());
        }
        Err(e) => return Err(Error::Sqlx(e).into()),
        Ok(x) => x,
    };

    let now = OffsetDateTime::now_utc();
    if res.timestamp < now - Duration::from_secs(60 * 60 * 24) {
        sqlx::query!(r#"delete from "session" where session_id = $1"#, session_id)
            .execute(&ctx.db)
            .await
            .ok();

        return Err(Error::Unauthorized.into());
    }

    if res.timestamp < now - Duration::from_secs(60 * 60) {
        sqlx::query!(
            r#"update "session" set timestamp = now() where session_id = $1"#,
            session_id
        )
        .execute(&ctx.db)
        .await
        .ok();
    }

    Ok(res)
}
