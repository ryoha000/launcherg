use std::sync::Arc;

use anyhow::{Context, Result};
use domain::{
    pubsub::PubSubService,
    service::app_signal_router::{AppSignal, AppSignalEvent},
};
use interprocess::local_socket::{
    tokio::{Listener as TokioListener, Stream},
    traits::tokio::Listener as _,
    ListenerOptions,
};
use tokio::{fs, io::AsyncReadExt};

use crate::app_signal_router::{endpoint::AppSignalEndpoint, APP_SIGNAL_EVENT, APP_SIGNAL_SYNC_REQUESTED_EVENT};

pub fn spawn_listener<P>(pubsub: Arc<P>) -> Result<()>
where
    P: PubSubService + 'static,
{
    let config = AppSignalEndpoint::prepare_listener()?;

    tauri::async_runtime::spawn(async move {
        if let Err(err) = run_listener(config.options, config.cleanup_path, pubsub).await {
            log::error!("app signal listener stopped: {err}");
        }
    });

    Ok(())
}

async fn run_listener<P>(
    options: ListenerOptions<'static>,
    cleanup_path: Option<std::path::PathBuf>,
    pubsub: Arc<P>,
) -> Result<()>
where
    P: PubSubService + 'static,
{
    let listener: TokioListener = options
        .create_tokio()
        .with_context(|| "failed to bind app signal endpoint")?;
    log::info!("app signal listener started");

    let result: Result<()> = loop {
        match listener.accept().await {
            Ok(stream) => {
                let pubsub = Arc::clone(&pubsub);
                tauri::async_runtime::spawn(async move {
                    if let Err(err) = handle_stream(stream, pubsub).await {
                        log::warn!("app signal handling failed: {err}");
                    }
                });
            }
            Err(err) => {
                log::error!("failed to accept app signal connection: {err}");
                break Err(err.into());
            }
        }
    };

    if let Some(path) = cleanup_path {
        if let Err(err) = fs::remove_file(&path).await {
            if err.kind() != std::io::ErrorKind::NotFound {
                log::debug!("failed to remove app signal socket {}: {err}", path.display());
            }
        }
    }

    result
}

async fn handle_stream<P>(mut stream: Stream, pubsub: Arc<P>) -> Result<()>
where
    P: PubSubService + 'static,
{
    let signal = read_signal(&mut stream).await?;

    pubsub
        .notify(APP_SIGNAL_EVENT, signal.clone())
        .with_context(|| format!("failed to emit {APP_SIGNAL_EVENT}"))?;

    if matches!(&signal.event, AppSignalEvent::SyncRequested { .. }) {
        pubsub
            .notify(APP_SIGNAL_SYNC_REQUESTED_EVENT, signal.clone())
            .with_context(|| format!("failed to emit {APP_SIGNAL_SYNC_REQUESTED_EVENT}"))?;
    }

    Ok(())
}

async fn read_signal(stream: &mut Stream) -> Result<AppSignal> {
    let mut len_buf = [0u8; 4];
    stream
        .read_exact(&mut len_buf)
        .await
        .with_context(|| "failed to read message length")?;
    let expected = u32::from_le_bytes(len_buf) as usize;
    let mut payload = vec![0u8; expected];
    stream
        .read_exact(&mut payload)
        .await
        .with_context(|| "failed to read app signal payload")?;

    let signal: AppSignal = serde_json::from_slice(&payload)
        .with_context(|| "failed to deserialize app signal")?;
    Ok(signal)
}
