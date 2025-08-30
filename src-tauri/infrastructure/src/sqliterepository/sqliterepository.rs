use domain::repositoryv2::{RepositoriesExt};
use sqlx::{Pool, Sqlite};
use sqlx::pool::PoolConnection;
use futures::future::BoxFuture;

pub enum RepositoryExecutor<'a> {
    Pool(&'a Pool<Sqlite>),
    OwnedConn(PoolConnection<Sqlite>),
}

impl RepositoryExecutor<'_> {
    pub async fn with_conn<F, T>(&mut self, f: F) -> Result<T, anyhow::Error>
    where
        F: for<'c> FnOnce(&'c mut PoolConnection<Sqlite>)
              -> BoxFuture<'c, Result<T, anyhow::Error>> + Send,
        T: Send,
    {
        match self {
            RepositoryExecutor::Pool(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut conn).await
            }
            RepositoryExecutor::OwnedConn(conn) => f(conn).await,
        }
    }
}

pub struct SqliteRepository<'a> {
    pub executor: RepositoryExecutor<'a>,
}

impl<'a> RepositoriesExt for SqliteRepository<'a> {
    type WorkRepo = SqliteRepository<'a>;
    type DmmWorkRepo = SqliteRepository<'a>;
    type DlsiteWorkRepo = SqliteRepository<'a>;
    type AllGameCacheRepo = SqliteRepository<'a>;
    type ExploredCacheRepo = SqliteRepository<'a>;
    type ImageQueueRepo = SqliteRepository<'a>;
    type HostLogRepo = SqliteRepository<'a>;
    type WorkOmitRepo = SqliteRepository<'a>;
    type WorkParentPacksRepo = SqliteRepository<'a>;
    type DmmPackRepo = SqliteRepository<'a>;
    type CollectionRepo = SqliteRepository<'a>;
    type TransactionRepo = SqliteRepository<'a>;

    fn work(&mut self) -> &mut Self::WorkRepo { self }
    fn dmm_work(&mut self) -> &mut Self::DmmWorkRepo { self }
    fn dlsite_work(&mut self) -> &mut Self::DlsiteWorkRepo { self }
    fn all_game_cache(&mut self) -> &mut Self::AllGameCacheRepo { self }
    fn explored_cache(&mut self) -> &mut Self::ExploredCacheRepo { self }
    fn image_queue(&mut self) -> &mut Self::ImageQueueRepo { self }
    fn host_log(&mut self) -> &mut Self::HostLogRepo { self }
    fn work_omit(&mut self) -> &mut Self::WorkOmitRepo { self }
    fn work_parent_packs(&mut self) -> &mut Self::WorkParentPacksRepo { self }
    fn dmm_pack(&mut self) -> &mut Self::DmmPackRepo { self }
    fn collection(&mut self) -> &mut Self::CollectionRepo { self }
    fn transaction(&mut self) -> &mut Self::TransactionRepo { self }
}

impl<'a> SqliteRepository<'a> {
    pub fn new(executor: RepositoryExecutor<'a>) -> Self { Self { executor } }
}
