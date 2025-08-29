use super::TestDatabase;
use futures::{FutureExt, future::BoxFuture};
use domain::repositoryv2::{RepositoriesExt, works::WorkRepository};
use domain::works::NewWork;

#[tokio::test]
async fn トランザクション成功時にコミットされる() -> anyhow::Result<()> {
    let test_db = TestDatabase::new().await?;
    let mut repos = test_db.sqlite_repository();

    let title = "work_tx_ok".to_string();
    let _ = repos
        .with_transaction(|r| -> BoxFuture<'_, anyhow::Result<()>> { async move {
            r.work().upsert(&NewWork { title: title.clone() }).await?;
            Ok(())
        }
        .boxed() })
        .await?;

    let found = repos.work().find_by_title("work_tx_ok").await?;
    assert!(found.is_some());
    Ok(())
}

#[tokio::test]
async fn トランザクション失敗時にロールバックされる() -> anyhow::Result<()> {
    let test_db = TestDatabase::new().await?;
    let mut repos = test_db.sqlite_repository();

    let _err = repos
        .with_transaction(|r| -> BoxFuture<'_, anyhow::Result<()>> { async move {
            r.work().upsert(&NewWork { title: "work_tx_ng".to_string() }).await?;
            anyhow::bail!("force rollback")
        }
        .boxed() })
        .await
        .unwrap_err();

    let found = repos.work().find_by_title("work_tx_ng").await?;
    assert!(found.is_none());
    Ok(())
}


