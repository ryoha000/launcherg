use super::TestDatabase;
use domain::native_host_log::{HostLogLevel, HostLogType};
use domain::repository::{native_host_log::NativeHostLogRepository, RepositoriesExt};

#[tokio::test]
async fn native_host_log_normal_flows() {
    let test_db = TestDatabase::new().await.unwrap();
    let repo = test_db.sqlite_repository();

    // insert
    {
        let mut r = repo.host_log();
        r.insert_log(HostLogLevel::Debug, HostLogType::Unknown, "hello")
            .await
            .unwrap();
    }

    // list
    {
        let mut r = repo.host_log();
        let rows = r.list_logs(10, 0, None, None).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].message, "hello");
    }

    // count
    {
        let mut r = repo.host_log();
        let cnt = r.count_logs(None, None).await.unwrap();
        assert_eq!(cnt, 1);
    }
}
