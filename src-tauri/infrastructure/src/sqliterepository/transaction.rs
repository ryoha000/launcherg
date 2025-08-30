use domain::repositoryv2::{RepositoriesExt, transaction::TransactionRepository};
use sqlx::{Pool, Sqlite};
use sqlx::pool::PoolConnection;
use futures::future::BoxFuture;
use std::mem;

use crate::sqliterepository::sqliterepository::{RepositoryExecutor, SqliteRepository};

impl<'a> TransactionRepository for SqliteRepository<'a> {
    async fn with_transaction<F, R>(&mut self, f: F) -> anyhow::Result<R>
    where
        for<'cx> F: FnOnce(&'cx mut Self) -> BoxFuture<'cx, anyhow::Result<R>> + Send,
        R: Send,
    {
        let pool = match &self.executor {
            RepositoryExecutor::Pool(p) => (*p).clone(),
            RepositoryExecutor::OwnedPool(p) => (**p).clone(),
            RepositoryExecutor::OwnedConn(_) => {
                anyhow::bail!("with_transaction は Pool 実行時のみ呼び出し可能です (Conn 内では unreachable) ");
            }
        };

        let mut conn = pool.acquire().await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let original = mem::replace(&mut self.executor, RepositoryExecutor::OwnedConn(conn));
        let result = f(self).await;
        let conn = match mem::replace(&mut self.executor, original) {
            RepositoryExecutor::OwnedConn(conn) => conn,
            _ => unreachable!(),
        };
        let mut conn = conn;

        match result {
            Ok(value) => {
                sqlx::query("COMMIT").execute(&mut conn).await?;
                Ok(value)
            }
            Err(err) => {
                sqlx::query("ROLLBACK").execute(&mut conn).await?;
                Err(err)
            }
        }
    }
}
