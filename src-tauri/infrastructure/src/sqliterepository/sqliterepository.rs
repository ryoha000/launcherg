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
    type TransactionRepo = SqliteRepository<'a>;

    fn work(&mut self) -> &mut Self::WorkRepo { self }
    fn transaction(&mut self) -> &mut Self::TransactionRepo { self }
}

impl<'a> SqliteRepository<'a> {
    pub fn new(executor: RepositoryExecutor<'a>) -> Self { Self { executor } }
}
