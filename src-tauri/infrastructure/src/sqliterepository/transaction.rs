use domain::repository::{RepositoriesExt, transaction::TransactionRepository};
use std::sync::Arc;
use sqlx::{Pool, Sqlite};
use sqlx::pool::PoolConnection;
use futures::future::BoxFuture;
use std::mem;

use crate::sqliterepository::sqliterepository::{RepositoryExecutor, SqliteRepositories};

impl TransactionRepository for SqliteRepositories {
    async fn with_transaction<F, R>(&mut self, f: F) -> anyhow::Result<R>
    where
        for<'cx> F: FnOnce(&'cx mut Self) -> BoxFuture<'cx, anyhow::Result<R>> + Send,
        R: Send,
    {
        let pool = match &*self.executor {
            RepositoryExecutor::Pool(p) => (*p).clone(),
            RepositoryExecutor::Conn(_) => {
                anyhow::bail!("with_transaction は Pool 実行時のみ呼び出し可能です (Conn 内では unreachable) ");
            }
        };

        let mut conn = pool.acquire().await?;
        sqlx::query("BEGIN IMMEDIATE").execute(&mut conn).await?;

        let original = mem::replace(Arc::get_mut(&mut self.executor).unwrap(), RepositoryExecutor::Conn(Arc::new(tokio::sync::Mutex::new(conn))));
        let result = f(self).await;
        let conn_arc = match mem::replace(Arc::get_mut(&mut self.executor).unwrap(), original) {
            RepositoryExecutor::Conn(conn) => conn,
            _ => unreachable!(),
        };

        match result {
            Ok(value) => {
                let mut conn = conn_arc.lock().await;
                sqlx::query("COMMIT").execute(&mut *conn).await?;
                Ok(value)
            }
            Err(err) => {
                let mut conn = conn_arc.lock().await;
                sqlx::query("ROLLBACK").execute(&mut *conn).await?;
                Err(err)
            }
        }
    }
}
