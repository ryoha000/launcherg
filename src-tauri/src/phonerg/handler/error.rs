use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum HandlerError {
    #[error("Error. StatusCode: {0}, Message: {1}")]
    Error(StatusCode, String),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        match self {
            HandlerError::Error(status, message) => (status, message).into_response(),
            HandlerError::Anyhow(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Something went wrong: {}", e),
            )
                .into_response(),
        }
    }
}
