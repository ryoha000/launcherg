use axum::{http::StatusCode, response::IntoResponse};

pub async fn hc() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
