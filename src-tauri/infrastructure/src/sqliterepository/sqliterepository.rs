use std::{marker::PhantomData};

use domain::repositoryv2::RepositoriesExt;
use sqlx::{Pool, Sqlite, SqliteConnection, Transaction};
use futures::future::BoxFuture;

pub enum RepositoryExecutor<'a> {
    Pool(&'a Pool<Sqlite>),
    Conn(&'a mut SqliteConnection),
}

impl RepositoryExecutor<'_> {
    pub async fn with_conn<F, T>(&mut self, f: F) -> Result<T, anyhow::Error>
    where
        F: for<'c> FnOnce(&'c mut SqliteConnection)
              -> BoxFuture<'c, Result<T, anyhow::Error>> + Send,
        T: Send,
    {
        match self {
            RepositoryExecutor::Pool(pool) => {
                let mut conn = pool.acquire().await?;
                f(&mut *conn).await
            }
            RepositoryExecutor::Conn(conn) => f(&mut **conn).await,
        }
    }
}


#[derive(derive_new::new)]
pub struct RepositoryImpl<'a, T> {
    pub executor: RepositoryExecutor<'a>,
    _marker: PhantomData<T>,
}

pub struct SqliteRepository<'a> {
    work_repository: RepositoryImpl<'a, domain::works::Work>,
}

impl<'a> RepositoriesExt for SqliteRepository<'a> {
    type WorkRepo = RepositoryImpl<'a, domain::works::Work>;

    fn work(&mut self) -> &mut Self::WorkRepo {
        &mut self.work_repository
    }
}

impl<'a> SqliteRepository<'a> {
    pub fn new(executor: RepositoryExecutor<'a>) -> Self {
        let work_repository = RepositoryImpl::new(executor);
        Self {
            work_repository,
        }
    }

    // 内部ユーティリティ: &mut Transaction の借用をこの関数スコープに閉じ込めることで、
    // commit/rollback 時に所有権移動が可能になるようにする。
    

    
	pub async fn with_transaction<F, R>(&mut self, f: F) -> anyhow::Result<R>
	where
		for<'cx> F: FnOnce(&'cx mut SqliteRepository<'cx>) -> BoxFuture<'cx, anyhow::Result<R>> + Send,
		R: Send,
	{
		match &self.work_repository.executor {
			RepositoryExecutor::Pool(pool) => {
				let mut conn = pool.acquire().await?;
				sqlx::query("BEGIN IMMEDIATE").execute(&mut *conn).await?;
				let result = {
					let mut repos: SqliteRepository<'_> = SqliteRepository::new(RepositoryExecutor::Conn(&mut conn));
					f(&mut repos).await
				};
				match result {
					Ok(value) => {
						sqlx::query("COMMIT").execute(&mut *conn).await?;
						Ok(value)
					}
					Err(err) => {
						sqlx::query("ROLLBACK").execute(&mut *conn).await?;
						Err(err)
					}
				}
			}
			RepositoryExecutor::Conn(_) => {
				anyhow::bail!("with_transaction は Pool 実行時のみ呼び出し可能です (Conn 内では unreachable) ");
			}
		}
	}
}
