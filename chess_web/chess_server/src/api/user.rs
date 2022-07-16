use anyhow::{anyhow, Context as ErrorContext};
use argon2::{
    password_hash::{rand_core::OsRng, Error as PassError, PasswordHasher, SaltString},
    Argon2, PasswordHash,
};
use axum::{Extension, Form, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use crate::{
    error::{ApiError, Error, ResultExt},
    session::{self, UserSession},
    ApiResult, Context,
};

#[derive(Deserialize)]
pub struct CreateUserReq {
    username: String,
    password: String,
}
#[derive(Serialize)]
pub enum CreateUserRes {
    Ok,
}

#[debug_handler]
pub async fn create(
    Form(user): Form<CreateUserReq>,
    Extension(ctx): Extension<Context>,
) -> Result<Json<CreateUserRes>> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    sqlx::query_scalar!(
        r#"insert into "user" (username, password,is_admin) values ($1,$2,false) returning user_id"#,
        user.username,
        password,
    )
        .fetch_one(&ctx.db)
        .await
        .on_constraint("user_username_key", |_|{
            anyhow!(Error::BadRequest).context("Username already exists")
        })?;

    Ok(Json(CreateUserRes::Ok))
}

#[derive(Deserialize)]
pub struct LoginUserReq {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub enum LoginUserRes {
    Ok { token: String },
}

#[debug_handler]
pub async fn login(
    Form(user): Form<LoginUserReq>,
    Extension(ctx): Extension<Context>,
) -> ApiResult<Json<LoginUserRes>> {
    struct Record {
        user_id: i32,
        password: String,
    }

    let res = sqlx::query_as!(
        Record,
        r#"select user_id,password from "user" where username = $1"#,
        user.username
    )
    .fetch_one(&ctx.db)
    .await;

    let res = res.map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            return ApiError {
                payload: Some("invalid username or password".to_string()),
                error: Error::BadRequest,
            }
        }
        _ => Error::ServerError.into(),
    })?;

    let pass_hash =
        PasswordHash::parse(&res.password, Default::default()).map_err(|_| Error::ServerError)?;

    let argon2 = Argon2::default();

    match pass_hash.verify_password(&[&argon2], &user.password) {
        Err(PassError::Password) => {
            return Err(ApiError {
                payload: Some("invalid username or password".to_string()),
                error: Error::BadRequest,
            })
        }
        Err(_) => Err(Error::ServerError)?,
        Ok(()) => {}
    }

    let token = session::create(res.user_id, &ctx).await?;

    Ok(Json(LoginUserRes::Ok { token }))
}

#[derive(Serialize)]
pub struct GetUserResponse {
    username: String,
    is_admin: bool,
}

pub async fn get(
    UserSession(user_id): UserSession,
    Extension(ctx): Extension<Context>,
) -> ApiResult<Json<GetUserResponse>> {
    let user = sqlx::query_as!(
        GetUserResponse,
        r#"select username, is_admin from "user" where user_id=$1"#,
        user_id
    )
    .fetch_one(&ctx.db)
    .await?;

    Ok(Json(user))
}
