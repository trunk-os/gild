use axum::response::{IntoResponse, Response};
use http::status::StatusCode;

// axum requires a ton of boilerplate to do anything sane with a handler
// this is it. ah, rust. this literally all gets compiled out
pub(crate) type Result<T> = core::result::Result<T, AppError>;

pub(crate) struct AppError(anyhow::Error);

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
