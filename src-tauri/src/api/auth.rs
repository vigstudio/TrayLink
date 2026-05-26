use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::state::SharedState;

pub fn extract_token(headers: &HeaderMap, state: &SharedState) -> Result<(), Response> {
    let expected = state.config.read().unwrap().token.clone();

    if let Some(auth) = headers.get("authorization") {
        if let Ok(value) = auth.to_str() {
            if let Some(token) = value.strip_prefix("Bearer ") {
                if token == expected {
                    return Ok(());
                }
            }
        }
    }

    if let Some(token_header) = headers.get("x-api-token") {
        if let Ok(value) = token_header.to_str() {
            if value == expected {
                return Ok(());
            }
        }
    }

    Err((
        StatusCode::UNAUTHORIZED,
        axum::Json(serde_json::json!({ "error": "invalid or missing token" })),
    )
        .into_response())
}
