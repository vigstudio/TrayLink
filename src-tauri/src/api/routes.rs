use std::net::SocketAddr;
use std::time::Instant;

use axum::extract::{ConnectInfo, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::api::auth::extract_token;
use crate::api::server::restart_server;
use crate::launcher::{exec_cmd, open_app, open_file, LauncherError};
use crate::state::{LogEntry, SharedState, APP_VERSION};

#[derive(Serialize)]
pub struct StatusResponse {
    online: bool,
    version: &'static str,
    port: u16,
}

#[derive(Deserialize)]
pub struct OpenAppRequest {
    app: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenFileRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct ExecRequest {
    cmd: String,
}

#[derive(Serialize)]
struct SuccessResponse {
    ok: bool,
    message: String,
}

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/status", get(status))
        .route("/open-app", post(open_app_handler))
        .route("/open-file", post(open_file_handler))
        .route("/exec", post(exec_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn status(State(state): State<SharedState>) -> Json<StatusResponse> {
    let port = state.config.read().unwrap().port;
    log_request(&state, "GET", "/status", 200, 0, "127.0.0.1");
    Json(StatusResponse {
        online: true,
        version: APP_VERSION,
        port,
    })
}

async fn open_app_handler(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OpenAppRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, &state) {
        log_request(
            &state,
            "POST",
            "/open-app",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    let apps = state.config.read().unwrap().apps.clone();
    match open_app::open_app(&body.app, &apps, body.url.as_deref()) {
        Ok(()) => {
            log_request(
                &state,
                "POST",
                "/open-app",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            let detail = body
                .url
                .as_deref()
                .filter(|value| !value.is_empty())
                .map(|url| format!(" with url '{url}'"))
                .unwrap_or_default();
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened app '{}{detail}'", body.app),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(&state, "POST", "/open-app", started, addr, err),
    }
}

async fn open_file_handler(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OpenFileRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, &state) {
        log_request(
            &state,
            "POST",
            "/open-file",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    match open_file::open_file(&body.path) {
        Ok(()) => {
            log_request(
                &state,
                "POST",
                "/open-file",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened file '{}'", body.path),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(&state, "POST", "/open-file", started, addr, err),
    }
}

async fn exec_handler(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ExecRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, &state) {
        log_request(
            &state,
            "POST",
            "/exec",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    let commands = state.config.read().unwrap().commands.clone();
    match exec_cmd::exec_command(&body.cmd, &commands) {
        Ok(is_internal) => {
            if is_internal && body.cmd == "restart_server" {
                let app = state.app_handle.clone();
                let state_clone = state.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = restart_server(app, state_clone).await;
                });
            }

            log_request(
                &state,
                "POST",
                "/exec",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("executed command '{}'", body.cmd),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(&state, "POST", "/exec", started, addr, err),
    }
}

fn error_response(
    state: &SharedState,
    method: &str,
    path: &str,
    started: Instant,
    addr: SocketAddr,
    err: LauncherError,
) -> Response {
    let status = match &err {
        LauncherError::AppNotAllowed(_) | LauncherError::CommandNotAllowed(_) => StatusCode::FORBIDDEN,
        LauncherError::PathNotAllowed(_) => StatusCode::FORBIDDEN,
        LauncherError::PathNotFound(_) => StatusCode::NOT_FOUND,
        LauncherError::LaunchFailed(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    log_request(
        state,
        method,
        path,
        status.as_u16(),
        started.elapsed().as_millis() as u64,
        &addr.ip().to_string(),
    );

    (
        status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

fn log_request(
    state: &SharedState,
    method: &str,
    path: &str,
    status: u16,
    duration_ms: u64,
    client_ip: &str,
) {
    state.push_log(LogEntry {
        timestamp: Utc::now(),
        method: method.to_string(),
        path: path.to_string(),
        status,
        duration_ms,
        client_ip: client_ip.to_string(),
    });
}
