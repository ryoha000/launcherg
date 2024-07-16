use std::sync::{Arc, Mutex};

use anyhow;
use axum::{routing::get, Router};
use tauri::AppHandle;
use tokio::{net::TcpListener, signal, sync::Notify};

use crate::infrastructure::{
    explorerimpl::explorer::Explorers,
    repositoryimpl::{driver::Db, repository::Repositories},
    windowsimpl::windows::Windows,
};

use super::{handler, module::Modules};

pub async fn serve(handle: AppHandle) -> anyhow::Result<Arc<Notify>> {
    let shutdown_notify = Arc::new(Notify::new());
    let shutdown_notify_clone = shutdown_notify.clone();

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    let db = Db::new(&handle).await;
    let modules = Arc::new(Modules::new(
        Mutex::new(None),
        Arc::new(handle),
        Arc::new(Repositories::new(db.clone())),
        Arc::new(Windows::new()),
        Arc::new(Explorers::new()),
    ));

    let hc_router = Router::new().route("/", get(handler::health_check::hc));
    let games_router = Router::new().route("/", get(handler::games::list_games));

    let current_router = Router::new().route("/screenshot", get(handler::current::get_screenshot));

    let app = Router::new()
        .nest("/hc", hc_router)
        .nest("/games", games_router)
        .nest("/current", current_router)
        .with_state(modules);

    tauri::async_runtime::spawn(async {
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(shutdown_notify_clone))
            .await
            .unwrap();
    });

    Ok(shutdown_notify)
}

async fn shutdown_signal(shutdown_notify: Arc<Notify>) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    let shutdown = async {
        shutdown_notify.notified().await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = shutdown => {},
    }
}
