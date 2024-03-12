use axum::{
    http::StatusCode,
    response::{IntoResponse, Response, Html}
};
use anyhow;

pub(crate) enum ServerError{
    NotFound(String),
    InternalError(anyhow::Error)
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(msg) =>
                (StatusCode::NOT_FOUND, format!("Resource not found: {}", msg)).into_response(),
            Self::InternalError(err) =>
                (StatusCode::INTERNAL_SERVER_ERROR, format!("Internal error: {}", err)).into_response()
        }
    }
}

impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>
{
    fn from(err: E) -> Self {
        Self::InternalError(err.into())
    }
}
