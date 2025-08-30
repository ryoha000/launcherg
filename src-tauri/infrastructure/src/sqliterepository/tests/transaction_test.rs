use super::TestDatabase;
use futures::{FutureExt, future::BoxFuture};
use domain::repository::{RepositoriesExt, works::WorkRepository, manager::RepositoryManager};
use std::sync::Arc;
use domain::works::NewWork;

#[tokio::test]
async fn トランザクション成功時にコミットされる() -> anyhow::Result<()> {
    let test_db = TestDatabase::new().await?;
    let manager = crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(test_db.pool.clone()));
    let title = "work_tx_ok".to_string();
    manager.run_in_transaction(|repos| -> BoxFuture<'_, anyhow::Result<()>> { async move {
        repos.work().upsert(&NewWork { title: title.clone() }).await?;
        Ok(())
    }.boxed() }).await?;

    let found = manager.run(|repos| -> BoxFuture<'_, anyhow::Result<_>> { async move { repos.work().find_by_title("work_tx_ok").await }.boxed() }).await?;
    assert!(found.is_some());
    Ok(())
}

#[tokio::test]
async fn トランザクション失敗時にロールバックされる() -> anyhow::Result<()> {
    let test_db = TestDatabase::new().await?;
    let manager = crate::sqliterepository::sqliterepository::SqliteRepositoryManager::new(Arc::new(test_db.pool.clone()));
    let _err = manager.run_in_transaction(|repos| -> BoxFuture<'_, anyhow::Result<()>> { async move {
        repos.work().upsert(&NewWork { title: "work_tx_ng".to_string() }).await?;
        anyhow::bail!("force rollback")
    }.boxed() }).await.unwrap_err();

    let found = manager.run(|repos| -> BoxFuture<'_, anyhow::Result<_>> { async move { repos.work().find_by_title("work_tx_ng").await }.boxed() }).await?;
    assert!(found.is_none());
    Ok(())
}


