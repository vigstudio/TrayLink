use std::net::SocketAddr;
use std::time::Instant;

use axum::extract::{ConnectInfo, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
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
pub struct OpenAppQuery {
    app: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct OpenFileRequest {
    path: String,
}

#[derive(Deserialize)]
pub struct OpenFileQuery {
    path: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct ExecRequest {
    cmd: String,
}

#[derive(Deserialize)]
pub struct ExecQuery {
    cmd: String,
    #[serde(default)]
    token: Option<String>,
}

#[derive(Serialize)]
struct SuccessResponse {
    ok: bool,
    message: String,
}

pub fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/status", get(status))
        .route("/open-app", get(open_app_get).post(open_app_post))
        .route("/open-file", get(open_file_get).post(open_file_post))
        .route("/exec", get(exec_get).post(exec_post))
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

fn reject_get_disabled() -> Response {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(serde_json::json!({
            "error": "GET API is disabled. Enable it in Dashboard -> Settings, or use POST."
        })),
    )
        .into_response()
}

async fn open_app_post(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OpenAppRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
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

    handle_open_app(
        &state,
        "POST",
        started,
        addr,
        &body.app,
        body.url.as_deref(),
    )
}

async fn open_app_get(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<OpenAppQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/open-app",
            405,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/open-app",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    handle_open_app(
        &state,
        "GET",
        started,
        addr,
        &query.app,
        query.url.as_deref(),
    )
}

fn handle_open_app(
    state: &SharedState,
    method: &str,
    started: Instant,
    addr: SocketAddr,
    app: &str,
    url: Option<&str>,
) -> Response {
    let apps = state.config.read().unwrap().apps.clone();
    match open_app::open_app(app, &apps, url) {
        Ok(()) => {
            log_request(
                state,
                method,
                "/open-app",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            let detail = url
                .filter(|value| !value.is_empty())
                .map(|value| format!(" with url '{value}'"))
                .unwrap_or_default();
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened app '{app}{detail}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/open-app", started, addr, err),
    }
}

async fn open_file_post(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<OpenFileRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
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

    handle_open_file(&state, "POST", started, addr, &body.path)
}

async fn open_file_get(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<OpenFileQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/open-file",
            405,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/open-file",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    handle_open_file(&state, "GET", started, addr, &query.path)
}

fn handle_open_file(
    state: &SharedState,
    method: &str,
    started: Instant,
    addr: SocketAddr,
    path: &str,
) -> Response {
    match open_file::open_file(path) {
        Ok(()) => {
            log_request(
                state,
                method,
                "/open-file",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("opened file '{path}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/open-file", started, addr, err),
    }
}

async fn exec_post(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(body): Json<ExecRequest>,
) -> Response {
    let started = Instant::now();
    if let Err(resp) = extract_token(&headers, None, &state) {
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

    handle_exec(&state, "POST", started, addr, &body.cmd)
}

async fn exec_get(
    State(state): State<SharedState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Query(query): Query<ExecQuery>,
) -> Response {
    let started = Instant::now();
    if !state.config.read().unwrap().allow_get {
        log_request(
            &state,
            "GET",
            "/exec",
            405,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return reject_get_disabled();
    }

    if let Err(resp) = extract_token(&headers, query.token.as_deref(), &state) {
        log_request(
            &state,
            "GET",
            "/exec",
            401,
            started.elapsed().as_millis() as u64,
            &addr.ip().to_string(),
        );
        return resp;
    }

    handle_exec(&state, "GET", started, addr, &query.cmd)
}

fn handle_exec(
    state: &SharedState,
    method: &str,
    started: Instant,
    addr: SocketAddr,
    cmd: &str,
) -> Response {
    let commands = state.config.read().unwrap().commands.clone();
    match exec_cmd::exec_command(cmd, &commands) {
        Ok(is_internal) => {
            if is_internal && cmd == "restart_server" {
                let app = state.app_handle.clone();
                let state_clone = state.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = restart_server(app, state_clone).await;
                });
            }

            log_request(
                state,
                method,
                "/exec",
                200,
                started.elapsed().as_millis() as u64,
                &addr.ip().to_string(),
            );
            (
                StatusCode::OK,
                Json(SuccessResponse {
                    ok: true,
                    message: format!("executed command '{cmd}'"),
                }),
            )
                .into_response()
        }
        Err(err) => error_response(state, method, "/exec", started, addr, err),
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
