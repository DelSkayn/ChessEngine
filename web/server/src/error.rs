use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::error::DatabaseError;
use std::{error::Error as StdError, fmt};
use tracing::error;

#[derive(Debug, Clone, Copy)]
pub enum ErrorKind {
    Unauthorized,
    Forbidden,
    NotFound,
    BadRequest,
}

impl ErrorKind {
    pub fn status_code(self) -> StatusCode {
        match self {
            ErrorKind::Unauthorized => StatusCode::UNAUTHORIZED,
            ErrorKind::Forbidden => StatusCode::FORBIDDEN,
            ErrorKind::NotFound => StatusCode::NOT_FOUND,
            ErrorKind::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            ErrorKind::Unauthorized => "Unauthorized",
            ErrorKind::Forbidden => "Forbidden",
            ErrorKind::NotFound => "Not found",
            ErrorKind::BadRequest => "Bad request",
        }
    }

    pub fn wrap<T>(self) -> Result<T, Error> {
        Err(self.into())
    }
}

#[derive(Debug)]
pub enum Error {
    Context { context: String, error: Box<Error> },
    Specified(ErrorKind),
    External(Box<dyn StdError + Send + Sync>),
    String(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Context { context, error } => {
                (*error).fmt(f)?;
                writeln!(f, "\t -> {context}")
            }
            Self::Specified(e) => {
                writeln!(f, "Error: {}", e.text())
            }
            Self::External(e) => {
                writeln!(f, "Error: {}", e)
            }
            Self::String(e) => {
                writeln!(f, "Error: {}", e)
            }
        }
    }
}

impl Error {
    pub fn root(&self) -> &Error {
        match *self {
            Error::Context { ref error, .. } => error.root(),
            ref x => &x,
        }
    }

    pub fn string<E: ToString>(e: E) -> Self {
        Error::String(e.to_string())
    }

    pub fn context<C: ToString>(self, context: C) -> Self {
        Error::Context {
            context: context.to_string(),
            error: Box::new(self),
        }
    }
}

impl<E: StdError + Send + Sync + 'static> From<E> for Error {
    fn from(e: E) -> Self {
        Error::External(Box::new(e))
    }
}

impl From<ErrorKind> for Error {
    fn from(e: ErrorKind) -> Self {
        Error::Specified(e)
    }
}

pub trait DbResultExt<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E: Into<Error>> DbResultExt<T> for Result<T, E> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::External(x) => match x.downcast::<sqlx::Error>() {
                Ok(e) => match *e {
                    sqlx::Error::Database(e) if e.constraint() == Some(name) => f(e),
                    e => e.into(),
                },
                Err(e) => Error::External(e),
            },
            e => e,
        })
    }
}

pub trait ErrorContext<T> {
    fn context<C: ToString>(self, context: C) -> Result<T, Error>;

    fn with_context<F, C>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce() -> C,
        C: ToString;
}

impl<T, E: Into<Error>> ErrorContext<T> for Result<T, E> {
    fn context<C: ToString>(self, context: C) -> Result<T, Error> {
        self.map_err(|e| Error::Context {
            context: context.to_string(),
            error: Box::new(e.into()),
        })
    }

    fn with_context<F, C>(self, context: F) -> Result<T, Error>
    where
        F: FnOnce() -> C,
        C: ToString,
    {
        self.map_err(|e| Error::Context {
            context: context().to_string(),
            error: Box::new(e.into()),
        })
    }
}

pub trait ResultExt {
    fn log_error(self) -> Self;
    fn log_error_debug(self) -> Self;
}

impl<T, E: std::fmt::Display + std::fmt::Debug> ResultExt for Result<T, E> {
    fn log_error(self) -> Self {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                error!("logging error: {e}");
                Err(e)
            }
        }
    }

    fn log_error_debug(self) -> Self {
        match self {
            Ok(x) => Ok(x),
            Err(e) => {
                error!("logging error: {e:?}");
                Err(e)
            }
        }
    }
}

#[derive(Serialize)]
enum RespError {
    Err {
        err: String,
        context: Option<Vec<String>>,
    },
}

impl RespError {
    fn from_error(e: Error) -> Self {
        match e {
            Error::Context {
                context: ctx,
                error,
            } => {
                if let Error::Specified(_) = *error {
                    return RespError::Err {
                        err: ctx,
                        context: None,
                    };
                }

                let RespError::Err { err, mut context } = Self::from_error(*error);
                context.get_or_insert_with(Vec::new).push(ctx);
                RespError::Err { err, context }
            }
            Error::Specified(e) => {
                return RespError::Err {
                    err: e.text().to_string(),
                    context: None,
                }
            }
            Error::External(e) => {
                return RespError::Err {
                    err: format!("{}", e),
                    context: None,
                }
            }
            Error::String(err) => return RespError::Err { err, context: None },
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let root = self.root();
        let status = match *root {
            Error::Specified(x) => x.status_code(),
            _ => {
                error!("{self}");

                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let resp_error = RespError::from_error(self);

        (status, Json(resp_error)).into_response()
    }
}
