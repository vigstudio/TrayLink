use std::path::Path;

use axum::extract::{Path as AxumPath, State};
use axum::http::header;
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::Json;
use serde::Serialize;

use crate::apps::get_app_icon_png;
use crate::config::{AppEntry, RemoteDeckLayout};
use crate::remote_icons::{content_type_for_filename, load_custom_icon, slot_id};
use crate::state::{SharedState, APP_VERSION};

const REMOTE_HTML: &str = include_str!("../../assets/remote.html");

#[derive(Serialize)]
struct DeckItem {
    #[serde(rename = "type")]
    item_type: String,
    key: String,
    label: String,
    icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[derive(Serialize)]
struct DeckResponse {
    version: &'static str,
    allow_get: bool,
    require_token: bool,
    items: Vec<DeckItem>,
}

pub fn routes() -> axum::Router<SharedState> {
    axum::Router::new()
        .route("/", get(remote_page))
        .route("/remote", get(remote_page))
        .route("/api/deck", get(deck_api))
        .route("/api/icons/{kind}/{key}", get(deck_icon))
        .route("/api/icons/{key}", get(legacy_app_icon))
}

async fn remote_page() -> Html<&'static str> {
    Html(REMOTE_HTML)
}

async fn deck_api(State(state): State<SharedState>) -> Json<DeckResponse> {
    let config = state.config.read().unwrap();
    let layout = &config.remote_deck;

    let deck_entries = RemoteDeckLayout::deck_items(
        config.apps.keys().map(String::as_str),
        config.commands.keys().map(String::as_str),
        layout,
    );

    let items: Vec<DeckItem> = deck_entries
        .into_iter()
        .filter_map(|(item_type, key)| match item_type.as_str() {
            "app" => config.apps.get(&key).map(|entry| DeckItem {
                item_type: "app".to_string(),
                key: key.clone(),
                label: app_display_name(&key, entry),
                icon: Some(format!("/api/icons/app/{key}")),
                url: entry.url.clone(),
            }),
            "cmd" => Some(DeckItem {
                item_type: "cmd".to_string(),
                key: key.clone(),
                label: key.replace('_', " "),
                icon: Some(format!("/api/icons/cmd/{key}")),
                url: None,
            }),
            _ => None,
        })
        .collect();

    Json(DeckResponse {
        version: APP_VERSION,
        allow_get: config.allow_get,
        require_token: config.require_token,
        items,
    })
}

async fn deck_icon(
    State(state): State<SharedState>,
    AxumPath((kind, key)): AxumPath<(String, String)>,
) -> Response {
    let config = state.config.read().unwrap();
    let slot = slot_id(&kind, &key);

    if let Some(filename) = config.remote_deck.custom_icons.get(&slot) {
        if let Some(bytes) = load_custom_icon(&state.app_handle, filename) {
            let content_type = content_type_for_filename(filename);
            return (
                [(
                    header::CONTENT_TYPE,
                    content_type,
                ), (header::CACHE_CONTROL, "public, max-age=3600")],
                bytes,
            )
                .into_response();
        }
    }

    if kind == "app" {
        if let Some(entry) = config.apps.get(&key) {
            if let Some(bytes) = get_app_icon_png(&entry.path) {
                return (
                    [(
                        header::CONTENT_TYPE,
                        "image/png",
                    ), (header::CACHE_CONTROL, "public, max-age=3600")],
                    bytes,
                )
                    .into_response();
            }
        }
    }

    default_icon_response(&kind)
}

fn default_icon_response(kind: &str) -> Response {
    let svg = if kind == "cmd" {
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#c8c8d8" stroke-width="1.5"><polyline points="4 17 10 11 4 5"/><line x1="12" y1="19" x2="20" y2="19"/></svg>"##
    } else {
        r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="#c8c8d8" stroke-width="1.5"><rect x="3" y="3" width="18" height="18" rx="3"/><path d="M8 12h8M12 8v8"/></svg>"##
    };

    (
        [(
            header::CONTENT_TYPE,
            "image/svg+xml",
        ), (header::CACHE_CONTROL, "public, max-age=86400")],
        svg,
    )
        .into_response()
}

async fn legacy_app_icon(
    State(state): State<SharedState>,
    AxumPath(key): AxumPath<String>,
) -> Response {
    deck_icon(State(state), AxumPath(("app".to_string(), key))).await
}

fn app_display_name(key: &str, entry: &AppEntry) -> String {
    if let Some(name) = &entry.name {
        let trimmed = name.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let path_obj = Path::new(&entry.path);
    if let Some(stem) = path_obj.file_stem() {
        let name = stem.to_string_lossy();
        if !name.is_empty() && name != key {
            return name.to_string();
        }
    }
    key.replace('_', " ")
}
