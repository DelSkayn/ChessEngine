use argon2::{
    password_hash::{rand_core::OsRng, Error as PassError, PasswordHasher, SaltString},
    Argon2, PasswordHash,
};
use axum::{extract::State, Form, Json};
use common::user::{self, LoginRequest, LoginResponse};

use crate::{
    error::{DbResultExt, Error, ErrorContext, ErrorKind, ResultExt},
    session::{self, UserSession},
    Pool, ServerState,
};

pub async fn create(
    State(db): State<Pool>,
    Form(user): Form<common::user::RegisterRequest>,
) -> Result<Json<common::user::RegisterResponse>, Error> {
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
        .fetch_one(&db)
        .await
        .on_constraint("user_username_key", |_|{
            Error::from(ErrorKind::BadRequest).context("Username already exists")
        }).log_error()?;

    Ok(Json(common::user::RegisterResponse::Ok))
}

pub async fn login(
    State(ctx): State<ServerState>,
    Form(user): Form<LoginRequest>,
) -> Result<Json<LoginResponse>, Error> {
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
            Error::from(ErrorKind::BadRequest).context("invalid username or password")
        }
        e => e.into(),
    })?;

    let pass_hash =
        PasswordHash::parse(&res.password, Default::default()).map_err(Error::string)?;

    let argon2 = Argon2::default();

    match pass_hash.verify_password(&[&argon2], &user.password) {
        Err(PassError::Password) => {
            return Err(ErrorKind::BadRequest).context("invalid username or password")
        }
        Err(e) => return Err(Error::string(e)),
        Ok(()) => {}
    }

    let token = session::create(res.user_id, &ctx).await?;

    Ok(Json(LoginResponse::Ok { token }))
}

pub async fn get(
    UserSession(user_id): UserSession,
    State(db): State<Pool>,
) -> Result<Json<user::GetResponse>, Error> {
    struct GetUser {
        username: String,
        is_admin: bool,
    }

    let user = sqlx::query_as!(
        GetUser,
        r#"select username, is_admin from "user" where user_id=$1"#,
        user_id,
    )
    .fetch_one(&db)
    .await?;

    Ok(Json(user::GetResponse::Ok {
        username: user.username,
        is_admin: user.is_admin,
    }))
}
