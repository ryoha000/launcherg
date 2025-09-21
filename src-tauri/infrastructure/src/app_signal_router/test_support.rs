use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(not(windows))]
use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use interprocess::local_socket::{ListenerOptions, Name};
use uuid::Uuid;

#[cfg(not(windows))]
use interprocess::local_socket::{GenericFilePath, ToFsName};
#[cfg(windows)]
use interprocess::local_socket::{GenericNamespaced, ToNsName};

use domain::pubsub::PubSubService;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::Notify;

/// グローバルに共有するテスト用ロック。
pub(crate) fn test_lock() -> MutexGuard<'static, ()> {
    static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("poisoned test lock")
}

#[cfg(not(windows))]
const ENV_KEYS: &[&str] = &["TMPDIR", "TEMP", "TMP"];

/// `std::env::temp_dir()` が参照するディレクトリをテスト毎に切り替えるためのガード。
#[cfg(not(windows))]
pub(crate) struct TempDirEnvGuard {
    originals: Vec<(&'static str, Option<OsString>)>,
    dir: PathBuf,
}

#[cfg(not(windows))]
impl TempDirEnvGuard {
    pub(crate) fn new() -> Result<Self> {
        let dir = tempfile::tempdir()
            .context("temporary directory for test could not be created")?
            .into_path();
        let mut originals = Vec::with_capacity(ENV_KEYS.len());
        for &key in ENV_KEYS {
            let original = std::env::var_os(key);
            std::env::set_var(key, &dir);
            originals.push((key, original));
        }
        Ok(Self { originals, dir })
    }

    pub(crate) fn base_dir(&self) -> &Path {
        &self.dir
    }
}

#[cfg(not(windows))]
impl Drop for TempDirEnvGuard {
    fn drop(&mut self) {
        for (key, original) in &self.originals {
            match original {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(not(windows))]
pub(crate) struct TestEndpoint {
    name: Name<'static>,
    socket_path: PathBuf,
    base_dir: PathBuf,
}

#[cfg(not(windows))]
impl TestEndpoint {
    pub(crate) fn new() -> Result<Self> {
        let base_dir = std::env::temp_dir().join(format!(
            "launcherg-test-{}",
            Uuid::new_v4().simple().to_string()
        ));
        std::fs::create_dir_all(&base_dir)?;
        let socket_path = base_dir.join("app-signal.sock");
        let name = socket_path
            .to_fs_name::<GenericFilePath>()
            .context("failed to convert socket path to local socket name")?
            .into_owned();
        Ok(Self {
            name,
            socket_path,
            base_dir,
        })
    }

    pub(crate) fn clone_name(&self) -> Name<'static> {
        self.name.clone()
    }

    pub(crate) fn listener_options(&self) -> ListenerOptions<'static> {
        ListenerOptions::new().name(self.clone_name())
    }

    pub(crate) fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

#[cfg(not(windows))]
impl Drop for TestEndpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
        let _ = std::fs::remove_dir_all(&self.base_dir);
    }
}

#[cfg(windows)]
pub(crate) struct TestEndpoint {
    name: Name<'static>,
}

#[cfg(windows)]
impl TestEndpoint {
    pub(crate) fn new() -> Result<Self> {
        let unique = format!(
            "launcherg_app_signal_test_{}",
            Uuid::new_v4().simple().to_string()
        );
        let name = unique
            .to_ns_name::<GenericNamespaced>()
            .context("failed to build namespaced pipe name")?
            .into_owned();
        Ok(Self { name })
    }

    pub(crate) fn clone_name(&self) -> Name<'static> {
        self.name.clone()
    }

    pub(crate) fn listener_options(&self) -> ListenerOptions<'static> {
        ListenerOptions::new().name(self.clone_name())
    }
}

/// `PubSubService` の通知内容を記録するテスト用実装。
pub(crate) struct RecordingPubSub {
    events: Mutex<Vec<(String, Value)>>,
    fail_for: Option<String>,
    notifier: Notify,
}

impl RecordingPubSub {
    pub(crate) fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            fail_for: None,
            notifier: Notify::new(),
        }
    }

    pub(crate) fn failing(event: &str) -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            fail_for: Some(event.to_string()),
            notifier: Notify::new(),
        }
    }

    pub(crate) fn events(&self) -> Vec<(String, Value)> {
        self.events.lock().expect("poisoned events").clone()
    }

    #[cfg(not(windows))]
    pub(crate) async fn wait_for_events(&self, expected: usize) -> Result<Vec<(String, Value)>> {
        use tokio::time::{Duration, Instant};

        let deadline = Instant::now() + Duration::from_secs(1);
        loop {
            if let Some(events) = {
                let guard = self.events.lock().expect("poisoned events");
                if guard.len() >= expected {
                    Some(guard.clone())
                } else {
                    None
                }
            } {
                return Ok(events);
            }

            let now = Instant::now();
            if now >= deadline {
                anyhow::bail!("timed out waiting for events");
            }
            let remaining = deadline.saturating_duration_since(now);
            tokio::select! {
                _ = self.notifier.notified() => {}
                _ = tokio::time::sleep(remaining) => anyhow::bail!("timed out waiting for events"),
            }
        }
    }
}

impl Default for RecordingPubSub {
    fn default() -> Self {
        Self::new()
    }
}

impl PubSubService for RecordingPubSub {
    fn notify<T: Serialize + Clone>(&self, event: &str, payload: T) -> Result<(), anyhow::Error> {
        if self.fail_for.as_deref() == Some(event) {
            return Err(anyhow::anyhow!("forced failure for {event}"));
        }
        let value = serde_json::to_value(payload)?;
        {
            let mut events = self.events.lock().expect("poisoned events");
            events.push((event.to_string(), value));
        }
        self.notifier.notify_waiters();
        Ok(())
    }
}
