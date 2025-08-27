use std::sync::Arc;

use derive_new::new;
use domain::{
    native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow},
    repository::{native_host_log::NativeHostLogRepository, RepositoriesExt},
};

#[derive(new)]
pub struct HostLogUseCase<R: RepositoriesExt> {
    repositories: Arc<R>,
}

impl<R: RepositoriesExt> HostLogUseCase<R> {
    pub async fn list_logs(
        &self,
        limit: i64,
        offset: i64,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<Vec<NativeHostLogRow>> {
        let repo = self.repositories.host_log_repository();
        repo.list_logs(limit, offset, level, typ).await
    }

    pub async fn count_logs(
        &self,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<i64> {
        let repo = self.repositories.host_log_repository();
        repo.count_logs(level, typ).await
    }
}


