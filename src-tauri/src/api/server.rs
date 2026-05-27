use std::net::SocketAddr;
use std::time::Duration;

use axum::serve;
use axum_server::tls_rustls::RustlsConfig;
use axum_server::Handle;
use tauri::AppHandle;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::api::routes::build_router;
use crate::net;
use crate::state::{ServerHandle, SharedState};
use crate::tls;

pub fn https_port(http_port: u16) -> u16 {
    http_port.saturating_add(1)
}

pub async fn start_server(app: AppHandle, state: SharedState) -> Result<(), String> {
    stop_server(&state).await;

    let port = state.config.read().unwrap().port;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            let message = format!("failed to bind 0.0.0.0:{port}: {err}");
            *state.server_error.write().unwrap() = Some(message.clone());
            *state.server_started_at.write().unwrap() = None;
            return Err(message);
        }
    };

    *state.server_error.write().unwrap() = None;
    *state.server_started_at.write().unwrap() = Some(std::time::Instant::now());

    let http_router = build_router(state.clone());
    let https_router = build_router(state.clone());
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    let https_handle = start_https_server(&app, state.clone(), https_router).await;

    {
        let mut guard = state.server.lock().await;
        *guard = Some(ServerHandle {
            shutdown_tx: Some(shutdown_tx),
            https_shutdown: https_handle,
        });
    }

    tauri::async_runtime::spawn(async move {
        if let Err(err) = serve(listener, http_router)
            .with_graceful_shutdown(async {
                let _ = shutdown_rx.await;
            })
            .await
        {
            eprintln!("API server error: {err}");
        }
    });

    Ok(())
}

async fn start_https_server(
    app: &AppHandle,
    state: SharedState,
    router: axum::Router,
) -> Option<Handle> {
    let http_port = state.config.read().unwrap().port;
    let https_port = https_port(http_port);
    let lan_ip = net::get_lan_ip();
    let materials = match tls::load_or_create_tls_materials(app, lan_ip.as_deref()) {
        Ok(m) => m,
        Err(err) => {
            eprintln!("HTTPS cert error: {err}");
            return None;
        }
    };

    let rustls_config = match RustlsConfig::from_pem(materials.cert_pem, materials.key_pem).await {
        Ok(c) => c,
        Err(err) => {
            eprintln!("HTTPS rustls config error: {err}");
            return None;
        }
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], https_port));
    let handle = Handle::new();
    let serve_handle = handle.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(err) = axum_server::bind_rustls(addr, rustls_config)
            .handle(serve_handle)
            .serve(router.into_make_service())
            .await
        {
            eprintln!("HTTPS API server error: {err}");
        }
    });

    let _ = app;
    eprintln!("TrayLink HTTPS listening on 0.0.0.0:{https_port}");
    Some(handle)
}

pub async fn restart_server(app: AppHandle, state: SharedState) -> Result<(), String> {
    stop_server(&state).await;
    start_server(app, state).await
}

async fn stop_server(state: &SharedState) {
    let handle = {
        let mut guard = state.server.lock().await;
        guard.take()
    };

    if let Some(handle) = handle {
        if let Some(https) = handle.https_shutdown {
            https.graceful_shutdown(Some(Duration::from_millis(400)));
        }
        if let Some(tx) = handle.shutdown_tx {
            let _ = tx.send(());
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }

    *state.server_started_at.write().unwrap() = None;
}
