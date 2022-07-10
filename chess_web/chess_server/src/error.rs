use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use sqlx::error::DatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("authentication required")]
    Unauthorized,
    #[error("user may not perform that action")]
    Forbidden,
    #[error("request path not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
    #[error("something went wrong on the server side.")]
    ServerError,
    #[error("an error occurred with the database")]
    Sqlx(#[from] sqlx::Error),
}

pub struct ApiError {
    pub error: Error,
    pub payload: Option<String>,
}

impl From<Error> for ApiError {
    fn from(e: Error) -> Self {
        ApiError {
            error: e,
            payload: None,
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        ApiError {
            error: Error::Sqlx(e),
            payload: None,
        }
    }
}

impl Error {
    fn text(&self) -> String {
        format!("{}", self)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (self.status_code(), self.text()).into_response()
    }
}

#[derive(Serialize)]
pub struct ApiResponse {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let text = self.payload.unwrap_or_else(|| self.error.text());

        (self.error.status_code(), Json(ApiResponse { error: text })).into_response()
    }
}

/// A little helper trait for more easily converting database constraint errors into API errors.
///
/// ```rust,ignore
/// let user_id = sqlx::query_scalar!(
///     r#"insert into "user" (username, email, password_hash) values ($1, $2, $3) returning user_id"#,
///     username,
///     email,
///     password_hash
/// )
///     .fetch_one(&ctxt.db)
///     .await
///     .on_constraint("user_username_key", |_| Error::unprocessable_entity([("username", "already taken")]))?;
/// ```
///
/// Something like this would ideally live in a `sqlx-axum` crate if it made sense to author one,
/// however its definition is tied pretty intimately to the `Error` type, which is itself
/// tied directly to application semantics.
///
/// To actually make this work in a generic context would make it quite a bit more complex,
/// as you'd need an intermediate error type to represent either a mapped or an unmapped error,
/// and even then it's not clear how to handle `?` in the unmapped case without more boilerplate.
pub trait ResultExt<T> {
    /// If `self` contains a SQLx database constraint error with the given name,
    /// transform the error.
    ///
    /// Otherwise, the result is passed through unchanged.
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<ApiError>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> ApiError,
    ) -> Result<T, ApiError> {
        self.map_err(|e| match e.into().error {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            error => ApiError {
                error,
                payload: None,
            },
        })
    }
}
