use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use crate::domain::native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait NativeHostLogRepository: Send + Sync {
    async fn insert_log(&self, level: HostLogLevel, typ: HostLogType, message: &str) -> anyhow::Result<()>;
}


