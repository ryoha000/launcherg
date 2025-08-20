use domain::repository::native_host_log::NativeHostLogRepository;
use domain::native_host_log::{HostLogLevel, HostLogType};
use crate::repositoryimpl::repository::RepositoriesExt;
use super::TestDatabase;

#[tokio::test]
async fn native_host_log_insert() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.repositories.host_log_repository();

    // insert log
    let res = repo.insert_log(HostLogLevel::Info, HostLogType::ReceiveDmmSyncGamesRequest, "received").await;
    assert!(res.is_ok());

    // count rows
    let cnt: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM native_messaging_host_logs").fetch_one(&test_db.pool).await.unwrap();
    assert_eq!(cnt.0, 1);
}


