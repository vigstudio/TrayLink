use std::convert::Infallible;
use std::net::SocketAddr;

use axum::extract::ConnectInfo;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::HeaderMap;

pub struct ClientIp(pub String);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Send + Sync,
{
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(ClientIp(resolve_client_ip(&parts.headers, &parts.extensions)))
    }
}

pub fn resolve_client_ip(headers: &HeaderMap, extensions: &axum::http::Extensions) -> String {
    if let Some(ConnectInfo(addr)) = extensions.get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }

    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(String::from)
        .unwrap_or_else(|| "unknown".to_string())
}
