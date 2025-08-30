use std::sync::Arc;

use derive_new::new;
use domain::{
    native_host_log::{HostLogLevel, HostLogType, NativeHostLogRow},
};
use domain::repository::{native_host_log::NativeHostLogRepository, RepositoriesExt, manager::RepositoryManager};
use std::marker::PhantomData;

#[derive(new)]
pub struct HostLogUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    manager: Arc<M>,
    _marker: PhantomData<R>,
}

impl<M, R> HostLogUseCase<M, R>
where
    M: RepositoryManager<R>,
    R: RepositoriesExt + Send + Sync + 'static,
{
    pub async fn list_logs(
        &self,
        limit: i64,
        offset: i64,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<Vec<NativeHostLogRow>> {
        self.manager.run(move |repos| Box::pin(async move { repos.host_log().list_logs(limit, offset, level, typ).await })).await
    }

    pub async fn count_logs(
        &self,
        level: Option<HostLogLevel>,
        typ: Option<HostLogType>,
    ) -> anyhow::Result<i64> {
        self.manager.run(move |repos| Box::pin(async move { repos.host_log().count_logs(level, typ).await })).await
    }
}


