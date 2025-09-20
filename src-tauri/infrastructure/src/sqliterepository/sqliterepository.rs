use domain::repository::{manager::RepositoryManager, RepositoriesExt};
use futures::future::BoxFuture;
use futures::FutureExt;
use sqlx::pool::PoolConnection;
use sqlx::{Pool, Sqlite};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum RepositoryExecutor {
    Pool(Arc<Pool<Sqlite>>),
    Conn(Arc<Mutex<PoolConnection<Sqlite>>>),
}

impl RepositoryExecutor {
    pub async fn with_conn<F, T>(&self, f: F) -> Result<T, anyhow::Error>
    where
        F: for<'c> FnOnce(
                &'c mut PoolConnection<Sqlite>,
            ) -> BoxFuture<'c, Result<T, anyhow::Error>>
            + Send,
        T: Send,
    {
        match self {
            RepositoryExecutor::Pool(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut conn).await
            }
            RepositoryExecutor::Conn(conn) => {
                let mut guard = conn.lock().await;
                f(&mut *guard).await
            }
        }
    }
}

#[derive(Clone, derive_new::new)]
pub struct RepositoryImpl<T> {
    pub executor: Arc<RepositoryExecutor>,
    _marker: PhantomData<T>,
}

pub struct SqliteRepositories {
    pub executor: Arc<RepositoryExecutor>,

    work: RepositoryImpl<domain::works::Work>,
    dmm_work: RepositoryImpl<domain::works::DmmWork>,
    dlsite_work: RepositoryImpl<domain::works::DlsiteWork>,
    all_game_cache: RepositoryImpl<domain::all_game_cache::AllGameCache>,
    explored_cache: RepositoryImpl<domain::explored_cache::ExploredCache>,
    image_queue: RepositoryImpl<domain::save_image_queue::ImageSaveQueueRow>,
    host_log: RepositoryImpl<domain::native_host_log::NativeHostLogRow>,
    work_omit: RepositoryImpl<domain::work_omit::WorkOmit>,
    work_parent_packs: RepositoryImpl<domain::work_parent_pack::WorkParentPack>,
    dmm_pack: RepositoryImpl<domain::dmm_work_pack::DmmWorkPack>,
    collection: RepositoryImpl<domain::collection::CollectionElement>,
    work_download_path: RepositoryImpl<domain::work_download_path::WorkDownloadPath>,
    work_lnk: RepositoryImpl<domain::repository::work_lnk::WorkLnk>,
}

impl RepositoriesExt for SqliteRepositories {
    type WorkRepo = RepositoryImpl<domain::works::Work>;
    type DmmWorkRepo = RepositoryImpl<domain::works::DmmWork>;
    type DlsiteWorkRepo = RepositoryImpl<domain::works::DlsiteWork>;
    type AllGameCacheRepo = RepositoryImpl<domain::all_game_cache::AllGameCache>;
    type ExploredCacheRepo = RepositoryImpl<domain::explored_cache::ExploredCache>;
    type ImageQueueRepo = RepositoryImpl<domain::save_image_queue::ImageSaveQueueRow>;
    type HostLogRepo = RepositoryImpl<domain::native_host_log::NativeHostLogRow>;
    type WorkOmitRepo = RepositoryImpl<domain::work_omit::WorkOmit>;
    type WorkParentPacksRepo = RepositoryImpl<domain::work_parent_pack::WorkParentPack>;
    type DmmPackRepo = RepositoryImpl<domain::dmm_work_pack::DmmWorkPack>;
    type CollectionRepo = RepositoryImpl<domain::collection::CollectionElement>;
    type WorkDownloadPathRepo = RepositoryImpl<domain::work_download_path::WorkDownloadPath>;
    type WorkLnkRepo = RepositoryImpl<domain::repository::work_lnk::WorkLnk>;

    fn work(&self) -> Self::WorkRepo {
        self.work.clone()
    }
    fn dmm_work(&self) -> Self::DmmWorkRepo {
        self.dmm_work.clone()
    }
    fn dlsite_work(&self) -> Self::DlsiteWorkRepo {
        self.dlsite_work.clone()
    }
    fn all_game_cache(&self) -> Self::AllGameCacheRepo {
        self.all_game_cache.clone()
    }
    fn explored_cache(&self) -> Self::ExploredCacheRepo {
        self.explored_cache.clone()
    }
    fn image_queue(&self) -> Self::ImageQueueRepo {
        self.image_queue.clone()
    }
    fn host_log(&self) -> Self::HostLogRepo {
        self.host_log.clone()
    }
    fn work_omit(&self) -> Self::WorkOmitRepo {
        self.work_omit.clone()
    }
    fn work_parent_packs(&self) -> Self::WorkParentPacksRepo {
        self.work_parent_packs.clone()
    }
    fn dmm_pack(&self) -> Self::DmmPackRepo {
        self.dmm_pack.clone()
    }
    fn collection(&self) -> Self::CollectionRepo {
        self.collection.clone()
    }
    fn work_download_path(&self) -> Self::WorkDownloadPathRepo {
        self.work_download_path.clone()
    }
    fn work_lnk(&self) -> Self::WorkLnkRepo {
        self.work_lnk.clone()
    }
}

impl SqliteRepositories {
    pub fn new_from_pool(pool: Arc<Pool<Sqlite>>) -> Self {
        let exec = RepositoryExecutor::Pool(pool);
        Self::new(Arc::new(exec))
    }
    pub fn new_from_conn(conn: PoolConnection<Sqlite>) -> Self {
        let exec = RepositoryExecutor::Conn(Arc::new(Mutex::new(conn)));
        Self::new(Arc::new(exec))
    }
    pub fn new_from_conn_arc(conn: Arc<Mutex<PoolConnection<Sqlite>>>) -> Self {
        let exec = RepositoryExecutor::Conn(conn);
        Self::new(Arc::new(exec))
    }
    fn new(executor: Arc<RepositoryExecutor>) -> Self {
        Self {
            executor: executor.clone(),

            work: RepositoryImpl::new(executor.clone()),
            dmm_work: RepositoryImpl::new(executor.clone()),
            dlsite_work: RepositoryImpl::new(executor.clone()),
            all_game_cache: RepositoryImpl::new(executor.clone()),
            explored_cache: RepositoryImpl::new(executor.clone()),
            image_queue: RepositoryImpl::new(executor.clone()),
            host_log: RepositoryImpl::new(executor.clone()),
            work_omit: RepositoryImpl::new(executor.clone()),
            work_parent_packs: RepositoryImpl::new(executor.clone()),
            dmm_pack: RepositoryImpl::new(executor.clone()),
            collection: RepositoryImpl::new(executor.clone()),
            work_download_path: RepositoryImpl::new(executor.clone()),
            work_lnk: RepositoryImpl::new(executor.clone()),
        }
    }
}

pub struct SqliteRepositoryManager {
    pool: Arc<Pool<Sqlite>>,
}

impl SqliteRepositoryManager {
    pub fn new(pool: Arc<Pool<Sqlite>>) -> Self {
        Self { pool }
    }
}

impl RepositoryManager<SqliteRepositories> for SqliteRepositoryManager {
    fn run<'a, T: Send + 'a>(
        &'a self,
        f: impl FnOnce(SqliteRepositories) -> BoxFuture<'a, anyhow::Result<T>> + Send + 'a,
    ) -> BoxFuture<'a, anyhow::Result<T>> {
        let repos = SqliteRepositories::new_from_pool(self.pool.clone());
        async move { f(repos).await }.boxed()
    }

    fn run_in_transaction<'a, T: Send + 'a>(
        &'a self,
        f: impl FnOnce(SqliteRepositories) -> BoxFuture<'a, anyhow::Result<T>> + Send + 'a,
    ) -> BoxFuture<'a, anyhow::Result<T>> {
        let pool = self.pool.clone();
        async move {
            let mut conn = pool.acquire().await?;
            sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;
            let conn_arc = Arc::new(Mutex::new(conn));
            let repos = SqliteRepositories::new_from_conn_arc(conn_arc.clone());
            let res = f(repos).await;
            match res {
                Ok(v) => {
                    let mut guard = conn_arc.lock().await;
                    sqlx::query("COMMIT").execute(&mut *guard).await?;
                    Ok(v)
                }
                Err(e) => {
                    let mut guard = conn_arc.lock().await;
                    let _ = sqlx::query("ROLLBACK").execute(&mut *guard).await;
                    Err(e)
                }
            }
        }
        .boxed()
    }
}
