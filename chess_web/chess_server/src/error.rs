use anyhow::Result;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::error::DatabaseError;
use tracing::error;

pub struct ApiError(anyhow::Error);

impl<T> From<T> for ApiError
where
    anyhow::Error: From<T>,
{
    fn from(e: T) -> Self {
        ApiError(e.into())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ErrorKind {
    #[error("authentication required")]
    Unauthorized,
    #[error("user may not perform that action")]
    Forbidden,
    #[error("request path not found")]
    NotFound,
    #[error("bad request")]
    BadRequest,
}

impl ErrorKind {
    fn text(&self) -> String {
        format!("{}", self)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest => StatusCode::BAD_REQUEST,
        }
    }
}

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
        self.map_err(|e| match e.into().0.downcast::<sqlx::Error>() {
            Ok(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => map_err(dbe),
            Ok(x) => x.into(),
            Err(e) => ApiError(e),
        })
    }
}

#[derive(Serialize)]
pub enum SerApiError {
    Error { error: String },
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if let Some(cause) = self.0.root_cause().downcast_ref::<ErrorKind>() {
            let status = cause.status_code();
            let msg = self.0.to_string();
            return (status, Json(SerApiError::Error { error: msg })).into_response();
        }

        error!("{:?}", self.0);

        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(SerApiError::Error {
                error: "Something went wrong internally".to_string(),
            }),
        )
            .into_response();
    }
}
