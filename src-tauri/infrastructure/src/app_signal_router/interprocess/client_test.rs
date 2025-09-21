#[cfg(test)]
mod tests {
    use super::super::{send_signal, InterprocessAppSignalRouter};
    use crate::app_signal_router::test_support::{test_lock, TestEndpoint};
    use anyhow::{Error, Result};
    use chrono::Utc;
    #[cfg(not(windows))]
    use domain::pubsub::PubSubEvent;
    use domain::service::app_signal_router::{
        AppSignal, AppSignalEvent, AppSignalRouter, AppSignalSource,
    };
    use interprocess::local_socket::{
        tokio::Stream,
        traits::tokio::{Listener as _, Stream as _},
    };
    use serde_json::from_slice;
    use tokio::io::AsyncReadExt;

    #[cfg(not(windows))]
    use crate::app_signal_router::{
        interprocess::listener::spawn_listener,
        test_support::{RecordingPubSub, TempDirEnvGuard},
        APP_SIGNAL_EVENT, APP_SIGNAL_REFETCH_WORKS_EVENT, APP_SIGNAL_REFETCH_WORK_EVENT,
        APP_SIGNAL_SHOW_ERROR_MESSAGE_EVENT, APP_SIGNAL_SHOW_MESSAGE_EVENT,
    };
    #[cfg(not(windows))]
    use std::sync::Arc;
    #[cfg(not(windows))]
    use tokio::time::{sleep, Duration};

    async fn setup_stream_pair(endpoint: &TestEndpoint) -> Result<(Stream, Stream)> {
        let listener = endpoint.listener_options().create_tokio()?;
        let name = endpoint.clone_name();
        let connect_name = name.clone();

        tokio::try_join!(
            async move { listener.accept().await.map_err(Error::from) },
            async move { Stream::connect(connect_name).await.map_err(Error::from) }
        )
    }

    fn sample_signal() -> AppSignal {
        AppSignal {
            source: AppSignalSource::NativeMessagingHost,
            event: AppSignalEvent::ShowMessage {
                message: "integration".to_string(),
            },
            issued_at: Utc::now(),
        }
    }

    #[cfg(not(windows))]
    fn sample_refetch_works_signal() -> AppSignal {
        AppSignal {
            source: AppSignalSource::NativeMessagingHost,
            event: AppSignalEvent::RefetchWorks,
            issued_at: Utc::now(),
        }
    }

    #[cfg(not(windows))]
    fn sample_refetch_work_signal(work_id: i32) -> AppSignal {
        AppSignal {
            source: AppSignalSource::NativeMessagingHost,
            event: AppSignalEvent::RefetchWork { work_id },
            issued_at: Utc::now(),
        }
    }

    #[cfg(not(windows))]
    fn sample_error_signal() -> AppSignal {
        AppSignal {
            source: AppSignalSource::NativeMessagingHost,
            event: AppSignalEvent::ShowErrorMessage {
                message: "integration".to_string(),
            },
            issued_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn send_signal_jsonと長さを書き込む() -> Result<()> {
        let _lock = test_lock();
        let endpoint = TestEndpoint::new()?;
        let (mut server_stream, mut client_stream) = setup_stream_pair(&endpoint).await?;
        let signal = sample_signal();

        send_signal(&mut client_stream, signal.clone()).await?;

        let mut len_buf = [0u8; 4];
        server_stream.read_exact(&mut len_buf).await?;
        let expected = u32::from_le_bytes(len_buf) as usize;
        let mut payload = vec![0u8; expected];
        server_stream.read_exact(&mut payload).await?;
        let restored: AppSignal = from_slice(&payload)?;
        assert_eq!(restored, signal);
        Ok(())
    }

    #[tokio::test]
    async fn dispatch_接続不能時に文脈付きエラーを返す() -> Result<()> {
        let _lock = test_lock();
        #[cfg(not(windows))]
        let _env = TempDirEnvGuard::new()?;

        let router = InterprocessAppSignalRouter::new();
        let err = router
            .dispatch(sample_signal())
            .await
            .expect_err("expected connection failure");
        assert!(err
            .to_string()
            .contains("failed to connect to app signal endpoint"));
        Ok(())
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn listener_client_統合テスト() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;
        let pubsub = Arc::new(RecordingPubSub::new());
        spawn_listener(Arc::clone(&pubsub))?;

        sleep(Duration::from_millis(50)).await;

        let router = InterprocessAppSignalRouter::new();
        let signal = sample_signal();
        router.dispatch(signal.clone()).await?;

        let events = pubsub.wait_for_events(2).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name(), APP_SIGNAL_EVENT);
        assert_eq!(events[1].event_name(), APP_SIGNAL_SHOW_MESSAGE_EVENT);

        let first: AppSignal = match &events[0] {
            PubSubEvent::AppSignal(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        let second: AppSignal = match &events[1] {
            PubSubEvent::AppSignalShowMessage(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        assert_eq!(first, signal);
        assert_eq!(second, signal);

        Ok(())
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn listener_client_リフェッチワークスイベント統合テスト() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;
        let pubsub = Arc::new(RecordingPubSub::new());
        spawn_listener(Arc::clone(&pubsub))?;

        sleep(Duration::from_millis(50)).await;

        let router = InterprocessAppSignalRouter::new();
        let signal = sample_refetch_works_signal();
        router.dispatch(signal.clone()).await?;

        let events = pubsub.wait_for_events(2).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name(), APP_SIGNAL_EVENT);
        assert_eq!(events[1].event_name(), APP_SIGNAL_REFETCH_WORKS_EVENT);

        let first: AppSignal = match &events[0] {
            PubSubEvent::AppSignal(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        let second: AppSignal = match &events[1] {
            PubSubEvent::AppSignalRefetchWorks(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        assert_eq!(first, signal);
        assert_eq!(second, signal);

        drop(env_guard);
        Ok(())
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn listener_client_リフェッチワークイベント統合テスト() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;
        let pubsub = Arc::new(RecordingPubSub::new());
        spawn_listener(Arc::clone(&pubsub))?;

        sleep(Duration::from_millis(50)).await;

        let router = InterprocessAppSignalRouter::new();
        let signal = sample_refetch_work_signal(42);
        router.dispatch(signal.clone()).await?;

        let events = pubsub.wait_for_events(2).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name(), APP_SIGNAL_EVENT);
        assert_eq!(events[1].event_name(), APP_SIGNAL_REFETCH_WORK_EVENT);

        let first: AppSignal = match &events[0] {
            PubSubEvent::AppSignal(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        let second: AppSignal = match &events[1] {
            PubSubEvent::AppSignalRefetchWork(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        assert_eq!(first, signal);
        assert_eq!(second, signal);

        drop(env_guard);
        Ok(())
    }

    #[cfg(not(windows))]
    #[tokio::test]
    async fn listener_client_エラーイベント統合テスト() -> Result<()> {
        let _lock = test_lock();
        let env_guard = TempDirEnvGuard::new()?;
        let pubsub = Arc::new(RecordingPubSub::new());
        spawn_listener(Arc::clone(&pubsub))?;

        sleep(Duration::from_millis(50)).await;

        let router = InterprocessAppSignalRouter::new();
        let signal = sample_error_signal();
        router.dispatch(signal.clone()).await?;

        let events = pubsub.wait_for_events(2).await?;
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_name(), APP_SIGNAL_EVENT);
        assert_eq!(events[1].event_name(), APP_SIGNAL_SHOW_ERROR_MESSAGE_EVENT);

        let first: AppSignal = match &events[0] {
            PubSubEvent::AppSignal(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        let second: AppSignal = match &events[1] {
            PubSubEvent::AppSignalShowErrorMessage(payload) => payload.clone().into(),
            _ => unreachable!(),
        };
        assert_eq!(first, signal);
        assert_eq!(second, signal);

        drop(env_guard);
        Ok(())
    }
}
