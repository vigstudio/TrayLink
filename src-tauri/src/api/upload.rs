use std::time::Instant;

use axum::extract::{Multipart, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::Json;
use serde::Serialize;

use crate::api::auth::extract_token;
use crate::api::client_ip::ClientIp;
use crate::api::routes::log_request;
use crate::launcher::open_file;
use crate::state::SharedState;
use crate::uploads;

#[derive(Serialize)]
struct UploadResponse {
    ok: bool,
    message: String,
    filename: String,
    path: String,
    size: u64,
}

pub fn routes() -> axum::Router<SharedState> {
    axum::Router::new().route("/api/upload", post(upload_file))
}

async fn upload_file(
    State(state): State<SharedState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let started = Instant::now();

    if let Err(resp) = extract_token(&headers, None, &state) {
        log_request(
            &state,
            "POST",
            "/api/upload",
            401,
            started.elapsed().as_millis() as u64,
            &ip,
        );
        return resp;
    }

    let mut file_name: Option<String> = None;
    let mut file_data: Option<Vec<u8>> = None;
    let mut open_after = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file" => {
                file_name = field.file_name().map(|s| s.to_string());
                match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(err) => {
                        return upload_error(
                            &state,
                            started,
                            &ip,
                            StatusCode::BAD_REQUEST,
                            &format!("Không đọc được file: {err}"),
                        );
                    }
                }
            }
            "open" => {
                if let Ok(value) = field.text().await {
                    open_after = value == "1" || value.eq_ignore_ascii_case("true");
                }
            }
            _ => {}
        }
    }

    let original_name = match file_name {
        Some(name) if !name.is_empty() => name,
        _ => {
            return upload_error(
                &state,
                started,
                &ip,
                StatusCode::BAD_REQUEST,
                "Thiếu file (field 'file')",
            );
        }
    };

    let data = match file_data {
        Some(data) => data,
        None => {
            return upload_error(
                &state,
                started,
                &ip,
                StatusCode::BAD_REQUEST,
                "Nội dung file trống",
            );
        }
    };

    let size = data.len() as u64;
    match uploads::save_upload(&state.app_handle, &original_name, &data) {
        Ok((path, filename)) => {
            let path_str = path.to_string_lossy().into_owned();
            let mut message = format!("Đã lưu vào Downloads/TrayLink/{filename}");

            if open_after {
                if let Err(err) = open_file::open_file(&path_str) {
                    message = format!("{message} (không mở được: {err})");
                } else {
                    message = format!("{message} — đã mở file");
                }
            }

            log_request(
                &state,
                "POST",
                "/api/upload",
                200,
                started.elapsed().as_millis() as u64,
                &ip,
            );

            (
                StatusCode::OK,
                Json(UploadResponse {
                    ok: true,
                    message,
                    filename,
                    path: path_str,
                    size,
                }),
            )
                .into_response()
        }
        Err(err) => upload_error(
            &state,
            started,
            &ip,
            StatusCode::BAD_REQUEST,
            &err,
        ),
    }
}

fn upload_error(
    state: &SharedState,
    started: Instant,
    client_ip: &str,
    status: StatusCode,
    message: &str,
) -> Response {
    log_request(
        state,
        "POST",
        "/api/upload",
        status.as_u16(),
        started.elapsed().as_millis() as u64,
        client_ip,
    );
    (
        status,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}
