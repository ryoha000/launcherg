#[cfg(test)]
mod tests {
    use super::super::{handle_stream, read_signal};
    use crate::app_signal_router::test_support::{test_lock, RecordingPubSub, TestEndpoint};
    use crate::app_signal_router::{APP_SIGNAL_EVENT, APP_SIGNAL_SYNC_REQUESTED_EVENT};
    use anyhow::{Error, Result};
    use chrono::Utc;
    use domain::service::app_signal_router::{AppSignal, AppSignalEvent, AppSignalSource};
    use interprocess::local_socket::{
        tokio::Stream,
        traits::tokio::{Listener as _, Stream as _},
    };
    use serde_json::from_value;
    use std::future::Future;
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;

    fn build_signal(message: Option<&str>) -> AppSignal {
        AppSignal {
            source: AppSignalSource::NativeMessagingHost,
            event: AppSignalEvent::SyncRequested {
                message: message.map(|msg| msg.to_string()),
            },
            issued_at: Utc::now(),
        }
    }

    async fn run_pair<F, Fut>(endpoint: &TestEndpoint, writer: F) -> Result<(Stream, Fut)>
    where
        F: FnOnce(Stream) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        let listener = endpoint.listener_options().create_tokio()?;
        let name = endpoint.clone_name();
        let connect_name = name.clone();

        let (server_stream, client_stream) = tokio::try_join!(
            async move { listener.accept().await.map_err(Error::from) },
            async move { Stream::connect(connect_name).await.map_err(Error::from) }
        )?;
        let output = writer(client_stream);
        Ok((server_stream, output))
    }

    #[tokio::test]
    async fn read_signal_正常入力を復元する() -> Result<()> {
        let _lock = test_lock();
        let endpoint = TestEndpoint::new()?;
        let signal = build_signal(Some("同期要求"));
        let payload = serde_json::to_vec(&signal)?;
        let len = (payload.len() as u32).to_le_bytes();

        let (mut server_stream, writer_handle) =
            run_pair(&endpoint, move |mut client_stream| async move {
                client_stream.write_all(&len).await?;
                client_stream.write_all(&payload).await?;
                client_stream.flush().await?;
                Result::<()>::Ok(())
            })
            .await?;

        writer_handle.await?;

        let received = read_signal(&mut server_stream).await?;
        assert_eq!(received, signal);
        Ok(())
    }

    #[tokio::test]
    async fn read_signal_長さ不足でエラーになる() -> Result<()> {
        let _lock = test_lock();
        let endpoint = TestEndpoint::new()?;
        let payload = b"{\"dummy\":true}".to_vec();
        let len = (payload.len() as u32).to_le_bytes();

        let (mut server_stream, writer_handle) =
            run_pair(&endpoint, move |mut client_stream| async move {
                client_stream.write_all(&len).await?;
                client_stream.write_all(&payload[..2]).await?;
                client_stream.flush().await?;
                Result::<()>::Ok(())
            })
            .await?;

        writer_handle.await?;

        let err = read_signal(&mut server_stream)
            .await
            .expect_err("expected failure");
        assert!(err
            .to_string()
            .contains("failed to read app signal payload"));
        Ok(())
    }

    #[tokio::test]
    async fn handle_stream_sync_同期イベントを二重配送する() -> Result<()> {
        let _lock = test_lock();
        let endpoint = TestEndpoint::new()?;
        let signal = build_signal(Some("sync"));
        let payload = serde_json::to_vec(&signal)?;
        let len = (payload.len() as u32).to_le_bytes();

        let (server_stream, writer_handle) =
            run_pair(&endpoint, move |mut client_stream| async move {
                client_stream.write_all(&len).await?;
                client_stream.write_all(&payload).await?;
                client_stream.flush().await?;
                Result::<()>::Ok(())
            })
            .await?;

        writer_handle.await?;

        let pubsub = Arc::new(RecordingPubSub::new());
        handle_stream(server_stream, Arc::clone(&pubsub)).await?;

        let events = pubsub.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].0, APP_SIGNAL_EVENT);
        assert_eq!(events[1].0, APP_SIGNAL_SYNC_REQUESTED_EVENT);
        let first_signal: AppSignal = from_value(events[0].1.clone())?;
        let second_signal: AppSignal = from_value(events[1].1.clone())?;
        assert_eq!(first_signal, signal);
        assert_eq!(second_signal, signal);

        Ok(())
    }

    #[tokio::test]
    async fn handle_stream_notify失敗時_文脈付きエラーを返す() -> Result<()> {
        let _lock = test_lock();
        let endpoint = TestEndpoint::new()?;
        let signal = build_signal(None);
        let payload = serde_json::to_vec(&signal)?;
        let len = (payload.len() as u32).to_le_bytes();

        let (server_stream, writer_handle) =
            run_pair(&endpoint, move |mut client_stream| async move {
                client_stream.write_all(&len).await?;
                client_stream.write_all(&payload).await?;
                client_stream.flush().await?;
                Result::<()>::Ok(())
            })
            .await?;

        writer_handle.await?;

        let pubsub = Arc::new(RecordingPubSub::failing(APP_SIGNAL_EVENT));
        let err = handle_stream(server_stream, pubsub)
            .await
            .expect_err("expected failure");
        assert!(err.to_string().contains("failed to emit appSignal"));
        Ok(())
    }
}
