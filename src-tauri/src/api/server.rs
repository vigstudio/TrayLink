use std::net::SocketAddr;

use axum::serve;
use tauri::AppHandle;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::api::routes::build_router;
use crate::state::{ServerHandle, SharedState};

pub async fn start_server(app: AppHandle, state: SharedState) -> Result<(), String> {
    stop_server(&state).await;

    let port = state.config.read().unwrap().port;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(err) => {
            let message = format!("failed to bind 127.0.0.1:{port}: {err}");
            *state.server_error.write().unwrap() = Some(message.clone());
            *state.server_started_at.write().unwrap() = None;
            return Err(message);
        }
    };

    *state.server_error.write().unwrap() = None;
    *state.server_started_at.write().unwrap() = Some(std::time::Instant::now());

    let router = build_router(state.clone());
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    {
        let mut guard = state.server.lock().await;
        *guard = Some(ServerHandle {
            shutdown_tx: Some(shutdown_tx),
        });
    }

    tauri::async_runtime::spawn(async move {
        if let Err(err) = serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await
        {
            eprintln!("API server error: {err}");
        }
    });

    let _ = app;
    Ok(())
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
        if let Some(tx) = handle.shutdown_tx {
            let _ = tx.send(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    *state.server_started_at.write().unwrap() = None;
}
