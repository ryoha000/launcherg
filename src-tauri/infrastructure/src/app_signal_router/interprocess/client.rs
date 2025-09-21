use anyhow::Context;
use domain::service::app_signal_router::{AppSignal, AppSignalRouter};
use interprocess::local_socket::{tokio::Stream, traits::tokio::Stream as _};

use crate::app_signal_router::endpoint::AppSignalEndpoint;

/// `interprocess` を利用したアプリシグナル送信クライアント（実装は後続タスクで拡張）。
pub struct InterprocessAppSignalRouter;

impl AppSignalRouter for InterprocessAppSignalRouter {
    async fn dispatch(&self, signal: AppSignal) -> anyhow::Result<()> {
        let name = AppSignalEndpoint::connect_name()?;
        let mut stream = Stream::connect(name)
            .await
            .with_context(|| "failed to connect to app signal endpoint")?;

        send_signal(&mut stream, signal).await
    }
}

async fn send_signal(stream: &mut Stream, signal: AppSignal) -> anyhow::Result<()> {
    let payload = serde_json::to_vec(&signal)?;
    let len = (payload.len() as u32).to_le_bytes();
    use tokio::io::AsyncWriteExt;
    stream.write_all(&len).await?;
    stream.write_all(&payload).await?;
    stream.flush().await?;
    Ok(())
}

impl InterprocessAppSignalRouter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for InterprocessAppSignalRouter {
    fn default() -> Self {
        Self
    }
}
