use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct AppError(pub StatusCode, pub String);

impl AppError {
    pub fn not_found() -> Self {
        AppError(StatusCode::NOT_FOUND, "not found".to_string())
    }
}

impl<E: std::fmt::Display> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("internal error: {e}"),
        )
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (self.0, self.1).into_response()
    }
}
