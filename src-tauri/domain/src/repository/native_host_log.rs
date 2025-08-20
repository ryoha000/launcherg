#[cfg(any(test, feature = "mocks"))]
use mockall::automock;
use crate::native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow};

#[trait_variant::make(Send)]
#[cfg_attr(any(test, feature = "mocks"), automock)]
pub trait NativeHostLogRepository {
    async fn insert_log(&self, level: HostLogLevel, typ: HostLogType, message: &str) -> anyhow::Result<()>;
    async fn list_logs(
        &self,
        limit: i64,
        offset: i64,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<Vec<NativeHostLogRow>>;
    async fn count_logs(
        &self,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<i64>;
}


